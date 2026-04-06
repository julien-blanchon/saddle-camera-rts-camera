use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraBounds, RtsCameraBoundsMode, RtsCameraFallbackControls,
    RtsCameraFallbackInputPlugin, RtsCameraPlugin, RtsCameraSettings,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera bounds".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
        RtsCameraFallbackInputPlugin::default(),
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
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "rts_camera bounds",
        "Soft focus bounds with visible debug gizmos.\n\nControls:\nWASD / screen edge - Pan  |  Q / E - Rotate  |  Wheel - Zoom\nRMB drag - Drag pan  |  MMB drag - Drag rotate\n\nThe pane lets you tune speeds and distances while the red loop shows the legal focus area.",
        Color::srgb(0.94, 0.36, 0.28),
        common::TerrainStyle::Uneven,
    );

    let camera = RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-14.0, 16.0, 14.0));
    let settings = RtsCameraSettings {
        bounds: Some(RtsCameraBounds {
            min: Vec2::new(-10.0, -8.0),
            max: Vec2::new(10.0, 8.0),
            mode: RtsCameraBoundsMode::Soft,
            soft_margin: 3.5,
        }),
        ..default()
    };
    common::spawn_rts_camera(
        &mut commands,
        "Bounds Camera",
        camera.clone(),
        settings.clone(),
        Some(RtsCameraFallbackControls::default()),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, false, true),
    );
}
