use bevy::{
    app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*, window::PrimaryWindow,
};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Action, Axial, Bidirectional, Binding, Bindings, Cardinal, EnhancedInputPlugin,
    EnhancedInputSystems, InputAction, Scale, TriggerState, actions, bindings,
};
use bevy_enhanced_input::preset::WithBundle;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraDebug, RtsCameraFallbackControls, RtsCameraGround, RtsCameraInput,
    RtsCameraInputTarget, RtsCameraSettings, RtsCameraSystems,
};

pub const DEFAULT_FOCUS: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const DEFAULT_EYE: Vec3 = Vec3::new(-18.0, 18.0, 18.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainStyle {
    Flat,
    Uneven,
}

#[derive(Component)]
pub struct ExampleOverlay;

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleInputSystems {
    WriteIntent,
}

#[derive(Component, Default)]
pub struct ExampleRtsCameraContext;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct ExamplePanAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct ExampleRotateAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct ExampleZoomAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ExampleDragPanAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ExampleDragRotateAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct ExamplePointerDeltaAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ExampleZoomToCursorAction;

pub struct ExampleRtsCameraControlsPlugin;

impl Plugin for ExampleRtsCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }

        app.add_input_context_to::<Update, ExampleRtsCameraContext>()
            .configure_sets(
                Update,
                ExampleInputSystems::WriteIntent
                    .after(EnhancedInputSystems::Apply)
                    .before(RtsCameraSystems::ResolveTarget),
            )
            .add_systems(
                Update,
                write_enhanced_input.in_set(ExampleInputSystems::WriteIntent),
            );
    }
}

pub fn apply_example_defaults(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));

    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn spawn_reference_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    title: &str,
    instructions: &str,
    accent: Color,
    terrain_style: TerrainStyle,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.58, 0.60, 0.68),
        brightness: 120.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        Name::new("Reference Sun"),
        DirectionalLight {
            illuminance: 38_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.88, 0.78, 0.0)),
    ));

    commands.spawn((
        Name::new("Reference Fill"),
        PointLight {
            intensity: 90_000.0,
            range: 70.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8.0, 10.0, 12.0),
    ));

    spawn_ground(commands, meshes, materials, terrain_style);
    spawn_landmarks(commands, meshes, materials, accent);

    commands.spawn((
        Name::new("Reference Overlay"),
        ExampleOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(430.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.80)),
        Text::new(format!("{title}\n{instructions}")),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_rts_camera(
    commands: &mut Commands,
    name: &str,
    camera: RtsCamera,
    settings: RtsCameraSettings,
    fallback_controls: Option<RtsCameraFallbackControls>,
    debug: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        camera,
        settings,
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            ..default()
        }),
        RtsCameraInputTarget,
    ));

    if let Some(fallback_controls) = fallback_controls {
        entity.insert(fallback_controls);
    }
    if debug {
        entity.insert(RtsCameraDebug::default());
    }

    entity.id()
}

pub fn attach_enhanced_input(commands: &mut Commands, camera_entity: Entity) {
    commands.entity(camera_entity).insert((
        ExampleRtsCameraContext,
        actions!(ExampleRtsCameraContext[
            (
                Action::<ExamplePanAction>::new(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<ExampleRotateAction>::new(),
                Bindings::spawn((
                    Bidirectional::new(KeyCode::KeyE, KeyCode::KeyQ),
                    Bidirectional::new(GamepadButton::DPadRight, GamepadButton::DPadLeft),
                )),
            ),
            (
                Action::<ExampleZoomAction>::new(),
                Bindings::spawn((Spawn((Binding::mouse_wheel(), Scale::splat(1.0))),)),
            ),
            (
                Action::<ExampleDragPanAction>::new(),
                bindings![MouseButton::Right, GamepadButton::LeftTrigger2],
            ),
            (
                Action::<ExampleDragRotateAction>::new(),
                bindings![MouseButton::Middle, GamepadButton::RightTrigger2],
            ),
            (
                Action::<ExamplePointerDeltaAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::splat(1.0))),
                    Axial::right_stick().with(Scale::splat(10.0)),
                )),
            ),
            (
                Action::<ExampleZoomToCursorAction>::new(),
                bindings![KeyCode::AltLeft, GamepadButton::South],
            ),
        ]),
    ));
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    terrain_style: TerrainStyle,
) {
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

    if terrain_style == TerrainStyle::Uneven {
        commands.spawn((
            Name::new("Ground South Ridge"),
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
            Name::new("Ground West Ramp"),
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
            Name::new("Ground Center Rise"),
            Mesh3d(meshes.add(Cuboid::new(10.0, 0.28, 6.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.26, 0.30, 0.22),
                perceptual_roughness: 0.95,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.95, 0.0).with_rotation(Quat::from_rotation_x(0.14)),
            RtsCameraGround,
        ));
        commands.spawn((
            Name::new("Ground Plateau"),
            Mesh3d(meshes.add(Cuboid::new(8.0, 0.32, 8.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.30, 0.34, 0.24),
                perceptual_roughness: 0.92,
                ..default()
            })),
            Transform::from_xyz(14.0, 3.5, 10.0),
            RtsCameraGround,
        ));
    }
}

