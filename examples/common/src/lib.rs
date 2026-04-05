use bevy::{
    app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*, window::PrimaryWindow,
};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Action, Axial, Bidirectional, Binding, Bindings, Cardinal, EnhancedInputPlugin,
    EnhancedInputSystems, InputAction, Scale, TriggerState, actions, bindings,
};
use bevy_enhanced_input::preset::WithBundle;
use bevy_flair::prelude::InlineStyle;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraDebug, RtsCameraFallbackControls, RtsCameraFollow, RtsCameraGround,
    RtsCameraInput, RtsCameraInputTarget, RtsCameraRuntime, RtsCameraSettings, RtsCameraSystems,
    RtsCameraZoomAnchorMode,
};
use saddle_pane::prelude::*;

const PANE_DARK_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#28292e"),
    ("--pane-elevation-2", "#222327"),
    ("--pane-elevation-3", "rgba(187, 188, 196, 0.10)"),
    ("--pane-border", "#3c3d44"),
    ("--pane-border-focus", "#7090b0"),
    ("--pane-border-subtle", "#333438"),
    ("--pane-text-primary", "#bbbcc4"),
    ("--pane-text-secondary", "#78797f"),
    ("--pane-text-muted", "#5c5d64"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#d0d1d8"),
    ("--pane-text-monitor", "#9a9ba2"),
    ("--pane-text-log", "#8a8b92"),
    ("--pane-accent", "#4a6fa5"),
    ("--pane-accent-hover", "#5a8fd5"),
    ("--pane-accent-active", "#3a5f95"),
    ("--pane-accent-subtle", "rgba(74, 111, 165, 0.15)"),
    ("--pane-accent-fill", "rgba(74, 111, 165, 0.60)"),
    ("--pane-accent-fill-hover", "rgba(90, 143, 213, 0.70)"),
    ("--pane-accent-fill-active", "rgba(90, 143, 213, 0.80)"),
    ("--pane-accent-checked", "rgba(74, 111, 165, 0.25)"),
    ("--pane-accent-checked-hover", "rgba(74, 111, 165, 0.35)"),
    ("--pane-accent-indicator", "rgba(74, 111, 165, 0.80)"),
    ("--pane-accent-knob", "#7aacdf"),
    ("--pane-widget-bg", "rgba(187, 188, 196, 0.10)"),
    ("--pane-widget-hover", "rgba(187, 188, 196, 0.15)"),
    ("--pane-widget-focus", "rgba(187, 188, 196, 0.20)"),
    ("--pane-widget-active", "rgba(187, 188, 196, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(187, 188, 196, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(187, 188, 196, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#1e1f24"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.25)"),
];

pub const DEFAULT_FOCUS: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const DEFAULT_EYE: Vec3 = Vec3::new(-18.0, 18.0, 18.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainStyle {
    Flat,
    Uneven,
}

#[derive(Component)]
pub struct ExampleOverlay;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Pane)]
#[pane(title = "RTS Camera", position = "top-right")]
pub struct ExampleRtsPane {
    #[pane(toggle)]
    pub cursor_zoom: bool,
    #[pane(toggle)]
    pub follow_enabled: bool,
    #[pane(toggle)]
    pub debug_gizmos: bool,
    #[pane(slider, min = 4.0, max = 36.0, step = 0.5)]
    pub distance: f32,
    #[pane(slider, min = 2.0, max = 28.0, step = 0.5)]
    pub min_distance: f32,
    #[pane(slider, min = 8.0, max = 80.0, step = 0.5)]
    pub max_distance: f32,
    #[pane(slider, min = 2.0, max = 30.0, step = 0.25)]
    pub pan_speed_near: f32,
    #[pane(slider, min = 4.0, max = 64.0, step = 0.25)]
    pub pan_speed_far: f32,
    #[pane(slider, min = 0.2, max = 8.0, step = 0.05)]
    pub zoom_speed: f32,
    #[pane(slider, min = 0.2, max = 4.0, step = 0.05)]
    pub rotation_speed: f32,
    #[pane(slider, min = 0.0, max = 64.0, step = 1.0)]
    pub edge_margin: f32,
    #[pane(slider, min = 1.0, max = 40.0, step = 0.5)]
    pub focus_decay: f32,
    #[pane(slider, min = 1.0, max = 40.0, step = 0.5)]
    pub ground_decay: f32,
    #[pane(toggle)]
    pub collision_enabled: bool,
    #[pane(slider, min = 0.0, max = 5.0, step = 0.1)]
    pub collision_clearance: f32,
    #[pane(monitor)]
    pub runtime_focus_x: f32,
    #[pane(monitor)]
    pub runtime_focus_z: f32,
    #[pane(monitor)]
    pub runtime_distance: f32,
}

impl Default for ExampleRtsPane {
    fn default() -> Self {
        Self {
            cursor_zoom: true,
            follow_enabled: true,
            debug_gizmos: false,
            distance: 18.0,
            min_distance: 8.0,
            max_distance: 44.0,
            pan_speed_near: 10.0,
            pan_speed_far: 34.0,
            zoom_speed: 2.8,
            rotation_speed: 1.9,
            edge_margin: 18.0,
            focus_decay: 18.0,
            ground_decay: 14.0,
            collision_enabled: true,
            collision_clearance: 1.5,
            runtime_focus_x: 0.0,
            runtime_focus_z: 0.0,
            runtime_distance: 18.0,
        }
    }
}

