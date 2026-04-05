//! RTS camera bounds example — soft and hard pan-limit boundaries.
//!
//! Shows how to restrict camera focus to a rectangular area using
//! `RtsCameraBounds`. The `Soft` mode applies a spring-like compression near
//! the boundary edges instead of an abrupt clamp. Debug gizmos visualise the
//! playable focus area as a red loop on the ground.

use bevy::{app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*};
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraBounds, RtsCameraBoundsMode, RtsCameraDebug, RtsCameraFallbackControls,
    RtsCameraGround, RtsCameraInputTarget, RtsCameraPlugin, RtsCameraSettings,
};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));

    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }

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
    ));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // -- Lights --
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.58, 0.60, 0.68),
        brightness: 120.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 38_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.88, 0.78, 0.0)),
    ));
    commands.spawn((
        Name::new("Fill"),
        PointLight {
            intensity: 90_000.0,
            range: 70.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8.0, 10.0, 12.0),
    ));

    // -- Ground (uneven terrain) --
    commands.spawn((
        Name::new("Ground Base"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.14, 0.16),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RtsCameraGround,
    ));
    commands.spawn((
        Name::new("South Ridge"),
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.35, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.28, 0.34, 0.24),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(10.0, 1.7, -10.0).with_rotation(Quat::from_rotation_x(-0.22)),
        RtsCameraGround,
    ));
    commands.spawn((
        Name::new("West Ramp"),
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.30, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.21, 0.28, 0.20),
            perceptual_roughness: 0.96,
            ..default()
        })),
        Transform::from_xyz(-11.0, 1.15, 9.0).with_rotation(Quat::from_rotation_z(0.24)),
        RtsCameraGround,
    ));
    commands.spawn((
        Name::new("Plateau"),
        Mesh3d(meshes.add(Cuboid::new(8.0, 0.32, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.34, 0.24),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(14.0, 3.5, 10.0),
        RtsCameraGround,
    ));

    // -- Landmark at origin for spatial reference --
    commands.spawn((
        Name::new("Accent Core"),
        Mesh3d(meshes.add(Sphere::new(1.2).mesh().ico(5).expect("icosphere"))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.94, 0.36, 0.28),
            metallic: 0.05,
            perceptual_roughness: 0.26,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.2, 0.0),
    ));

    // -- Camera with bounds configuration --
    let settings = RtsCameraSettings {
        // Restrict focus to a 20x16 rectangle centred on the origin.
        // Soft mode compresses motion near edges instead of hard-clamping.
        bounds: Some(RtsCameraBounds {
            min: Vec2::new(-10.0, -8.0),
            max: Vec2::new(10.0, 8.0),
            mode: RtsCameraBoundsMode::Soft,
            soft_margin: 3.5,
        }),
        ..default()
    };
    let camera = RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-14.0, 16.0, 14.0));

    commands.spawn((
        Name::new("Bounds Camera"),
        camera,
        settings,
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            ..default()
        }),
        RtsCameraInputTarget,
        RtsCameraFallbackControls::default(),
        // Debug gizmos show the bounds rectangle on the ground
        RtsCameraDebug::default(),
    ));

    // -- HUD --
    commands.spawn((
        Name::new("Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(430.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.80)),
        Text::new(
            "rts_camera bounds\n\n\
             Controls:\n\
             WASD / screen edge  -  Pan\n\
             Q / E               -  Rotate\n\
             Mouse wheel          -  Zoom\n\
             RMB drag             -  Drag pan\n\
             MMB drag             -  Drag rotate\n\n\
             Soft bounds compress motion near the map edge.\n\
             The red debug loop shows the playable focus area.",
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// ---------------------------------------------------------------------------
// Auto-exit (used in CI / lab runs)
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct AutoExitAfter(Timer);

fn auto_exit_from_env() -> Option<AutoExitAfter> {
    let seconds = std::env::var("RTS_CAMERA_AUTO_EXIT_SECONDS")
        .ok()?
        .parse::<f32>()
        .ok()?;
    Some(AutoExitAfter(Timer::from_seconds(
        seconds.max(0.1),
        TimerMode::Once,
    )))
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}
