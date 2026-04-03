use bevy::prelude::*;

use crate::RtsCameraBookmark;

#[derive(Clone, Debug, Message)]
pub struct RtsCameraBookmarkStored {
    pub camera: Entity,
    pub slot: usize,
    pub bookmark: RtsCameraBookmark,
}

#[derive(Clone, Debug, Message)]
pub struct RtsCameraBookmarkRecalled {
    pub camera: Entity,
    pub slot: usize,
    pub bookmark: RtsCameraBookmark,
}

#[derive(Clone, Debug, Message)]
pub struct RtsCameraFlyToApplied {
    pub camera: Entity,
    pub focus: Vec3,
    pub yaw: f32,
    pub distance: f32,
    pub snap: bool,
}
