use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraFallbackInputPlugin, RtsCameraGround,
    RtsCameraPlugin, RtsCameraSettings,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera terrain_follow".into(),
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
        "rts_camera terrain_follow",
        "Terrain-follow and collision on uneven ground.\n\nControls:\nWASD / screen edge - Pan  |  Q / E - Rotate  |  Wheel - Zoom\nRMB drag - Drag pan  |  MMB drag - Drag rotate\n\nPan across the ramps, plateau, and cliff wall. The pane lets you tune smoothing and collision live.",
        Color::srgb(0.32, 0.74, 0.54),
        common::TerrainStyle::Uneven,
    );

    commands.spawn((
        Name::new("Cliff Wall"),
        Mesh3d(meshes.add(Cuboid::new(6.0, 6.0, 12.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.28, 0.22),
            perceptual_roughness: 0.98,
            ..default()
        })),
        Transform::from_xyz(-6.0, 3.0, -4.0),
        RtsCameraGround,
    ));

    let camera = RtsCamera::looking_at(Vec3::new(8.0, 0.0, -8.0), Vec3::new(-18.0, 20.0, 18.0));
    let settings = RtsCameraSettings::default();
    common::spawn_rts_camera(
        &mut commands,
        "Terrain Follow Camera",
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
