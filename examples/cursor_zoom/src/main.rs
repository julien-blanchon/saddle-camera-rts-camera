use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraAnchorSettings, RtsCameraFallbackControls, RtsCameraFallbackInputPlugin,
    RtsCameraPlugin, RtsCameraSettings, RtsCameraZoomAnchorMode,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera cursor_zoom".into(),
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
        "rts_camera cursor_zoom",
        "Focus zoom by default, cursor-preserving zoom on demand.\n\nControls:\nWASD / screen edge - Pan  |  Q / E - Rotate\nWheel - Zoom toward focus  |  Alt + wheel - Zoom toward cursor\nRMB drag - Drag pan  |  MMB drag - Drag rotate\n\nDebug gizmos show the active zoom anchor while the pane flips the zoom mode live.",
        Color::srgb(0.34, 0.66, 0.94),
        common::TerrainStyle::Uneven,
    );

    let camera = RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-16.0, 18.0, 16.0));
    let settings = RtsCameraSettings {
        anchors: RtsCameraAnchorSettings {
            zoom_anchor: RtsCameraZoomAnchorMode::Focus,
            ..default()
        },
        ..default()
    };
    let controls = RtsCameraFallbackControls {
        zoom_to_cursor: false,
        zoom_to_cursor_modifier: Some(KeyCode::AltLeft),
        ..default()
    };
    common::spawn_rts_camera(
        &mut commands,
        "Cursor Zoom Camera",
        camera.clone(),
        settings.clone(),
        Some(controls),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, false, true),
    );
}
