//! Basic RTS camera example — fallback raw-input path.
//!
//! Demonstrates the simplest possible RTS camera setup using the built-in
//! `RtsCameraFallbackControls` (no `bevy_enhanced_input` dependency).
//! WASD or screen-edge pan, Q/E rotate, RMB drag-pan, MMB drag-rotate, wheel zoom.

use bevy::{app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*};
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraFallbackControls, RtsCameraGround, RtsCameraInputTarget, RtsCameraPlugin,
    RtsCameraSettings,
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
                title: "rts_camera basic".into(),
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

    // -- Ground (flat) --
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.14, 0.16),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RtsCameraGround,
    ));

    // -- Landmark pillars for spatial reference --
    let accent = Color::srgb(0.90, 0.58, 0.24);
    commands.spawn((
        Name::new("Accent Core"),
        Mesh3d(meshes.add(Sphere::new(1.2).mesh().ico(5).expect("icosphere"))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: accent,
            metallic: 0.05,
            perceptual_roughness: 0.26,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.2, 0.0),
    ));
    for (i, (x, z, h, color)) in [
        (-12.0, -12.0, 2.8, Color::srgb(0.42, 0.46, 0.54)),
        (12.0, -12.0, 4.8, Color::srgb(0.32, 0.39, 0.48)),
        (-12.0, 12.0, 3.6, Color::srgb(0.52, 0.42, 0.30)),
        (12.0, 12.0, 2.3, Color::srgb(0.34, 0.44, 0.34)),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Tower {}", i + 1)),
            Mesh3d(meshes.add(Cuboid::new(1.1, h, 1.1))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x, h * 0.5, z),
        ));
    }

    // -- Camera entity with all components visible --
    let focus = Vec3::ZERO;
    let eye = Vec3::new(-18.0, 18.0, 18.0);

    let camera = RtsCamera::looking_at(focus, eye);
    let settings = RtsCameraSettings::default();
    let fallback_controls = RtsCameraFallbackControls::default();

    commands.spawn((
        Name::new("Basic RTS Camera"),
        camera,
        settings,
        // Narrow FOV keeps the top-down perspective tight
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            ..default()
        }),
        // Mark this entity as the one receiving input
        RtsCameraInputTarget,
        // Built-in keyboard/mouse controls — no external input crate needed
        fallback_controls,
    ));

    // -- HUD overlay --
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
            "rts_camera basic\n\n\
             Controls:\n\
             WASD / screen edge  -  Pan\n\
             Q / E               -  Rotate\n\
             Mouse wheel          -  Zoom\n\
             RMB drag             -  Drag pan\n\
             MMB drag             -  Drag rotate\n\n\
             Fallback raw input path (no bevy_enhanced_input).",
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
