use bevy::prelude::*;

use crate::{
    RtsCameraBounds, RtsCameraBoundsMode, RtsCameraDistanceSettings, RtsCameraGroundSettings,
    RtsCameraPitchSettings, camera_pitch_for_distance, clamp_distance, pan_vector_from_yaw,
    resolve_ground_height_target, shortest_angle_delta, smooth_scalar, smoothing_factor,
    soft_clamp_axis_delta, wrap_angle,
};

#[test]
fn clamp_distance_respects_min_and_max() {
    let settings = RtsCameraDistanceSettings {
        min: 8.0,
        max: 24.0,
    };
    assert_eq!(clamp_distance(4.0, &settings), 8.0);
    assert_eq!(clamp_distance(42.0, &settings), 24.0);
    assert_eq!(clamp_distance(16.0, &settings), 16.0);
}

#[test]
fn pan_direction_rotates_with_yaw() {
    let pan = pan_vector_from_yaw(std::f32::consts::FRAC_PI_2, Vec2::Y);
    assert!(pan.x < -0.99);
    assert!(pan.z.abs() < 0.01);
}

#[test]
fn dynamic_pitch_interpolates_across_distance_range() {
    let distance = RtsCameraDistanceSettings {
        min: 10.0,
        max: 30.0,
    };
    let pitch = RtsCameraPitchSettings {
        near_angle: 60.0_f32.to_radians(),
        far_angle: 30.0_f32.to_radians(),
    };
    let mid = camera_pitch_for_distance(20.0, &distance, &pitch);
    assert!((mid - 45.0_f32.to_radians()).abs() < 0.001);
}

#[test]
fn angle_helpers_take_shortest_path() {
    let current = 179.0_f32.to_radians();
    let target = -179.0_f32.to_radians();
    let delta = shortest_angle_delta(current, target);
    assert!(delta.abs() < 5.0_f32.to_radians());
    assert!((wrap_angle(current + delta) - target).abs() < 0.01);
}

#[test]
fn soft_clamp_slows_near_edge() {
    let slowed = soft_clamp_axis_delta(9.5, 4.0, -10.0, 10.0, 2.0);
    assert!(slowed > 0.0);
    assert!(slowed < 4.0);
}

#[test]
fn hard_bounds_default_shape_is_symmetric() {
    let bounds = RtsCameraBounds::default();
    assert_eq!(bounds.mode, RtsCameraBoundsMode::Hard);
    assert_eq!(bounds.min, Vec2::splat(-24.0));
    assert_eq!(bounds.max, Vec2::splat(24.0));
}

#[test]
fn zero_decay_snaps_immediately() {
    assert_eq!(smoothing_factor(0.0, 1.0 / 60.0), 1.0);
    assert_eq!(smooth_scalar(2.0, 8.0, 0.0, 1.0 / 60.0), 8.0);
}

#[test]
fn ground_height_resolution_prefers_hit_then_last_height() {
    let settings = RtsCameraGroundSettings {
        enabled: true,
        clearance: 1.5,
        probe_height: 64.0,
        keep_last_height_on_miss: true,
    };

    assert_eq!(
        resolve_ground_height_target(Some(4.0), Some(6.0), &settings, 0.0),
        Some(7.5)
    );
    assert_eq!(
        resolve_ground_height_target(Some(4.0), None, &settings, 0.0),
        Some(4.0)
    );
    assert_eq!(
        resolve_ground_height_target(None, None, &settings, 2.0),
        Some(3.5)
    );
}

#[test]
fn ground_height_resolution_can_fall_back_to_focus_height() {
    let settings = RtsCameraGroundSettings {
        enabled: true,
        clearance: 1.0,
        probe_height: 64.0,
        keep_last_height_on_miss: false,
    };

    assert_eq!(
        resolve_ground_height_target(Some(4.0), None, &settings, 2.5),
        Some(2.5)
    );
}
