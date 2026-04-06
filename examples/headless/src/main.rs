use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraInput, RtsCameraInputTarget, RtsCameraPlugin, RtsCameraSettings,
    RtsCameraSystems,
};

#[derive(Resource, Clone, Copy)]
struct HeadlessCamera(Entity);

#[derive(Clone, Copy)]
struct HeadlessWaypoint {
    focus: Vec3,
    yaw: f32,
    distance: f32,
}

#[derive(Resource)]
struct HeadlessFlightScript {
    timer: Timer,
    next_waypoint: usize,
    waypoints: Vec<HeadlessWaypoint>,
}

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera headless".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        drive_headless_camera.before(RtsCameraSystems::ResolveTarget),
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
        "rts_camera headless",
        "No raw keyboard, mouse, or action adapter is installed.\n\nThis camera only responds to programmatic fly-to intents written into RtsCameraInput.\nThe script hops between authored viewpoints every few seconds while the pane exposes the live runtime and tuning values.",
        Color::srgb(0.74, 0.64, 0.26),
        common::TerrainStyle::Uneven,
    );

    let waypoints = vec![
        HeadlessWaypoint {
            focus: Vec3::new(-14.0, 0.0, -10.0),
            yaw: 0.35,
            distance: 16.0,
        },
        HeadlessWaypoint {
            focus: Vec3::new(14.0, 0.0, -8.0),
            yaw: -0.85,
            distance: 13.0,
        },
        HeadlessWaypoint {
            focus: Vec3::new(4.0, 0.0, 16.0),
            yaw: 2.55,
            distance: 18.0,
        },
    ];

    for (index, waypoint) in waypoints.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Headless Waypoint {}", index + 1)),
            Mesh3d(meshes.add(Cylinder::new(1.2, 1.4).mesh().resolution(24))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.88 - index as f32 * 0.14, 0.62, 0.24 + index as f32 * 0.1),
                metallic: 0.08,
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_translation(waypoint.focus + Vec3::Y * 0.7),
        ));
    }

    let first = waypoints[0];
    let camera = RtsCamera::looking_at(first.focus, first.focus + Vec3::new(-18.0, 18.0, 18.0));
    let settings = RtsCameraSettings::default();
    let camera_entity = common::spawn_rts_camera(
        &mut commands,
        "Headless Camera",
        camera.clone(),
        settings.clone(),
        None,
        true,
    );
    commands.entity(camera_entity).remove::<RtsCameraInputTarget>();

    commands.insert_resource(HeadlessCamera(camera_entity));
    commands.insert_resource(HeadlessFlightScript {
        timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        next_waypoint: 1,
        waypoints,
    });
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, false, true),
    );
}

fn drive_headless_camera(
    time: Res<Time>,
    camera_entity: Res<HeadlessCamera>,
    mut script: ResMut<HeadlessFlightScript>,
    mut inputs: Query<&mut RtsCameraInput>,
) {
    if !script.timer.tick(time.delta()).just_finished() {
        return;
    }

    let waypoint = script.waypoints[script.next_waypoint];
    script.next_waypoint = (script.next_waypoint + 1) % script.waypoints.len();

    let Ok(mut input) = inputs.get_mut(camera_entity.0) else {
        return;
    };
    input.fly_to_focus = Some(waypoint.focus);
    input.fly_to_yaw = Some(waypoint.yaw);
    input.fly_to_distance = Some(waypoint.distance);
    input.fly_to_snap = false;
}
