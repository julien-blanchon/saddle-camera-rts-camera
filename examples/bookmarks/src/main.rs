use saddle_camera_rts_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraBookmark, RtsCameraBookmarks, RtsCameraInput, RtsCameraPlugin,
    RtsCameraSettings, RtsCameraSystems,
};

#[derive(Resource, Clone, Copy)]
struct BookmarkCamera(Entity);

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rts_camera bookmarks".into(),
                resolution: (1600, 900).into(),
                ..default()
            }),
            ..default()
        }),
        RtsCameraPlugin::default(),
        common::ExampleRtsCameraControlsPlugin,
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        handle_bookmarks.before(RtsCameraSystems::ResolveTarget),
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
        "rts_camera bookmarks",
        "Battlefield command posts with RTS bookmark recall.\n\nControls:\nWASD / screen edge - Pan  |  Q/E - Rotate  |  Wheel - Zoom\nRMB drag - Drag pan  |  MMB drag - Drag rotate\n1 / 2 / 3 - Fly to stored bookmark\nCtrl + 1/2/3 - Overwrite bookmark with current view",
        Color::srgb(0.96, 0.60, 0.18),
        common::TerrainStyle::Uneven,
    );

    for (index, (translation, color)) in [
        (Vec3::new(-18.0, 0.75, -14.0), Color::srgb(0.89, 0.32, 0.28)),
        (Vec3::new(18.0, 1.20, -6.0), Color::srgb(0.24, 0.64, 0.88)),
        (Vec3::new(2.0, 2.80, 18.0), Color::srgb(0.26, 0.74, 0.52)),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Command Post {}", index + 1)),
            Mesh3d(meshes.add(Cylinder::new(1.5, 1.6).mesh().resolution(28))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.08,
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_translation(translation),
        ));

        commands.spawn((
            Name::new(format!("Beacon {}", index + 1)),
            Mesh3d(meshes.add(Sphere::new(0.55).mesh().ico(4).expect("icosphere"))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.with_alpha(0.92),
                emissive: color.into(),
                ..default()
            })),
            Transform::from_translation(translation + Vec3::Y * 2.4),
        ));
    }

    let camera = RtsCamera::looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-20.0, 20.0, 18.0));
    let settings = RtsCameraSettings::default();
    let camera_entity = common::spawn_rts_camera(
        &mut commands,
        "Bookmark Camera",
        camera.clone(),
        settings.clone(),
        None,
        true,
    );
    common::attach_enhanced_input(&mut commands, camera_entity);
    commands.entity(camera_entity).insert(RtsCameraBookmarks {
        slots: vec![
            Some(RtsCameraBookmark {
                focus: Vec3::new(-18.0, 0.0, -14.0),
                yaw: 0.35,
                distance: 17.0,
            }),
            Some(RtsCameraBookmark {
                focus: Vec3::new(18.0, 0.0, -6.0),
                yaw: -0.85,
                distance: 15.0,
            }),
            Some(RtsCameraBookmark {
                focus: Vec3::new(2.0, 0.0, 18.0),
                yaw: 2.75,
                distance: 14.0,
            }),
        ],
    });

    commands.insert_resource(BookmarkCamera(camera_entity));
    common::queue_example_pane(
        &mut commands,
        common::ExampleRtsPane::from_setup(&camera, &settings, true, true),
    );
}

fn handle_bookmarks(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Res<BookmarkCamera>,
    mut inputs: Query<&mut RtsCameraInput>,
) {
    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    for (key, slot) in [
        (KeyCode::Digit1, 0usize),
        (KeyCode::Digit2, 1usize),
        (KeyCode::Digit3, 2usize),
    ] {
        if !keys.just_pressed(key) {
            continue;
        }

        let Ok(mut input) = inputs.get_mut(camera.0) else {
            return;
        };

        if ctrl_pressed {
            input.set_bookmark_slot = Some(slot);
        } else {
            input.recall_bookmark_slot = Some(slot);
            input.recall_bookmark_snap = false;
        }
    }
}
