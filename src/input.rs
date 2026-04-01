use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraInput, RtsCameraInputTarget, RtsCameraSettings,
};

pub(crate) fn apply_fallback_controls(
    key_input: Option<Res<ButtonInput<KeyCode>>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse_motion: Option<Res<AccumulatedMouseMotion>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut cameras: ParamSet<(
        Query<
            (
                Entity,
                &Camera,
                &RtsCameraFallbackControls,
                &RtsCameraSettings,
            ),
            (With<RtsCamera>, With<RtsCameraInputTarget>),
        >,
        Query<
            (
                &Camera,
                &RtsCameraFallbackControls,
                &RtsCameraSettings,
                &mut RtsCameraInput,
            ),
            (With<RtsCamera>, With<RtsCameraInputTarget>),
        >,
    )>,
) {
    let Some(entity) = ({
        let selection = cameras.p0();
        select_input_target(&selection)
    }) else {
        return;
    };

    let Ok(window) = primary_window.single() else {
        return;
    };

    let mut inputs = cameras.p1();
    let Ok((camera, controls, settings, mut input)) = inputs.get_mut(entity) else {
        return;
    };
    if !controls.enabled {
        return;
    }

    let keys = key_input.as_deref();
    let buttons = mouse_buttons.as_deref();
    let cursor_position = window.cursor_position();
    let motion = mouse_motion.map_or(Vec2::ZERO, |value| value.delta);
    let scroll = mouse_scroll.map_or(Vec2::ZERO, |value| value.delta);

    let mut pan = Vec2::ZERO;
    if settings.controls.pan {
        if keys.is_some_and(|keys| keys.pressed(controls.pan_up)) {
            pan.y += 1.0;
        }
        if keys.is_some_and(|keys| keys.pressed(controls.pan_down)) {
            pan.y -= 1.0;
        }
        if keys.is_some_and(|keys| keys.pressed(controls.pan_right)) {
            pan.x += 1.0;
        }
        if keys.is_some_and(|keys| keys.pressed(controls.pan_left)) {
            pan.x -= 1.0;
        }
    }

    let mut edge_pan = Vec2::ZERO;
    if settings.controls.edge_pan {
        edge_pan = edge_pan_delta(camera, window, cursor_position, settings.edge_pan.margin);
    }

    if settings.controls.rotation {
        if keys.is_some_and(|keys| keys.pressed(controls.rotate_left)) {
            input.rotate += 1.0;
        }
        if keys.is_some_and(|keys| keys.pressed(controls.rotate_right)) {
            input.rotate -= 1.0;
        }
    }

    input.pan += pan.normalize_or_zero();
    input.edge_pan += edge_pan.normalize_or_zero();
    input.cursor_position = cursor_position;
    input.drag_pan_active = settings.controls.drag_pan
        && buttons.is_some_and(|buttons| buttons.pressed(controls.drag_pan_button));
    input.drag_rotate_active = settings.controls.rotation
        && buttons.is_some_and(|buttons| buttons.pressed(controls.rotate_drag_button));

    if input.drag_rotate_active {
        input.rotate_drag_delta += motion.x;
    }

    if settings.controls.zoom {
        input.zoom += scroll.y;
    }

    input.zoom_to_cursor = if let Some(modifier) = controls.zoom_to_cursor_modifier {
        keys.is_some_and(|keys| keys.pressed(modifier))
    } else {
        controls.zoom_to_cursor
    };
}

fn edge_pan_delta(
    camera: &Camera,
    window: &Window,
    cursor_position: Option<Vec2>,
    margin: f32,
) -> Vec2 {
    let Some(cursor_position) = cursor_position else {
        return Vec2::ZERO;
    };
    if margin <= 0.0 {
        return Vec2::ZERO;
    }

    let (origin, size) = camera
        .logical_viewport_rect()
        .map(|rect| (rect.min, rect.size()))
        .unwrap_or((Vec2::ZERO, window.size()));
    let local = cursor_position - origin;
    if local.x < 0.0 || local.y < 0.0 || local.x > size.x || local.y > size.y {
        return Vec2::ZERO;
    }

    let mut delta = Vec2::ZERO;
    if local.x <= margin {
        delta.x -= 1.0;
    }
    if local.x >= size.x - margin {
        delta.x += 1.0;
    }
    if local.y <= margin {
        delta.y += 1.0;
    }
    if local.y >= size.y - margin {
        delta.y -= 1.0;
    }
    delta
}

fn select_input_target(
    query: &Query<
        (
            Entity,
            &Camera,
            &RtsCameraFallbackControls,
            &RtsCameraSettings,
        ),
        (With<RtsCamera>, With<RtsCameraInputTarget>),
    >,
) -> Option<Entity> {
    query
        .iter()
        .filter(|(_, camera, controls, settings)| {
            camera.is_active
                && controls.enabled
                && (settings.controls.pan || settings.controls.zoom || settings.controls.rotation)
        })
        .max_by_key(|(entity, camera, _, _)| (camera.order, entity.to_bits()))
        .map(|(entity, _, _, _)| entity)
}
