use bevy::prelude::*;

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct RtsCameraSettings {
    pub distance: RtsCameraDistanceSettings,
    pub pitch: RtsCameraPitchSettings,
    pub motion: RtsCameraMotionSettings,
    pub ground: RtsCameraGroundSettings,
    pub bounds: Option<RtsCameraBounds>,
    pub anchors: RtsCameraAnchorSettings,
    pub controls: RtsCameraControlFlags,
    pub edge_pan: RtsCameraEdgePanSettings,
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraDistanceSettings {
    pub min: f32,
    pub max: f32,
}

impl Default for RtsCameraDistanceSettings {
    fn default() -> Self {
        Self {
            min: 8.0,
            max: 44.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraPitchSettings {
    pub near_angle: f32,
    pub far_angle: f32,
}

impl Default for RtsCameraPitchSettings {
    fn default() -> Self {
        Self {
            near_angle: 58.0_f32.to_radians(),
            far_angle: 32.0_f32.to_radians(),
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraMotionSettings {
    pub pan_speed_near: f32,
    pub pan_speed_far: f32,
    pub zoom_speed: f32,
    pub rotation_speed: f32,
    pub drag_rotation_speed: f32,
    pub focus_decay: f32,
    pub ground_decay: f32,
    pub yaw_decay: f32,
    pub distance_decay: f32,
}

impl Default for RtsCameraMotionSettings {
    fn default() -> Self {
        Self {
            pan_speed_near: 10.0,
            pan_speed_far: 34.0,
            zoom_speed: 2.8,
            rotation_speed: 1.9,
            drag_rotation_speed: 0.009,
            focus_decay: 18.0,
            ground_decay: 10.0,
            yaw_decay: 18.0,
            distance_decay: 16.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraGroundSettings {
    pub enabled: bool,
    pub clearance: f32,
    pub probe_height: f32,
    pub keep_last_height_on_miss: bool,
}

impl Default for RtsCameraGroundSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            clearance: 1.2,
            probe_height: 256.0,
            keep_last_height_on_miss: true,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraEdgePanSettings {
    pub margin: f32,
}

impl Default for RtsCameraEdgePanSettings {
    fn default() -> Self {
        Self { margin: 18.0 }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraBounds {
    pub min: Vec2,
    pub max: Vec2,
    pub mode: RtsCameraBoundsMode,
    pub soft_margin: f32,
}

impl Default for RtsCameraBounds {
    fn default() -> Self {
        Self {
            min: Vec2::splat(-24.0),
            max: Vec2::splat(24.0),
            mode: RtsCameraBoundsMode::Hard,
            soft_margin: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
pub enum RtsCameraBoundsMode {
    Hard,
    Soft,
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraAnchorSettings {
    pub zoom_anchor: RtsCameraZoomAnchorMode,
    pub rotation_pivot: RtsCameraRotationPivotMode,
}

impl Default for RtsCameraAnchorSettings {
    fn default() -> Self {
        Self {
            zoom_anchor: RtsCameraZoomAnchorMode::Cursor,
            rotation_pivot: RtsCameraRotationPivotMode::Focus,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
pub enum RtsCameraZoomAnchorMode {
    Focus,
    Cursor,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
pub enum RtsCameraRotationPivotMode {
    Focus,
    Cursor,
}

#[derive(Clone, Debug, Reflect)]
pub struct RtsCameraControlFlags {
    pub pan: bool,
    pub edge_pan: bool,
    pub drag_pan: bool,
    pub zoom: bool,
    pub rotation: bool,
    pub follow: bool,
}

impl Default for RtsCameraControlFlags {
    fn default() -> Self {
        Self {
            pan: true,
            edge_pan: true,
            drag_pan: true,
            zoom: true,
            rotation: true,
            follow: true,
        }
    }
}
