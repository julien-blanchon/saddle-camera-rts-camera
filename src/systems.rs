use bevy::{
    gizmos::prelude::*,
    math::{Dir3, StableInterpolate},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
    prelude::*,
};

use crate::{
    RtsCamera, RtsCameraBookmark, RtsCameraBookmarkRecalled, RtsCameraBookmarkStored,
    RtsCameraBookmarks, RtsCameraBoundsMode, RtsCameraDebug, RtsCameraFlyToApplied,
    RtsCameraFollow, RtsCameraGround, RtsCameraInput, RtsCameraInternalState,
    RtsCameraRotationPivotMode, RtsCameraRuntime, RtsCameraSettings, RtsCameraZoomAnchorMode,
    math::{
        apply_bounds_delta, camera_pitch_for_distance, camera_transform_from_state, clamp_distance,
        clamp_focus_to_bounds, pan_vector_from_yaw, resolve_ground_height_target, smooth_angle,
        smooth_scalar, smoothing_factor,
    },
};

pub(crate) fn initialize_added_cameras(
    mut cameras: Query<
        (
            &RtsCamera,
            &RtsCameraSettings,
            &mut RtsCameraRuntime,
            &mut RtsCameraInternalState,
            &mut Transform,
        ),
        Added<RtsCamera>,
    >,
) {
    for (camera, settings, mut runtime, mut internal, mut transform) in &mut cameras {
        runtime.focus = camera.target_focus;
        runtime.yaw = camera.target_yaw;
        runtime.distance = clamp_distance(camera.target_distance, &settings.distance);
        runtime.pitch =
            camera_pitch_for_distance(runtime.distance, &settings.distance, &settings.pitch);
        *transform = camera_transform_from_state(
            runtime.focus,
            runtime.yaw,
            runtime.pitch,
            runtime.distance,
        );
        runtime.last_ground_hit = None;
        runtime.last_cursor_anchor = None;
        internal.drag_anchor_world = None;
    }
}

pub(crate) fn sync_follow_targets(
    mut cameras: Query<(&mut RtsCamera, &RtsCameraSettings, Option<&RtsCameraFollow>)>,
    targets: Query<&GlobalTransform>,
) {
    for (mut camera, settings, follow) in &mut cameras {
        let Some(follow) = follow else {
            continue;
        };
        if !follow.enabled || !settings.controls.follow {
            continue;
        }
        let Ok(target) = targets.get(follow.target) else {
            continue;
        };
        camera.target_focus = target.translation() + follow.offset;
        if follow.snap {
            camera.snap = true;
        }
    }
}

