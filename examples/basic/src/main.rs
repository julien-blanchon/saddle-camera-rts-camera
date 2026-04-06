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
                title: "rts_camera basic".into(),
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
        "rts_camera basic",
        "Explicit raw RTS fallback mappings.\n\nControls:\nW / A / S / D - Pan  |  Q / E - Rotate\nMouse wheel - Zoom  |  Alt + wheel - Zoom to cursor\nRMB drag - Drag pan  |  MMB drag - Drag rotate\nScreen edge - Edge pan\n\nThe live pane exposes the camera tuning surface while the mapping stays explicit in code.",
        Color::srgb(0.90, 0.58, 0.24),
        common::TerrainStyle::Flat,
    );

    let camera = RtsCamera::looking_at(common::DEFAULT_FOCUS, common::DEFAULT_EYE);
    let settings = RtsCameraSettings {
        anchors: RtsCameraAnchorSettings {
            zoom_anchor: RtsCameraZoomAnchorMode::Focus,
            ..default()
        },
        ..default()
    };
    let controls = RtsCameraFallbackControls {
        pan_up: KeyCode::KeyW,
        pan_down: KeyCode::KeyS,
        pan_left: KeyCode::KeyA,
        pan_right: KeyCode::KeyD,
        rotate_left: KeyCode::KeyQ,
        rotate_right: KeyCode::KeyE,
        drag_pan_button: MouseButton::Right,
        rotate_drag_button: MouseButton::Middle,
        zoom_to_cursor: false,
        zoom_to_cursor_modifier: Some(KeyCode::AltLeft),
        enabled: true,
    };
    common::spawn_rts_camera(
        &mut commands,
        "Basic RTS Camera",
        camera.clone(),
        settings.clone(),
        Some(controls),
        false,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, false, false),
    );
}
