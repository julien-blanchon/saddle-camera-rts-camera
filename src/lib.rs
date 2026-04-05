mod components;
mod config;
mod input;
mod math;
mod messages;
mod systems;

pub use components::{
    RtsCamera, RtsCameraBookmark, RtsCameraBookmarks, RtsCameraDebug, RtsCameraFallbackControls,
    RtsCameraFollow, RtsCameraGround, RtsCameraInput, RtsCameraInputTarget, RtsCameraRuntime,
};
pub use config::{
    RtsCameraAnchorSettings, RtsCameraBounds, RtsCameraBoundsMode, RtsCameraCollisionSettings,
    RtsCameraControlFlags, RtsCameraDistanceSettings, RtsCameraEdgePanSettings,
    RtsCameraGroundSettings, RtsCameraMotionSettings, RtsCameraPitchSettings,
    RtsCameraRotationPivotMode, RtsCameraSettings, RtsCameraZoomAnchorMode,
};
pub use math::{
    camera_pitch_for_distance, camera_transform_from_state, clamp_distance, pan_vector_from_yaw,
    resolve_ground_height_target, shortest_angle_delta, smooth_angle, smooth_scalar,
    smoothing_factor, soft_clamp_axis_delta, wrap_angle,
};
pub use messages::{RtsCameraBookmarkRecalled, RtsCameraBookmarkStored, RtsCameraFlyToApplied};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    gizmos::{config::DefaultGizmoConfigGroup, gizmos::GizmoStorage},
    prelude::*,
    transform::TransformSystems,
};

use crate::components::RtsCameraInternalState;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RtsCameraSystems {
    ReadInput,
    ResolveTarget,
    FollowGround,
    ApplyBounds,
    AdvanceRuntime,
    ResolveCollision,
    SyncTransform,
    Debug,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct RtsCameraRuntimeActive(bool);

pub struct RtsCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl RtsCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for RtsCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for RtsCameraPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<RtsCameraRuntimeActive>()
            .register_type::<RtsCamera>()
            .register_type::<RtsCameraAnchorSettings>()
            .register_type::<RtsCameraBounds>()
            .register_type::<RtsCameraBoundsMode>()
            .register_type::<RtsCameraBookmark>()
            .register_type::<RtsCameraBookmarks>()
            .register_type::<RtsCameraControlFlags>()
            .register_type::<RtsCameraDebug>()
            .register_type::<RtsCameraDistanceSettings>()
            .register_type::<RtsCameraEdgePanSettings>()
            .register_type::<RtsCameraFallbackControls>()
            .register_type::<RtsCameraFollow>()
            .register_type::<RtsCameraGround>()
            .register_type::<RtsCameraGroundSettings>()
            .register_type::<RtsCameraInput>()
            .register_type::<RtsCameraInputTarget>()
            .register_type::<RtsCameraMotionSettings>()
            .register_type::<RtsCameraPitchSettings>()
            .register_type::<RtsCameraRotationPivotMode>()
            .register_type::<RtsCameraRuntime>()
            .register_type::<RtsCameraSettings>()
            .register_type::<RtsCameraCollisionSettings>()
            .register_type::<RtsCameraZoomAnchorMode>()
            .add_message::<RtsCameraBookmarkStored>()
            .add_message::<RtsCameraBookmarkRecalled>()
            .add_message::<RtsCameraFlyToApplied>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .add_systems(self.activate_schedule, systems::initialize_added_cameras)
            .configure_sets(
                self.update_schedule,
                (
                    RtsCameraSystems::ReadInput,
                    RtsCameraSystems::ResolveTarget,
                    RtsCameraSystems::FollowGround,
                    RtsCameraSystems::ApplyBounds,
                    RtsCameraSystems::AdvanceRuntime,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                input::apply_fallback_controls
                    .in_set(RtsCameraSystems::ReadInput)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::initialize_added_cameras,
                    systems::apply_programmatic_commands,
                    systems::sync_follow_targets,
                    systems::apply_camera_input,
                )
                    .chain()
                    .in_set(RtsCameraSystems::ResolveTarget)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::resolve_ground_height
                    .in_set(RtsCameraSystems::FollowGround)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::apply_bounds
                    .in_set(RtsCameraSystems::ApplyBounds)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (systems::advance_runtime, systems::clear_consumed_input)
                    .chain()
                    .in_set(RtsCameraSystems::AdvanceRuntime)
                    .run_if(runtime_is_active),
            )
            .configure_sets(
                PostUpdate,
                (
                    RtsCameraSystems::ResolveCollision,
                    RtsCameraSystems::SyncTransform,
                    RtsCameraSystems::Debug,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                systems::resolve_camera_collision
                    .in_set(RtsCameraSystems::ResolveCollision)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                systems::sync_transform
                    .in_set(RtsCameraSystems::SyncTransform)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                systems::draw_debug_gizmos
                    .in_set(RtsCameraSystems::Debug)
                    .after(RtsCameraSystems::SyncTransform)
                    .run_if(resource_exists::<GizmoStorage<DefaultGizmoConfigGroup, ()>>)
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<RtsCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<RtsCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<RtsCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
