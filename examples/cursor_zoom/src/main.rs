use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraAnchorSettings, RtsCameraFallbackControls, RtsCameraPlugin,
    RtsCameraSettings, RtsCameraZoomAnchorMode,
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
        "Wheel zooms toward focus by default.\nHold Alt while scrolling to preserve the cursor-ground anchor. Debug gizmos show the active anchor.",
        Color::srgb(0.34, 0.66, 0.94),
        common::TerrainStyle::Uneven,
    );

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
    let camera = RtsCamera::looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-16.0, 18.0, 16.0));

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
        common::ExampleRtsPane::from_setup(&camera, &settings, true, true),
    );
}
