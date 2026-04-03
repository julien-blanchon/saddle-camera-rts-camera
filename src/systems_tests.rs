use bevy::{prelude::*, time::TimeUpdateStrategy};

use crate::{
    RtsCamera, RtsCameraBookmarks, RtsCameraBounds, RtsCameraBoundsMode, RtsCameraPlugin,
    RtsCameraRuntime, RtsCameraSettings,
};

fn spawn_camera(app: &mut App, settings: RtsCameraSettings) -> Entity {
    app.world_mut()
        .spawn((
            RtsCamera::looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-16.0, 16.0, 16.0)),
            settings,
        ))
        .id()
}

fn init_test_assets(app: &mut App) {
    app.insert_resource(Assets::<Mesh>::default());
}

fn start(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.update();
}

#[test]
fn camera_initializes_runtime_without_invalid_values() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let entity = spawn_camera(&mut app, RtsCameraSettings::default());
    start(&mut app);

    let runtime = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .expect("runtime should exist");
    assert!(runtime.distance > 0.0);
    assert!(runtime.pitch.is_finite());
}

#[test]
fn bounds_clamp_programmatic_focus_changes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let settings = RtsCameraSettings {
        bounds: Some(RtsCameraBounds {
            min: Vec2::new(-2.0, -3.0),
            max: Vec2::new(2.0, 3.0),
            mode: RtsCameraBoundsMode::Hard,
            soft_margin: 1.0,
        }),
        ..default()
    };
    let entity = spawn_camera(&mut app, settings);
    start(&mut app);

    {
        let mut camera = app
            .world_mut()
            .get_mut::<RtsCamera>(entity)
            .expect("camera should exist");
        camera.target_focus = Vec3::new(12.0, 0.0, -12.0);
    }

    app.update();

    let camera = app.world().get::<RtsCamera>(entity).expect("camera exists");
    assert_eq!(camera.target_focus.x, 2.0);
    assert_eq!(camera.target_focus.z, -3.0);
}

#[test]
fn snap_applies_target_state_in_one_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let mut settings = RtsCameraSettings::default();
    settings.ground.enabled = false;
    let entity = spawn_camera(&mut app, settings);
    start(&mut app);

    {
        let mut camera = app
            .world_mut()
            .get_mut::<RtsCamera>(entity)
            .expect("camera should exist");
        camera.snap_to(Vec3::new(6.0, 2.0, -4.0), 1.2, 12.0);
    }

    app.update();

    let camera = app.world().get::<RtsCamera>(entity).expect("camera exists");
    let runtime = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .expect("runtime exists");

    assert!(!camera.snap);
    assert!((runtime.focus.x - 6.0).abs() < 0.001);
    assert!((runtime.focus.y - 2.0).abs() < 0.001);
    assert!((runtime.focus.z + 4.0).abs() < 0.001);
    assert!((runtime.yaw - 1.2).abs() < 0.001);
    assert!((runtime.distance - 12.0).abs() < 0.001);
}

#[test]
fn edge_pan_still_moves_when_general_pan_is_disabled() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let settings = RtsCameraSettings {
        controls: crate::RtsCameraControlFlags {
            pan: false,
            edge_pan: true,
            ..default()
        },
        ground: crate::RtsCameraGroundSettings {
            enabled: false,
            ..default()
        },
        ..default()
    };
    let entity = spawn_camera(&mut app, settings);
    start(&mut app);

    {
        let mut input = app
            .world_mut()
            .get_mut::<crate::RtsCameraInput>(entity)
            .expect("input should exist");
        input.edge_pan = Vec2::new(1.0, 0.0);
    }

    let before = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera should exist")
        .target_focus;
    app.update();
    let after = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera should exist")
        .target_focus;

    assert!(after.xz().distance(before.xz()) > 0.1);
}

#[test]
fn bookmarks_store_current_runtime_and_restore_target_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let mut settings = RtsCameraSettings::default();
    settings.ground.enabled = false;
    let entity = spawn_camera(&mut app, settings);
    start(&mut app);

    {
        let mut input = app
            .world_mut()
            .get_mut::<crate::RtsCameraInput>(entity)
            .expect("input should exist");
        input.set_bookmark_slot = Some(2);
    }
    app.update();

    {
        let mut camera = app
            .world_mut()
            .get_mut::<RtsCamera>(entity)
            .expect("camera should exist");
        camera.target_focus = Vec3::new(14.0, 0.0, -9.0);
        camera.target_yaw = 1.8;
        camera.target_distance = 30.0;
    }
    {
        let mut input = app
            .world_mut()
            .get_mut::<crate::RtsCameraInput>(entity)
            .expect("input should exist");
        input.recall_bookmark_slot = Some(2);
        input.recall_bookmark_snap = true;
    }
    app.update();

    let bookmarks = app
        .world()
        .get::<RtsCameraBookmarks>(entity)
        .expect("bookmarks should exist");
    let bookmark = bookmarks.get(2).expect("bookmark should be stored");
    let camera = app.world().get::<RtsCamera>(entity).expect("camera exists");
    let runtime = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .expect("runtime exists");

    assert_eq!(camera.target_focus, bookmark.focus);
    assert!((camera.target_yaw - bookmark.yaw).abs() < 0.001);
    assert!((camera.target_distance - bookmark.distance).abs() < 0.001);
    assert!(runtime.focus.distance(bookmark.focus) < 0.001);
    assert!((runtime.yaw - bookmark.yaw).abs() < 0.001);
    assert!((runtime.distance - bookmark.distance).abs() < 0.001);
}

#[test]
fn fly_to_focus_updates_target_state_without_forcing_a_snap() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);

    let mut settings = RtsCameraSettings::default();
    settings.ground.enabled = false;
    let entity = spawn_camera(&mut app, settings);
    start(&mut app);

    {
        let mut input = app
            .world_mut()
            .get_mut::<crate::RtsCameraInput>(entity)
            .expect("input should exist");
        input.fly_to_focus = Some(Vec3::new(10.0, 3.0, -6.0));
        input.fly_to_yaw = Some(0.65);
        input.fly_to_distance = Some(11.0);
    }

    app.update();

    let camera = app.world().get::<RtsCamera>(entity).expect("camera exists");
    let runtime = app
        .world()
        .get::<RtsCameraRuntime>(entity)
        .expect("runtime exists");

    assert_eq!(camera.target_focus, Vec3::new(10.0, 3.0, -6.0));
    assert!((camera.target_yaw - 0.65).abs() < 0.001);
    assert!((camera.target_distance - 11.0).abs() < 0.001);
    assert!(runtime.focus.distance(camera.target_focus) > 0.001);
}
