use bevy::{math::StableInterpolate, prelude::*};

use crate::{
    RtsCameraBounds, RtsCameraBoundsMode, RtsCameraDistanceSettings, RtsCameraGroundSettings,
    RtsCameraPitchSettings,
};

pub fn clamp_distance(distance: f32, settings: &RtsCameraDistanceSettings) -> f32 {
    distance.clamp(
        settings.min.min(settings.max),
        settings.max.max(settings.min),
    )
}

pub fn normalized_distance(distance: f32, settings: &RtsCameraDistanceSettings) -> f32 {
    let span = (settings.max - settings.min).max(f32::EPSILON);
    ((distance - settings.min) / span).clamp(0.0, 1.0)
}

pub fn camera_pitch_for_distance(
    distance: f32,
    distance_settings: &RtsCameraDistanceSettings,
    pitch: &RtsCameraPitchSettings,
) -> f32 {
    let t = normalized_distance(distance, distance_settings);
    pitch.near_angle.lerp(pitch.far_angle, t)
}

pub fn smoothing_factor(decay: f32, dt: f32) -> f32 {
    if decay <= 0.0 || dt <= 0.0 {
        return 1.0;
    }

    1.0 - (-decay * dt).exp()
}

pub fn smooth_scalar(current: f32, target: f32, decay: f32, dt: f32) -> f32 {
    current.interpolate_stable(&target, smoothing_factor(decay, dt))
}

pub fn pan_vector_from_yaw(yaw: f32, local_pan: Vec2) -> Vec3 {
    let local = Vec3::new(local_pan.x, 0.0, -local_pan.y);
    Quat::from_rotation_y(yaw) * local
}

pub fn camera_transform_from_state(focus: Vec3, yaw: f32, pitch: f32, distance: f32) -> Transform {
    let rotation = Quat::from_euler(EulerRot::YXZ, yaw, -pitch, 0.0);
    let forward = rotation * Vec3::NEG_Z;
    let position = focus - forward * distance;
    Transform::from_translation(position).looking_at(focus, Vec3::Y)
}

pub fn wrap_angle(angle: f32) -> f32 {
    let mut wrapped = angle % std::f32::consts::TAU;
    if wrapped <= -std::f32::consts::PI {
        wrapped += std::f32::consts::TAU;
    } else if wrapped > std::f32::consts::PI {
        wrapped -= std::f32::consts::TAU;
    }
    wrapped
}

pub fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    wrap_angle(target - current)
}

pub fn smooth_angle(current: f32, target: f32, decay: f32, dt: f32) -> f32 {
    let delta = shortest_angle_delta(current, target);
    let smoothed = 0.0_f32.interpolate_stable(&delta, smoothing_factor(decay, dt));
    wrap_angle(current + smoothed)
}

pub fn resolve_ground_height_target(
    current_height: Option<f32>,
    hit_y: Option<f32>,
    settings: &RtsCameraGroundSettings,
    fallback_y: f32,
) -> Option<f32> {
    if !settings.enabled {
        return Some(fallback_y);
    }

    if let Some(hit_y) = hit_y {
        return Some(hit_y + settings.clearance);
    }

    if settings.keep_last_height_on_miss {
        current_height.or(Some(fallback_y + settings.clearance))
    } else {
        Some(fallback_y)
    }
}

pub fn soft_clamp_axis_delta(position: f32, delta: f32, min: f32, max: f32, margin: f32) -> f32 {
    if delta.abs() <= f32::EPSILON || margin <= 0.0 {
        return delta;
    }

    if delta > 0.0 {
        let remaining = max - position;
        if remaining <= 0.0 {
            return 0.0;
        }
        if remaining < margin {
            return delta * (remaining / margin).clamp(0.0, 1.0);
        }
    } else {
        let remaining = position - min;
        if remaining <= 0.0 {
            return 0.0;
        }
        if remaining < margin {
            return delta * (remaining / margin).clamp(0.0, 1.0);
        }
    }

    delta
}

pub fn apply_bounds_delta(position: Vec2, delta: Vec2, bounds: &RtsCameraBounds) -> Vec2 {
    match bounds.mode {
        RtsCameraBoundsMode::Hard => delta,
        RtsCameraBoundsMode::Soft => Vec2::new(
            soft_clamp_axis_delta(
                position.x,
                delta.x,
                bounds.min.x,
                bounds.max.x,
                bounds.soft_margin,
            ),
            soft_clamp_axis_delta(
                position.y,
                delta.y,
                bounds.min.y,
                bounds.max.y,
                bounds.soft_margin,
            ),
        ),
    }
}

pub fn clamp_focus_to_bounds(focus: Vec3, bounds: &RtsCameraBounds) -> Vec3 {
    Vec3::new(
        focus.x.clamp(bounds.min.x, bounds.max.x),
        focus.y,
        focus.z.clamp(bounds.min.y, bounds.max.y),
    )
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
