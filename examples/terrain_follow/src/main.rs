use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{RtsCamera, RtsCameraFallbackControls, RtsCameraPlugin, RtsCameraSettings};

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
    ));
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
        "Move across the ramps and plateau.\nThe runtime keeps focus height above the marked ground meshes and draws debug gizmos.",
        Color::srgb(0.32, 0.74, 0.54),
        common::TerrainStyle::Uneven,
    );

    common::spawn_rts_camera(
        &mut commands,
        "Terrain Follow Camera",
        RtsCamera::looking_at(Vec3::new(8.0, 0.0, -8.0), Vec3::new(-18.0, 20.0, 18.0)),
        RtsCameraSettings::default(),
        Some(RtsCameraFallbackControls::default()),
        true,
    );
}
