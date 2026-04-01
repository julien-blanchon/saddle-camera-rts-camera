use bevy::{app::PostStartup, ecs::schedule::ScheduleLabel, prelude::*, time::TimeUpdateStrategy};

use crate::{RtsCamera, RtsCameraPlugin, RtsCameraRuntime, RtsCameraSettings, RtsCameraSystems};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterRuntime;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

fn push_runtime_marker(mut log: ResMut<OrderLog>) {
    log.0.push("runtime");
}

fn push_after_marker(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

fn spawn_camera(app: &mut App) {
    app.world_mut().spawn((
        RtsCamera::looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::new(-10.0, 10.0, 12.0)),
        RtsCameraSettings::default(),
    ));
}

fn init_test_assets(app: &mut App) {
    app.insert_resource(Assets::<Mesh>::default());
}

fn start_runtime(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(PostStartup);
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_ordering_points() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(RtsCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            RtsCameraSystems::AdvanceRuntime.before(AfterRuntime),
        )
        .add_systems(
            SimulationSchedule,
            (
                push_runtime_marker.in_set(RtsCameraSystems::AdvanceRuntime),
                push_after_marker.in_set(AfterRuntime),
            ),
        );
    init_test_assets(&mut app);

    spawn_camera(&mut app);
    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["runtime", "after"]
    );
}

#[test]
fn always_on_constructor_activates_runtime_after_startup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    spawn_camera(&mut app);
    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<(&RtsCameraRuntime, &Transform)>();
    let (runtime, transform) = query.single(app.world()).expect("camera runtime exists");
    assert!(runtime.distance > 0.0);
    assert!(transform.translation.length() > 0.0);
}

#[test]
fn deactivate_schedule_stops_runtime_updates() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .add_plugins(RtsCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ));
    init_test_assets(&mut app);

    spawn_camera(&mut app);
    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<RtsCamera>>()
        .single(app.world())
        .expect("camera exists");

    {
        let mut camera = app
            .world_mut()
            .get_mut::<RtsCamera>(entity)
            .expect("camera exists");
        camera.snap_to(Vec3::new(12.0, 0.0, -6.0), 0.8, 10.0);
    }

    app.world_mut().run_schedule(DeactivateSchedule);
    let before = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .cloned()
        .expect("runtime exists");
    app.world_mut().run_schedule(SimulationSchedule);
    let after = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .cloned()
        .expect("runtime exists");

    assert_eq!(before.focus, after.focus);
    assert_eq!(before.yaw, after.yaw);
    assert_eq!(before.distance, after.distance);
}
