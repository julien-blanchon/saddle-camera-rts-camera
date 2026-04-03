use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraPlugin, RtsCameraSettings,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera basic".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = RtsCamera::looking_at(common::DEFAULT_FOCUS, common::DEFAULT_EYE);
    let settings = RtsCameraSettings::default();

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "rts_camera basic",
        "Fallback raw input path.\nWASD or screen edge pan, Q/E rotate, RMB drag pan, MMB drag rotate, wheel zoom.",
        Color::srgb(0.90, 0.58, 0.24),
        common::TerrainStyle::Flat,
    );

    common::spawn_rts_camera(
        &mut commands,
        "Basic RTS Camera",
        camera.clone(),
        settings.clone(),
        Some(RtsCameraFallbackControls::default()),
        false,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, true, false),
    );
}
