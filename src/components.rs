use bevy::prelude::*;

use crate::config::RtsCameraSettings;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(
    Camera3d,
    Transform,
    RtsCameraSettings,
    RtsCameraRuntime,
    RtsCameraInput,
    RtsCameraBookmarks,
    RtsCameraInternalState
)]
pub struct RtsCamera {
    pub target_focus: Vec3,
    pub target_yaw: f32,
    pub target_distance: f32,
    pub snap: bool,
}

impl Default for RtsCamera {
    fn default() -> Self {
        Self {
            target_focus: Vec3::ZERO,
            target_yaw: 0.0,
            target_distance: 18.0,
            snap: false,
        }
    }
}

impl RtsCamera {
    pub fn looking_at(focus: Vec3, eye: Vec3) -> Self {
        let delta = eye - focus;
        let horizontal = Vec2::new(delta.x, delta.z);
        let target_distance = delta.length().max(0.01);
        let target_yaw = horizontal.x.atan2(-horizontal.y);
        Self {
            target_focus: focus,
            target_yaw,
            target_distance,
            snap: true,
        }
    }

    pub fn snap_to(&mut self, focus: Vec3, yaw: f32, distance: f32) {
        self.target_focus = focus;
        self.target_yaw = yaw;
        self.target_distance = distance.max(0.01);
        self.snap = true;
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct RtsCameraRuntime {
    pub focus: Vec3,
    pub yaw: f32,
    pub distance: f32,
    pub pitch: f32,
    pub ground_height: Option<f32>,
    pub last_ground_hit: Option<Vec3>,
    pub last_cursor_anchor: Option<Vec3>,
}

impl Default for RtsCameraRuntime {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            yaw: 0.0,
            distance: 18.0,
            pitch: 50.0_f32.to_radians(),
            ground_height: None,
            last_ground_hit: None,
            last_cursor_anchor: None,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct RtsCameraInput {
    pub pan: Vec2,
    pub edge_pan: Vec2,
    pub zoom: f32,
    pub rotate: f32,
    pub rotate_drag_delta: f32,
    pub drag_pan_active: bool,
    pub drag_rotate_active: bool,
    pub cursor_position: Option<Vec2>,
    pub zoom_to_cursor: bool,
    pub fly_to_focus: Option<Vec3>,
    pub fly_to_yaw: Option<f32>,
    pub fly_to_distance: Option<f32>,
    pub fly_to_snap: bool,
    pub set_bookmark_slot: Option<usize>,
    pub recall_bookmark_slot: Option<usize>,
    pub recall_bookmark_snap: bool,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub struct RtsCameraBookmark {
    pub focus: Vec3,
    pub yaw: f32,
    pub distance: f32,
}

impl RtsCameraBookmark {
    pub fn from_runtime(runtime: &RtsCameraRuntime) -> Self {
        Self {
            focus: runtime.focus,
            yaw: runtime.yaw,
            distance: runtime.distance,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct RtsCameraBookmarks {
    pub slots: Vec<Option<RtsCameraBookmark>>,
}

impl RtsCameraBookmarks {
    pub fn set(&mut self, slot: usize, bookmark: RtsCameraBookmark) {
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }
        self.slots[slot] = Some(bookmark);
    }

    pub fn get(&self, slot: usize) -> Option<RtsCameraBookmark> {
        self.slots.get(slot).copied().flatten()
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct RtsCameraFollow {
    pub target: Entity,
    pub offset: Vec3,
    pub enabled: bool,
    pub snap: bool,
}

impl Default for RtsCameraFollow {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            offset: Vec3::ZERO,
            enabled: true,
            snap: false,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct RtsCameraFallbackControls {
    pub pan_up: KeyCode,
    pub pan_down: KeyCode,
    pub pan_left: KeyCode,
    pub pan_right: KeyCode,
    pub rotate_left: KeyCode,
    pub rotate_right: KeyCode,
    pub drag_pan_button: MouseButton,
    pub rotate_drag_button: MouseButton,
    pub zoom_to_cursor: bool,
    pub zoom_to_cursor_modifier: Option<KeyCode>,
    pub enabled: bool,
}

impl Default for RtsCameraFallbackControls {
    fn default() -> Self {
        Self {
            pan_up: KeyCode::KeyW,
            pan_down: KeyCode::KeyS,
            pan_left: KeyCode::KeyA,
            pan_right: KeyCode::KeyD,
            rotate_left: KeyCode::KeyQ,
            rotate_right: KeyCode::KeyE,
            drag_pan_button: MouseButton::Right,
            rotate_drag_button: MouseButton::Middle,
            zoom_to_cursor: true,
            zoom_to_cursor_modifier: None,
            enabled: true,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct RtsCameraInputTarget;

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct RtsCameraGround;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct RtsCameraDebug {
    pub enabled: bool,
}

impl Default for RtsCameraDebug {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub(crate) struct RtsCameraInternalState {
    pub drag_anchor_world: Option<Vec3>,
}
