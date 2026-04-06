use bevy::{prelude::*, time::TimeUpdateStrategy, window::PrimaryWindow};

use crate::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraFallbackInputPlugin, RtsCameraInputTarget,
    RtsCameraPlugin, RtsCameraSettings,
};

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

fn spawn_primary_window(app: &mut App) {
    app.world_mut().spawn((Window::default(), PrimaryWindow));
}

fn spawn_fallback_camera(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Fallback Camera"),
            RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-16.0, 16.0, 16.0)),
            RtsCameraSettings {
                ground: crate::RtsCameraGroundSettings {
                    enabled: false,
                    ..default()
                },
                ..default()
            },
            RtsCameraInputTarget,
            RtsCameraFallbackControls::default(),
        ))
        .id()
}

fn press_key(app: &mut App, key: KeyCode) {
    if !app.world().contains_resource::<ButtonInput<KeyCode>>() {
        app.insert_resource(ButtonInput::<KeyCode>::default());
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

#[test]
fn fallback_controls_do_not_run_without_the_adapter_plugin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RtsCameraPlugin::always_on(Update));
    init_test_assets(&mut app);
    spawn_primary_window(&mut app);
    let entity = spawn_fallback_camera(&mut app);
    start(&mut app);

    let before = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera exists")
        .target_focus;
    press_key(&mut app, KeyCode::KeyW);
    app.update();
    let after = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera exists")
        .target_focus;

    assert_eq!(after, before);
}

#[test]
fn fallback_controls_move_the_camera_when_the_adapter_plugin_is_added() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins((
        RtsCameraPlugin::always_on(Update),
        RtsCameraFallbackInputPlugin::default(),
    ));
    init_test_assets(&mut app);
    spawn_primary_window(&mut app);
    let entity = spawn_fallback_camera(&mut app);
    start(&mut app);

    let before = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera exists")
        .target_focus;
    press_key(&mut app, KeyCode::KeyW);
    app.update();
    let after = app
        .world()
        .get::<RtsCamera>(entity)
        .expect("camera exists")
        .target_focus;

    assert!(after.distance(before) > 0.1);
}