fn spawn_landmarks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    accent: Color,
) {
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

    for (index, (x, z, height, color)) in [
        (-12.0, -12.0, 2.8, Color::srgb(0.42, 0.46, 0.54)),
        (12.0, -12.0, 4.8, Color::srgb(0.32, 0.39, 0.48)),
        (-12.0, 12.0, 3.6, Color::srgb(0.52, 0.42, 0.30)),
        (12.0, 12.0, 2.3, Color::srgb(0.34, 0.44, 0.34)),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Landmark Tower {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(1.1, height, 1.1))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x, height * 0.5, z),
        ));
    }
}

fn write_enhanced_input(
    pan: Single<&Action<ExamplePanAction>>,
    rotate: Single<&Action<ExampleRotateAction>>,
    zoom: Single<&Action<ExampleZoomAction>>,
    pointer_delta: Single<&Action<ExamplePointerDeltaAction>>,
    drag_pan_state: Single<&TriggerState, With<Action<ExampleDragPanAction>>>,
    drag_rotate_state: Single<&TriggerState, With<Action<ExampleDragRotateAction>>>,
    zoom_to_cursor_state: Single<&TriggerState, With<Action<ExampleZoomToCursorAction>>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_info: Single<(&Camera, &RtsCameraSettings), With<RtsCamera>>,
    mut camera_input: Single<&mut RtsCameraInput, With<RtsCamera>>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };
    let (camera, settings) = *camera_info;

    camera_input.pan += ***pan;
    if settings.controls.edge_pan {
        camera_input.edge_pan += edge_pan_delta(
            camera,
            window,
            window.cursor_position(),
            settings.edge_pan.margin,
        );
    }
    camera_input.rotate += ***rotate;
    camera_input.zoom += zoom.y;
    camera_input.cursor_position = window.cursor_position();
    camera_input.drag_pan_active = action_active(*drag_pan_state);
    camera_input.drag_rotate_active = action_active(*drag_rotate_state);
    camera_input.zoom_to_cursor = action_active(*zoom_to_cursor_state);

    if camera_input.drag_rotate_active {
        camera_input.rotate_drag_delta += pointer_delta.x;
    }
}

fn edge_pan_delta(
    camera: &Camera,
    window: &Window,
    cursor_position: Option<Vec2>,
    margin: f32,
) -> Vec2 {
    let Some(cursor_position) = cursor_position else {
        return Vec2::ZERO;
    };
    if margin <= 0.0 {
        return Vec2::ZERO;
    }

    let (origin, size) = camera
        .logical_viewport_rect()
        .map(|rect| (rect.min, rect.size()))
        .unwrap_or((Vec2::ZERO, window.size()));
    let local = cursor_position - origin;
    if local.x < 0.0 || local.y < 0.0 || local.x > size.x || local.y > size.y {
        return Vec2::ZERO;
    }

    let mut delta = Vec2::ZERO;
    if local.x <= margin {
        delta.x -= 1.0;
    }
    if local.x >= size.x - margin {
        delta.x += 1.0;
    }
    if local.y <= margin {
        delta.y += 1.0;
    }
    if local.y >= size.y - margin {
        delta.y -= 1.0;
    }
    delta
}

fn action_active(state: &TriggerState) -> bool {
    matches!(state, TriggerState::Ongoing | TriggerState::Fired)
}

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
