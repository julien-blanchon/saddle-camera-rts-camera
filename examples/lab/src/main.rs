use saddle_camera_rts_camera_example_common as common;
#[cfg(feature = "e2e")]
mod e2e;

use bevy::{
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraFollow, RtsCameraRuntime, RtsCameraSettings, RtsCameraSystems,
};

#[derive(Component)]
struct LabFollowTarget;

#[derive(Component)]
struct LabOverlay;

#[derive(Resource, Clone, Copy)]
pub struct LabCameraEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabTargetEntity(pub Entity);

#[derive(Resource, Clone, Copy, Default)]
pub struct LabTargetOverride(pub Option<Vec3>);

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera_lab".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        saddle_camera_rts_camera::RtsCameraPlugin::default(),
        common::ExampleRtsCameraControlsPlugin,
        RemotePlugin::default(),
    ));
    common::install_pane(&mut app);
    #[cfg(feature = "brp")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::RtsCameraLabE2EPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_target.before(RtsCameraSystems::ResolveTarget),
            update_overlay.after(RtsCameraSystems::AdvanceRuntime),
        ),
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
        "rts_camera_lab",
        "BEI path on an uneven scene.\nWASD pan, Q/E rotate, RMB drag pan, MMB drag rotate, wheel zoom, Alt cursor zoom.",
        Color::srgb(0.90, 0.58, 0.22),
        common::TerrainStyle::Uneven,
    );

    let target = commands
        .spawn((
            Name::new("Lab Follow Target"),
            LabFollowTarget,
            Mesh3d(meshes.add(Capsule3d::new(0.55, 1.8).mesh().rings(10).latitudes(14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.96, 0.28, 0.34),
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_xyz(8.0, 1.1, 0.0),
        ))
        .id();

    let settings = RtsCameraSettings {
        bounds: Some(saddle_camera_rts_camera::RtsCameraBounds {
            min: Vec2::new(-18.0, -18.0),
            max: Vec2::new(18.0, 18.0),
            mode: saddle_camera_rts_camera::RtsCameraBoundsMode::Soft,
            soft_margin: 4.0,
        }),
        ..default()
    };
    let camera = RtsCamera::looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-18.0, 18.0, 18.0));

    let camera_entity = common::spawn_rts_camera(
        &mut commands,
        "Lab RTS Camera",
        camera.clone(),
        settings.clone(),
        None,
        true,
    );
    common::attach_enhanced_input(&mut commands, camera_entity);
    commands.entity(camera_entity).insert(RtsCameraFollow {
        target,
        offset: Vec3::ZERO,
        enabled: false,
        snap: false,
    });

    commands.insert_resource(LabCameraEntity(camera_entity));
    commands.insert_resource(LabTargetEntity(target));
    commands.insert_resource(LabTargetOverride::default());
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, false, true),
    );

    commands.spawn((
        Name::new("Lab Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(440.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
        Text::new(String::new()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn animate_target(
    time: Res<Time>,
    target_override: Option<Res<LabTargetOverride>>,
    mut targets: Query<&mut Transform, With<LabFollowTarget>>,
) {
    let Ok(mut transform) = targets.single_mut() else {
        return;
    };

    if let Some(target_override) = target_override.and_then(|target_override| target_override.0) {
        transform.translation = target_override;
        return;
    }

    let t = time.elapsed_secs() * 0.55;
    transform.translation.x = 10.0 * t.cos();
    transform.translation.z = 8.0 * t.sin();
    transform.translation.y = 1.1;
}

fn update_overlay(
    camera_entity: Res<LabCameraEntity>,
    target_entity: Res<LabTargetEntity>,
    cameras: Query<(&RtsCamera, &RtsCameraRuntime, Option<&RtsCameraFollow>)>,
    targets: Query<&Transform, With<LabFollowTarget>>,
    mut overlays: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok((camera, runtime, follow)) = cameras.get(camera_entity.0) else {
        return;
    };
    let Ok(target_transform) = targets.get(target_entity.0) else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };

    *text = Text::new(format!(
        "RTS Camera Lab\nfocus {:.2?}\nyaw {:.2} distance {:.2} pitch {:.2}\nground {:?}\nanchor {:?}\ntarget focus {:.2?}\nfollow {:?}\ntracked entity {:.2?}",
        runtime.focus,
        runtime.yaw,
        runtime.distance,
        runtime.pitch,
        runtime.ground_height,
        runtime.last_cursor_anchor,
        camera.target_focus,
        follow.map(|follow| (follow.enabled, follow.offset)),
        target_transform.translation,
    ));
}
