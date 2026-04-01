use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraFollow, RtsCameraPlugin, RtsCameraSettings,
    RtsCameraSystems,
};

#[derive(Component)]
struct DemoFollowTarget;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera follow_target".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
    ));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        animate_target.before(RtsCameraSystems::ResolveTarget),
    );
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
        "rts_camera follow_target",
        "The camera keeps its yaw and zoom while tracking the moving target.\nManual pan adjusts follow offset instead of fighting the follow system.",
        Color::srgb(0.88, 0.28, 0.40),
        common::TerrainStyle::Uneven,
    );

    let target = commands
        .spawn((
            Name::new("Demo Follow Target"),
            DemoFollowTarget,
            Mesh3d(meshes.add(Capsule3d::new(0.55, 1.8).mesh().rings(8).latitudes(12))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.28, 0.32),
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_xyz(4.0, 1.1, 0.0),
        ))
        .id();

    let camera = common::spawn_rts_camera(
        &mut commands,
        "Follow Camera",
        RtsCamera::looking_at(Vec3::new(4.0, 0.0, 0.0), Vec3::new(-16.0, 16.0, 16.0)),
        RtsCameraSettings::default(),
        Some(RtsCameraFallbackControls::default()),
        true,
    );

    commands.entity(camera).insert(RtsCameraFollow {
        target,
        offset: Vec3::ZERO,
        enabled: true,
        snap: false,
    });
}

fn animate_target(time: Res<Time>, mut targets: Query<&mut Transform, With<DemoFollowTarget>>) {
    let Ok(mut transform) = targets.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.7;
    transform.translation.x = 8.0 * t.cos();
    transform.translation.z = 6.0 * t.sin();
    transform.translation.y = 1.1;
}