pub(crate) fn apply_programmatic_commands(
    mut cameras: Query<
        (
            Entity,
            &mut RtsCamera,
            &RtsCameraRuntime,
            &mut RtsCameraBookmarks,
            &mut RtsCameraInput,
        ),
        With<RtsCamera>,
    >,
    mut bookmark_stored: MessageWriter<RtsCameraBookmarkStored>,
    mut bookmark_recalled: MessageWriter<RtsCameraBookmarkRecalled>,
    mut fly_to_applied: MessageWriter<RtsCameraFlyToApplied>,
) {
    for (entity, mut camera, runtime, mut bookmarks, input) in &mut cameras {
        if let Some(slot) = input.set_bookmark_slot {
            let bookmark = RtsCameraBookmark::from_runtime(runtime);
            bookmarks.set(slot, bookmark);
            bookmark_stored.write(RtsCameraBookmarkStored {
                camera: entity,
                slot,
                bookmark,
            });
        }

        if let Some(slot) = input.recall_bookmark_slot
            && let Some(bookmark) = bookmarks.get(slot)
        {
            camera.target_focus = bookmark.focus;
            camera.target_yaw = bookmark.yaw;
            camera.target_distance = bookmark.distance;
            camera.snap |= input.recall_bookmark_snap;
            bookmark_recalled.write(RtsCameraBookmarkRecalled {
                camera: entity,
                slot,
                bookmark,
            });
        }

        if let Some(focus) = input.fly_to_focus {
            camera.target_focus = focus;
            if let Some(yaw) = input.fly_to_yaw {
                camera.target_yaw = yaw;
            }
            if let Some(distance) = input.fly_to_distance {
                camera.target_distance = distance;
            }
            camera.snap |= input.fly_to_snap;
            fly_to_applied.write(RtsCameraFlyToApplied {
                camera: entity,
                focus: camera.target_focus,
                yaw: camera.target_yaw,
                distance: camera.target_distance,
                snap: input.fly_to_snap,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_camera_input(
    time: Res<Time>,
    mut ray_cast: MeshRayCast,
    ground: Query<Entity, With<RtsCameraGround>>,
    mut cameras: Query<
        (
            &Camera,
            &GlobalTransform,
            &mut RtsCamera,
            &RtsCameraSettings,
            &mut RtsCameraRuntime,
            Option<&mut RtsCameraFollow>,
            &mut RtsCameraInput,
            &mut RtsCameraInternalState,
        ),
        With<RtsCamera>,
    >,
) {
    let dt = time.delta_secs();
    for (
        camera_component,
        camera_transform,
        mut camera,
        settings,
        mut runtime,
        follow,
        input,
        mut internal,
    ) in &mut cameras
    {
        let mut follow = follow;
        let target_distance = clamp_distance(camera.target_distance, &settings.distance);
        let distance_t = ((target_distance - settings.distance.min)
            / (settings.distance.max - settings.distance.min).max(f32::EPSILON))
        .clamp(0.0, 1.0);
        let pan_speed = settings
            .motion
            .pan_speed_near
            .lerp(settings.motion.pan_speed_far, distance_t);

        let mut cursor_anchor = input.cursor_position.and_then(|cursor| {
            cursor_ground_anchor(
                camera_component,
                camera_transform,
                cursor,
                runtime.focus.y,
                &mut ray_cast,
                &ground,
            )
        });

        if input.drag_pan_active && settings.controls.drag_pan {
            if internal.drag_anchor_world.is_none() {
                internal.drag_anchor_world = cursor_anchor;
            } else if let (Some(anchor), Some(current_hit)) =
                (internal.drag_anchor_world, cursor_anchor)
            {
                let delta = anchor - current_hit;
                apply_focus_delta(
                    &mut camera,
                    settings,
                    follow.as_deref_mut(),
                    Vec2::new(delta.x, delta.z),
                );
            }
        } else {
            internal.drag_anchor_world = None;
        }

        let mut total_pan = Vec2::ZERO;
        if settings.controls.pan {
            total_pan += input.pan;
        }
        if settings.controls.edge_pan {
            total_pan += input.edge_pan;
        }

        if total_pan.length_squared() > 0.0 {
            let desired = pan_vector_from_yaw(camera.target_yaw, total_pan.normalize_or_zero())
                * pan_speed
                * dt;
            let desired_xz = Vec2::new(desired.x, desired.z);
            let applied = settings
                .bounds
                .as_ref()
                .map(|bounds| {
                    if matches!(bounds.mode, RtsCameraBoundsMode::Soft) {
                        apply_bounds_delta(
                            Vec2::new(camera.target_focus.x, camera.target_focus.z),
                            desired_xz,
                            bounds,
                        )
                    } else {
                        desired_xz
                    }
                })
                .unwrap_or(desired_xz);
            apply_focus_delta(&mut camera, settings, follow.as_deref_mut(), applied);
        }

        if settings.controls.rotation {
            let mut yaw_delta = input.rotate * settings.motion.rotation_speed * dt;
            if input.drag_rotate_active {
                yaw_delta -= input.rotate_drag_delta * settings.motion.drag_rotation_speed;
            }

            if yaw_delta.abs() > f32::EPSILON {
                let anchor =
                    if settings.anchors.rotation_pivot == RtsCameraRotationPivotMode::Cursor {
                        cursor_anchor
                    } else {
                        None
                    };
                camera.target_yaw += yaw_delta;
                if let Some(anchor_before) = anchor
                    && let Some(anchor_after) = predicted_cursor_anchor(
                        camera_component,
                        input.cursor_position,
                        &camera,
                        settings,
                        &mut ray_cast,
                        &ground,
                    )
                {
                    let delta = anchor_before - anchor_after;
                    apply_focus_delta(
                        &mut camera,
                        settings,
                        follow.as_deref_mut(),
                        Vec2::new(delta.x, delta.z),
                    );
                    cursor_anchor = Some(anchor_before);
                }
            }
        }

        if settings.controls.zoom && input.zoom.abs() > f32::EPSILON {
            let use_cursor_anchor = input.zoom_to_cursor
                || settings.anchors.zoom_anchor == RtsCameraZoomAnchorMode::Cursor;
            let anchor = if use_cursor_anchor {
                cursor_anchor
            } else {
                None
            };
            camera.target_distance = clamp_distance(
                camera.target_distance - input.zoom * settings.motion.zoom_speed,
                &settings.distance,
            );

            if let Some(anchor_before) = anchor
                && let Some(anchor_after) = predicted_cursor_anchor(
                    camera_component,
                    input.cursor_position,
                    &camera,
                    settings,
                    &mut ray_cast,
                    &ground,
                )
            {
                let delta = anchor_before - anchor_after;
                apply_focus_delta(
                    &mut camera,
                    settings,
                    follow.as_deref_mut(),
                    Vec2::new(delta.x, delta.z),
                );
                cursor_anchor = Some(anchor_before);
            }
        }

        runtime.last_cursor_anchor = cursor_anchor;

        if camera.snap {
            runtime.focus = Vec3::new(
                camera.target_focus.x,
                runtime.ground_height.unwrap_or(camera.target_focus.y),
                camera.target_focus.z,
            );
            runtime.yaw = camera.target_yaw;
            runtime.distance = clamp_distance(camera.target_distance, &settings.distance);
            runtime.pitch =
                camera_pitch_for_distance(runtime.distance, &settings.distance, &settings.pitch);
        }
    }
}

pub(crate) fn resolve_ground_height(
    mut ray_cast: MeshRayCast,
    ground: Query<Entity, With<RtsCameraGround>>,
    mut cameras: Query<(&RtsCamera, &RtsCameraSettings, &mut RtsCameraRuntime), With<RtsCamera>>,
) {
    for (camera, settings, mut runtime) in &mut cameras {
        if !settings.ground.enabled {
            runtime.ground_height = Some(camera.target_focus.y);
            runtime.last_ground_hit = None;
            continue;
        }

        let probe_origin = Vec3::new(
            camera.target_focus.x,
            camera.target_focus.y + settings.ground.probe_height,
            camera.target_focus.z,
        );
        let Some(ray) = Dir3::new(Vec3::NEG_Y)
            .ok()
            .map(|dir| Ray3d::new(probe_origin, dir))
        else {
            continue;
        };
        let filter = |entity| ground.get(entity).is_ok();
        let settings_ray = MeshRayCastSettings {
            filter: &filter,
            ..default()
        };

        let hit = ray_cast
            .cast_ray(ray, &settings_ray)
            .first()
            .map(|(_, hit)| hit.clone());
        runtime.last_ground_hit = hit.as_ref().map(|hit| hit.point);
        runtime.ground_height = resolve_ground_height_target(
            runtime.ground_height,
            hit.as_ref().map(|hit| hit.point.y),
            &settings.ground,
            camera.target_focus.y,
        );
    }
}

pub(crate) fn apply_bounds(
    mut cameras: Query<
        (
            &mut RtsCamera,
            &RtsCameraSettings,
            Option<&mut RtsCameraFollow>,
        ),
        With<RtsCamera>,
    >,
) {
    for (mut camera, settings, follow) in &mut cameras {
        if let Some(bounds) = &settings.bounds {
            let clamped = clamp_focus_to_bounds(camera.target_focus, bounds);
            let delta = clamped - camera.target_focus;
            camera.target_focus = clamped;

            if let Some(mut follow) = follow
                && follow.enabled
                && settings.controls.follow
            {
                follow.offset += delta;
            }
        }
    }
}

pub(crate) fn advance_runtime(
    time: Res<Time>,
    mut cameras: Query<
        (&mut RtsCamera, &mut RtsCameraRuntime, &RtsCameraSettings),
        With<RtsCamera>,
    >,
) {
    let dt = time.delta_secs();
    for (mut camera, mut runtime, settings) in &mut cameras {
        let target_distance = clamp_distance(camera.target_distance, &settings.distance);
        if camera.snap {
            runtime.focus = Vec3::new(
                camera.target_focus.x,
                runtime.ground_height.unwrap_or(camera.target_focus.y),
                camera.target_focus.z,
            );
            runtime.yaw = camera.target_yaw;
            runtime.distance = target_distance;
        } else {
            let target_focus = Vec3::new(
                camera.target_focus.x,
                runtime.ground_height.unwrap_or(camera.target_focus.y),
                camera.target_focus.z,
            );
            let horizontal_target = Vec2::new(target_focus.x, target_focus.z);
            let focus_factor = smoothing_factor(settings.motion.focus_decay, dt);
            let horizontal_current = Vec2::new(runtime.focus.x, runtime.focus.z)
                .interpolate_stable(&horizontal_target, focus_factor);
            runtime.focus.x = horizontal_current.x;
            runtime.focus.z = horizontal_current.y;
            runtime.focus.y = smooth_scalar(
                runtime.focus.y,
                target_focus.y,
                settings.motion.ground_decay,
                dt,
            );
            runtime.yaw = smooth_angle(
                runtime.yaw,
                camera.target_yaw,
                settings.motion.yaw_decay,
                dt,
            );
            runtime.distance = smooth_scalar(
                runtime.distance,
                target_distance,
                settings.motion.distance_decay,
                dt,
            );
        }

        runtime.pitch =
            camera_pitch_for_distance(runtime.distance, &settings.distance, &settings.pitch);
        camera.snap = false;
    }
}

pub(crate) fn clear_consumed_input(mut inputs: Query<&mut RtsCameraInput, With<RtsCamera>>) {
    for mut input in &mut inputs {
        *input = RtsCameraInput::default();
    }
}

pub(crate) fn sync_transform(
    mut cameras: Query<(&RtsCameraRuntime, &mut Transform), With<RtsCamera>>,
) {
    for (runtime, mut transform) in &mut cameras {
        *transform = camera_transform_from_state(
            runtime.focus,
            runtime.yaw,
            runtime.pitch,
            runtime.distance,
        );
    }
}

pub(crate) fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    cameras: Query<
        (
            &RtsCameraRuntime,
            &RtsCameraSettings,
            Option<&RtsCameraDebug>,
        ),
        With<RtsCamera>,
    >,
) {
    for (runtime, settings, debug) in &cameras {
        if !debug.is_some_and(|debug| debug.enabled) {
            continue;
        }

        gizmos.sphere(runtime.focus, 0.25, Color::srgb(0.95, 0.75, 0.2));
        if let Some(hit) = runtime.last_ground_hit {
            gizmos.sphere(hit, 0.18, Color::srgb(0.25, 0.9, 0.55));
        }
        if let Some(anchor) = runtime.last_cursor_anchor {
            gizmos.circle(
                Isometry3d::new(anchor + Vec3::Y * 0.03, Quat::IDENTITY),
                0.4,
                Color::srgb(0.3, 0.8, 1.0),
            );
        }
        if let Some(bounds) = &settings.bounds {
            let y =
                runtime.ground_height.unwrap_or(runtime.focus.y) - settings.ground.clearance + 0.05;
            let corners = [
                Vec3::new(bounds.min.x, y, bounds.min.y),
                Vec3::new(bounds.max.x, y, bounds.min.y),
                Vec3::new(bounds.max.x, y, bounds.max.y),
                Vec3::new(bounds.min.x, y, bounds.max.y),
            ];
            gizmos.linestrip(
                [corners[0], corners[1], corners[2], corners[3], corners[0]],
                Color::srgb(0.95, 0.2, 0.25),
            );
        }
    }
}

fn predicted_cursor_anchor(
    camera: &Camera,
    cursor_position: Option<Vec2>,
    rts_camera: &RtsCamera,
    settings: &RtsCameraSettings,
    ray_cast: &mut MeshRayCast<'_, '_>,
    ground: &Query<Entity, With<RtsCameraGround>>,
) -> Option<Vec3> {
    let cursor_position = cursor_position?;
    let predicted_distance = clamp_distance(rts_camera.target_distance, &settings.distance);
    let predicted_pitch =
        camera_pitch_for_distance(predicted_distance, &settings.distance, &settings.pitch);
    let predicted_transform = camera_transform_from_state(
        rts_camera.target_focus,
        rts_camera.target_yaw,
        predicted_pitch,
        predicted_distance,
    );
    let predicted_global = GlobalTransform::from(predicted_transform);
    cursor_ground_anchor(
        camera,
        &predicted_global,
        cursor_position,
        rts_camera.target_focus.y,
        ray_cast,
        ground,
    )
}

fn cursor_ground_anchor(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_position: Vec2,
    fallback_plane_height: f32,
    ray_cast: &mut MeshRayCast<'_, '_>,
    ground: &Query<Entity, With<RtsCameraGround>>,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    if let Some(hit) = cast_ground_ray(ray, ray_cast, ground) {
        return Some(hit.point);
    }
    ray.plane_intersection_point(
        Vec3::new(0.0, fallback_plane_height, 0.0),
        InfinitePlane3d::new(Vec3::Y),
    )
}

fn cast_ground_ray(
    ray: Ray3d,
    ray_cast: &mut MeshRayCast<'_, '_>,
    ground: &Query<Entity, With<RtsCameraGround>>,
) -> Option<bevy::picking::mesh_picking::ray_cast::RayMeshHit> {
    let filter = |entity| ground.get(entity).is_ok();
    let settings = MeshRayCastSettings {
        filter: &filter,
        ..default()
    };
    ray_cast
        .cast_ray(ray, &settings)
        .first()
        .map(|(_, hit)| hit.clone())
}

fn apply_focus_delta(
    camera: &mut RtsCamera,
    settings: &RtsCameraSettings,
    follow: Option<&mut RtsCameraFollow>,
    delta_xz: Vec2,
) {
    if delta_xz.length_squared() <= f32::EPSILON {
        return;
    }

    let delta = Vec3::new(delta_xz.x, 0.0, delta_xz.y);
    camera.target_focus += delta;

    if let Some(follow) = follow
        && follow.enabled
        && settings.controls.follow
    {
        follow.offset += delta;
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