impl ExampleRtsPane {
    pub fn from_setup(
        camera: &RtsCamera,
        settings: &RtsCameraSettings,
        follow_enabled: bool,
        debug_gizmos: bool,
    ) -> Self {
        Self {
            cursor_zoom: settings.anchors.zoom_anchor == RtsCameraZoomAnchorMode::Cursor,
            follow_enabled,
            debug_gizmos,
            distance: camera.target_distance,
            min_distance: settings.distance.min,
            max_distance: settings.distance.max,
            pan_speed_near: settings.motion.pan_speed_near,
            pan_speed_far: settings.motion.pan_speed_far,
            zoom_speed: settings.motion.zoom_speed,
            rotation_speed: settings.motion.rotation_speed,
            edge_margin: settings.edge_pan.margin,
            focus_decay: settings.motion.focus_decay,
            ground_decay: settings.motion.ground_decay,
            collision_enabled: settings.collision.enabled,
            collision_clearance: settings.collision.clearance,
            runtime_focus_x: camera.target_focus.x,
            runtime_focus_z: camera.target_focus.z,
            runtime_distance: camera.target_distance,
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct ExampleRtsPaneBootstrap(ExampleRtsPane);

pub fn queue_example_pane(commands: &mut Commands, pane: ExampleRtsPane) {
    commands.insert_resource(ExampleRtsPaneBootstrap(pane));
}

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

pub fn install_pane(app: &mut App) {
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<ExampleRtsPane>()
    .add_systems(
        PreUpdate,
        (
            prime_pane_theme_vars,
            apply_bootstrapped_pane,
            sync_example_pane,
        )
            .chain(),
    )
    .add_systems(
        Update,
        reflect_runtime_into_pane.after(RtsCameraSystems::AdvanceRuntime),
    );
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExampleRtsPaneBootstrap>>,
    mut pane: ResMut<ExampleRtsPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExampleRtsPane::default() {
        *pane = bootstrap.0;
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

fn sync_example_pane(
    mut pane: ResMut<ExampleRtsPane>,
    bootstrap: Option<Res<ExampleRtsPaneBootstrap>>,
    mut commands: Commands,
    mut cameras: Query<(
        Entity,
        &mut RtsCamera,
        &mut RtsCameraSettings,
        Option<&mut RtsCameraFallbackControls>,
        Option<&mut RtsCameraFollow>,
        Option<&RtsCameraDebug>,
    )>,
) {
    let has_bootstrap = bootstrap.is_some();
    if let Some(bootstrap) = bootstrap
        && *pane == ExampleRtsPane::default()
        && bootstrap.0 != *pane
    {
        *pane = bootstrap.0;
    }

    for (entity, mut camera, mut settings, fallback_controls, follow, debug) in &mut cameras {
        let scene_pane = ExampleRtsPane::from_setup(
            &camera,
            &settings,
            follow.as_ref().is_some_and(|follow| follow.enabled),
            debug.is_some(),
        );
        if !has_bootstrap && *pane == ExampleRtsPane::default() && scene_pane != *pane {
            *pane = scene_pane;
            return;
        }

        let min_distance = pane.min_distance.max(1.0);
        let max_distance = pane.max_distance.max(min_distance + 0.5);

        settings.distance.min = min_distance;
        settings.distance.max = max_distance;
        settings.motion.pan_speed_near = pane.pan_speed_near.max(0.0);
        settings.motion.pan_speed_far = pane.pan_speed_far.max(settings.motion.pan_speed_near);
        settings.motion.zoom_speed = pane.zoom_speed.max(0.0);
        settings.motion.rotation_speed = pane.rotation_speed.max(0.0);
        settings.edge_pan.margin = pane.edge_margin.max(0.0);
        settings.motion.focus_decay = pane.focus_decay.max(0.0);
        settings.motion.ground_decay = pane.ground_decay.max(0.0);
        settings.collision.enabled = pane.collision_enabled;
        settings.collision.clearance = pane.collision_clearance.max(0.0);
        settings.anchors.zoom_anchor = if pane.cursor_zoom {
            RtsCameraZoomAnchorMode::Cursor
        } else {
            RtsCameraZoomAnchorMode::Focus
        };

        camera.target_distance = pane.distance.clamp(min_distance, max_distance);

        if let Some(mut fallback_controls) = fallback_controls {
            fallback_controls.zoom_to_cursor = pane.cursor_zoom;
        }

        if let Some(mut follow) = follow {
            follow.enabled = pane.follow_enabled;
        }

        match (pane.debug_gizmos, debug.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(RtsCameraDebug::default());
            }
            (false, true) => {
                commands.entity(entity).remove::<RtsCameraDebug>();
            }
            _ => {}
        }
    }
}

fn reflect_runtime_into_pane(
    cameras: Query<(&RtsCamera, &RtsCameraRuntime, Option<&RtsCameraFollow>)>,
    mut pane: ResMut<ExampleRtsPane>,
) {
    let Some((camera, runtime, follow)) = cameras.iter().next() else {
        return;
    };

    if let Some(follow) = follow {
        pane.follow_enabled = follow.enabled;
    }
    pane.distance = camera.target_distance;
    pane.runtime_focus_x = runtime.focus.x;
    pane.runtime_focus_z = runtime.focus.z;
    pane.runtime_distance = runtime.distance;
}
