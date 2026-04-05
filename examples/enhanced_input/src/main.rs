//! RTS camera with `bevy_enhanced_input` — production input path.
//!
//! Instead of the built-in `RtsCameraFallbackControls`, this example wires
//! `bevy_enhanced_input` actions directly into `RtsCameraInput`. This gives
//! full control over bindings, composites, modifiers, and trigger states.
//!
//! Controls: WASD pan, Q/E rotate, RMB drag-pan, MMB drag-rotate, wheel zoom,
//! Alt toggles cursor zoom, edge-pan from viewport borders.

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
    RtsCamera, RtsCameraAnchorSettings, RtsCameraDebug, RtsCameraGround, RtsCameraInput,
    RtsCameraInputTarget, RtsCameraPlugin, RtsCameraSettings, RtsCameraSystems,
    RtsCameraZoomAnchorMode,
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
                title: "rts_camera enhanced_input".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
        EnhancedInputPlugin,
    ));

    // Register our input context and wire the write system between
    // enhanced_input's Apply phase and the camera's ResolveTarget phase.
    app.add_input_context_to::<Update, RtsCameraInputContext>()
        .configure_sets(
            Update,
            InputWriteSet
                .after(EnhancedInputSystems::Apply)
                .before(RtsCameraSystems::ResolveTarget),
        )
        .add_systems(Update, write_enhanced_input.in_set(InputWriteSet));

    app.add_systems(Startup, setup);
    app.run();
}

// ---------------------------------------------------------------------------
// Input action declarations
// ---------------------------------------------------------------------------

#[derive(Component, Default)]
struct RtsCameraInputContext;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct InputWriteSet;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct PanAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
struct RotateAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct ZoomAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct DragPanAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct DragRotateAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct PointerDeltaAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ZoomToCursorAction;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

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

    // -- Uneven ground --
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
        Name::new("Center Rise"),
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
            base_color: Color::srgb(0.28, 0.70, 0.88),
            metallic: 0.05,
            perceptual_roughness: 0.26,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.2, 0.0),
    ));

    // -- Camera with enhanced-input bindings --
    let settings = RtsCameraSettings {
        anchors: RtsCameraAnchorSettings {
            zoom_anchor: RtsCameraZoomAnchorMode::Focus,
            ..default()
        },
        ..default()
    };
    let camera = RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-18.0, 18.0, 18.0));

    commands.spawn((
        Name::new("Enhanced Input Camera"),
        camera,
        settings,
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            ..default()
        }),
        RtsCameraInputTarget,
        RtsCameraDebug::default(),
        // -- bevy_enhanced_input context + action bindings --
        RtsCameraInputContext,
        actions!(RtsCameraInputContext[
            (
                Action::<PanAction>::new(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<RotateAction>::new(),
                Bindings::spawn((
                    Bidirectional::new(KeyCode::KeyE, KeyCode::KeyQ),
                    Bidirectional::new(GamepadButton::DPadRight, GamepadButton::DPadLeft),
                )),
            ),
            (
                Action::<ZoomAction>::new(),
                Bindings::spawn((Spawn((Binding::mouse_wheel(), Scale::splat(1.0))),)),
            ),
            (
                Action::<DragPanAction>::new(),
                bindings![MouseButton::Right, GamepadButton::LeftTrigger2],
            ),
            (
                Action::<DragRotateAction>::new(),
                bindings![MouseButton::Middle, GamepadButton::RightTrigger2],
            ),
            (
                Action::<PointerDeltaAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::splat(1.0))),
                    Axial::right_stick().with(Scale::splat(10.0)),
                )),
            ),
            (
                Action::<ZoomToCursorAction>::new(),
                bindings![KeyCode::AltLeft, GamepadButton::South],
            ),
        ]),
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
            "rts_camera enhanced_input\n\n\
             Controls (bevy_enhanced_input production path):\n\
             WASD / left stick     -  Pan\n\
             Q / E / DPad          -  Rotate\n\
             Mouse wheel            -  Zoom\n\
             Alt + wheel             -  Cursor zoom\n\
             RMB / LT               -  Drag pan\n\
             MMB / RT               -  Drag rotate\n\
             Screen edge             -  Edge pan",
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// ---------------------------------------------------------------------------
// System: map enhanced-input actions into RtsCameraInput each frame
// ---------------------------------------------------------------------------

fn write_enhanced_input(
    pan: Single<&Action<PanAction>>,
    rotate: Single<&Action<RotateAction>>,
    zoom: Single<&Action<ZoomAction>>,
    pointer_delta: Single<&Action<PointerDeltaAction>>,
    drag_pan_state: Single<&TriggerState, With<Action<DragPanAction>>>,
    drag_rotate_state: Single<&TriggerState, With<Action<DragRotateAction>>>,
    zoom_to_cursor_state: Single<&TriggerState, With<Action<ZoomToCursorAction>>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_info: Single<(&Camera, &RtsCameraSettings), With<RtsCamera>>,
    mut camera_input: Single<&mut RtsCameraInput, With<RtsCamera>>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };
    let (camera, settings) = *camera_info;

    camera_input.pan += ***pan;

    // Edge-pan: accelerate toward whichever screen border the cursor is near
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

    let active = |s: &TriggerState| matches!(s, TriggerState::Ongoing | TriggerState::Fired);
    camera_input.drag_pan_active = active(*drag_pan_state);
    camera_input.drag_rotate_active = active(*drag_rotate_state);
    camera_input.zoom_to_cursor = active(*zoom_to_cursor_state);

    if camera_input.drag_rotate_active {
        camera_input.rotate_drag_delta += pointer_delta.x;
    }
}

/// Compute a normalised pan vector from cursor proximity to viewport edges.
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

// ---------------------------------------------------------------------------
// Auto-exit
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
