use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraAnchorSettings, RtsCameraPlugin, RtsCameraSettings, RtsCameraZoomAnchorMode,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera enhanced_input".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
        common::ExampleRtsCameraControlsPlugin,
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
        "rts_camera enhanced_input",
        "Production input path using bevy_enhanced_input.\nWASD pan, Q/E rotate, RMB drag pan, MMB drag rotate, wheel zoom, Alt toggles cursor zoom, edge pan remains viewport based.",
        Color::srgb(0.28, 0.70, 0.88),
        common::TerrainStyle::Uneven,
    );

    let settings = RtsCameraSettings {
        anchors: RtsCameraAnchorSettings {
            zoom_anchor: RtsCameraZoomAnchorMode::Focus,
            ..default()
        },
        ..default()
    };
    let camera = RtsCamera::looking_at(common::DEFAULT_FOCUS, common::DEFAULT_EYE);

    let camera_entity = common::spawn_rts_camera(
        &mut commands,
        "Enhanced Input Camera",
        camera.clone(),
        settings.clone(),
        None,
        true,
    );
    common::attach_enhanced_input(&mut commands, camera_entity);
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, true, true),
    );
}
