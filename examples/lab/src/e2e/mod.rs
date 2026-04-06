use bevy::{math::DVec2, prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::EnhancedInputSystems;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use saddle_camera_rts_camera::{RtsCamera, RtsCameraFollow, RtsCameraInput, RtsCameraRuntime};

use crate::{LabCameraEntity, LabTargetEntity};

pub struct RtsCameraLabE2EPlugin;

impl Plugin for RtsCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(
            Update,
            (
                E2ESet.before(EnhancedInputSystems::Update),
                E2ESet.before(saddle_camera_rts_camera::RtsCameraSystems::ReadInput),
            ),
        );

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[rts_camera_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct RuntimeBaseline {
    focus: Vec3,
    yaw: f32,
    distance: f32,
    last_cursor_anchor: Option<Vec3>,
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(build_smoke_launch()),
        "rts_camera_smoke" => Some(build_smoke()),
        "rts_camera_controls" => Some(build_controls()),
        "rts_camera_pointer_controls" => Some(build_pointer_controls()),
        "rts_camera_follow_target" => Some(build_follow_target()),
        "rts_camera_bookmarks" => Some(build_bookmarks()),
        "rts_camera_headless_intents" => Some(build_headless_intents()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "rts_camera_smoke",
        "rts_camera_controls",
        "rts_camera_pointer_controls",
        "rts_camera_follow_target",
        "rts_camera_bookmarks",
        "rts_camera_headless_intents",
    ]
}

fn camera_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabCameraEntity>()
        .map(|resource| resource.0)
}

fn target_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabTargetEntity>()
        .map(|resource| resource.0)
}

fn runtime(world: &World) -> Option<RtsCameraRuntime> {
    let entity = camera_entity(world)?;
    world.get::<RtsCameraRuntime>(entity).cloned()
}

fn store_runtime_baseline(world: &mut World) {
    if let Some(runtime) = runtime(world) {
        world.insert_resource(RuntimeBaseline {
            focus: runtime.focus,
            yaw: runtime.yaw,
            distance: runtime.distance,
            last_cursor_anchor: runtime.last_cursor_anchor,
        });
    }
}

fn set_cursor_position(world: &mut World, logical_position: Vec2) {
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let Ok(mut window) = windows.single_mut(world) else {
        return;
    };
    let scale = f64::from(window.scale_factor());
    window.set_physical_cursor_position(Some(DVec2::new(
        logical_position.x as f64 * scale,
        logical_position.y as f64 * scale,
    )));
}

fn build_smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the lab, wait for the scene to stabilize, take a screenshot, and exit.")
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<RtsCamera>(
            "camera entity exists",
        ))
        .then(assertions::component_satisfies::<RtsCameraRuntime>(
            "runtime initialized",
            |runtime| runtime.distance > 0.0 && runtime.pitch.is_finite(),
        ))
        .then(assertions::log_summary("smoke_launch summary"))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_smoke() -> Scenario {
    Scenario::builder("rts_camera_smoke")
        .description(
            "Assert the lab camera has valid runtime state and an active terrain-follow probe, then capture a baseline screenshot.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::component_satisfies::<RtsCameraRuntime>(
            "terrain follow has a height",
            |runtime| runtime.ground_height.is_some() && runtime.last_ground_hit.is_some(),
        ))
        .then(assertions::log_summary("rts_camera_smoke summary"))
        .then(inspect::dump_component_json::<RtsCameraRuntime>("rts_camera_smoke_runtime"))
        .then(Action::Screenshot("rts_camera_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_controls() -> Scenario {
    Scenario::builder("rts_camera_controls")
        .description(
            "Drive the real BEI input path for pan, rotate, and zoom on the uneven battlefield scene, then capture a verification screenshot.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut settings) =
                world.get_mut::<saddle_camera_rts_camera::RtsCameraSettings>(entity)
            {
                settings.controls.edge_pan = false;
            }
            set_cursor_position(world, Vec2::new(720.0, 450.0));
            store_runtime_baseline(world);
        })))
        .then(Action::Screenshot("rts_camera_controls_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::HoldKey {
            key: KeyCode::KeyW,
            frames: 24,
        })
        .then(Action::WaitFrames(8))
        .then(Action::HoldKey {
            key: KeyCode::KeyQ,
            frames: 18,
        })
        .then(Action::WaitFrames(8))
        .then(Action::MouseScroll {
            delta: Vec2::new(0.0, 6.0),
        })
        .then(Action::WaitFrames(24))
        .then(assertions::custom(
            "pan rotate zoom changed runtime",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.focus.distance(baseline.focus) > 1.0
                    && (runtime.yaw - baseline.yaw).abs() > 0.08
                    && (runtime.distance - baseline.distance).abs() > 0.5
            }),
        ))
        .then(assertions::log_summary("rts_camera_controls summary"))
        .then(inspect::dump_component_json::<RtsCameraRuntime>(
            "rts_camera_controls_runtime",
        ))
        .then(Action::Screenshot("rts_camera_controls_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_pointer_controls() -> Scenario {
    Scenario::builder("rts_camera_pointer_controls")
        .description(
            "Exercise edge pan, drag pan, drag rotation, and cursor-preserving zoom in the crate-local lab with assertions and visual checkpoints.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut settings) = world.get_mut::<saddle_camera_rts_camera::RtsCameraSettings>(entity) {
                settings.controls.pan = false;
                settings.controls.edge_pan = true;
                settings.controls.drag_pan = true;
                settings.controls.rotation = true;
                settings.controls.zoom = true;
                settings.anchors.zoom_anchor = saddle_camera_rts_camera::RtsCameraZoomAnchorMode::Focus;
            }
            set_cursor_position(world, Vec2::new(720.0, 450.0));
            store_runtime_baseline(world);
        })))
        .then(Action::Screenshot("rts_camera_pointer_controls_before".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_cursor_position(world, Vec2::new(1424.0, 450.0));
        })))
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "edge pan moves camera without general pan input",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.focus.xz().distance(baseline.focus.xz()) > 1.0
            }),
        ))
        .then(Action::Screenshot("rts_camera_pointer_controls_edge_pan".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut settings) = world.get_mut::<saddle_camera_rts_camera::RtsCameraSettings>(entity) {
                settings.controls.edge_pan = false;
            }
            set_cursor_position(world, Vec2::new(840.0, 430.0));
            store_runtime_baseline(world);
        })))
        .then(Action::WaitFrames(2))
        .then(Action::PressMouseButton(MouseButton::Right))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_cursor_position(world, Vec2::new(500.0, 660.0));
        })))
        .then(Action::WaitFrames(12))
        .then(Action::ReleaseMouseButton(MouseButton::Right))
        .then(assertions::custom(
            "drag pan adjusts focus",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.focus.xz().distance(baseline.focus.xz()) > 0.35
            }),
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_runtime_baseline(world);
        })))
        .then(Action::PressMouseButton(MouseButton::Middle))
        .then(Action::MouseMotion {
            delta: Vec2::new(180.0, 0.0),
        })
        .then(Action::WaitFrames(8))
        .then(Action::ReleaseMouseButton(MouseButton::Middle))
        .then(assertions::custom(
            "drag rotation updates yaw",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                (runtime.yaw - baseline.yaw).abs() > 0.08
            }),
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_cursor_position(world, Vec2::new(1040.0, 320.0));
        })))
        .then(Action::WaitUntil {
            label: "cursor anchor resolves".into(),
            condition: Box::new(|world: &World| {
                runtime(world).and_then(|runtime| runtime.last_cursor_anchor).is_some()
            }),
            max_frames: 45,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_runtime_baseline(world);
        })))
        .then(Action::PressKey(KeyCode::AltLeft))
        .then(Action::MouseScroll {
            delta: Vec2::new(0.0, 6.0),
        })
        .then(Action::WaitFrames(20))
        .then(Action::ReleaseKey(KeyCode::AltLeft))
        .then(assertions::custom(
            "cursor-preserving zoom override repositions focus",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                if baseline.last_cursor_anchor.is_none() {
                    return false;
                }
                (runtime.distance - baseline.distance).abs() > 0.5
                    && runtime.focus.xz().distance(baseline.focus.xz()) > 0.25
                    && runtime.last_cursor_anchor.is_some()
            }),
        ))
        .then(assertions::log_summary(
            "rts_camera_pointer_controls summary",
        ))
        .then(inspect::dump_component_json::<RtsCameraRuntime>(
            "rts_camera_pointer_controls_runtime",
        ))
        .then(Action::Screenshot("rts_camera_pointer_controls_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_follow_target() -> Scenario {
    Scenario::builder("rts_camera_follow_target")
        .description(
            "Enable follow mode, let the moving target travel, verify the camera converges near the tracked entity, then capture a screenshot.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("rts_camera_follow_target_before".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut follow) = world.get_mut::<RtsCameraFollow>(entity) {
                follow.enabled = true;
                follow.snap = true;
            }
        })))
        .then(Action::WaitFrames(45))
        .then(assertions::custom(
            "follow converges near tracked target",
            Box::new(|world: &World| {
                let Some(camera_entity) = camera_entity(world) else {
                    return false;
                };
                let Some(target_entity) = target_entity(world) else {
                    return false;
                };
                let Some(runtime) = world.get::<RtsCameraRuntime>(camera_entity) else {
                    return false;
                };
                let Some(target) = world.get::<Transform>(target_entity) else {
                    return false;
                };
                runtime.focus.xz().distance(target.translation.xz()) < 3.0
            }),
        ))
        .then(assertions::log_summary("rts_camera_follow_target summary"))
        .then(Action::Screenshot("rts_camera_follow_target_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_bookmarks() -> Scenario {
    Scenario::builder("rts_camera_bookmarks")
        .description(
            "Store the baseline camera as a bookmark, fly to a new battlefield location, then recall the saved bookmark and assert the runtime snaps back cleanly.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_cursor_position(world, Vec2::new(720.0, 450.0));
            store_runtime_baseline(world);
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut settings) =
                world.get_mut::<saddle_camera_rts_camera::RtsCameraSettings>(entity)
            {
                settings.controls.edge_pan = false;
            }
            if let Some(mut follow) = world.get_mut::<RtsCameraFollow>(entity) {
                follow.enabled = false;
            }
            if let Some(mut pane) = world
                .get_resource_mut::<saddle_camera_rts_camera_example_common::ExampleRtsPane>()
            {
                pane.follow_enabled = false;
            }
            if let Some(mut input) = world.get_mut::<RtsCameraInput>(entity) {
                input.set_bookmark_slot = Some(0);
            }
        })))
        .then(Action::Screenshot("rts_camera_bookmarks_before".into()))
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut pane) = world
                .get_resource_mut::<saddle_camera_rts_camera_example_common::ExampleRtsPane>()
            {
                pane.distance = 12.0;
            }
            if let Some(mut input) = world.get_mut::<RtsCameraInput>(entity) {
                input.fly_to_focus = Some(Vec3::new(-12.0, 0.0, -8.0));
                input.fly_to_yaw = Some(0.95);
                input.fly_to_distance = Some(12.0);
                input.fly_to_snap = false;
            }
        })))
        .then(Action::WaitFrames(36))
        .then(assertions::custom(
            "fly-to command moves camera to the authored bookmark location",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.focus.xz().distance(Vec2::new(-12.0, -8.0)) < 2.2
                    && (runtime.distance - 12.0).abs() < 1.5
                    && runtime.focus.distance(baseline.focus) > 4.0
            }),
        ))
        .then(Action::Screenshot("rts_camera_bookmarks_fly_to".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            let baseline_distance = world
                .get_resource::<RuntimeBaseline>()
                .map(|baseline| baseline.distance);
            if let Some(mut pane) = world
                .get_resource_mut::<saddle_camera_rts_camera_example_common::ExampleRtsPane>()
            {
                if let Some(distance) = baseline_distance {
                    pane.distance = distance;
                }
            }
            if let Some(mut input) = world.get_mut::<RtsCameraInput>(entity) {
                input.recall_bookmark_slot = Some(0);
                input.recall_bookmark_snap = true;
            }
        })))
        .then(Action::WaitFrames(6))
        .then(assertions::custom(
            "bookmark recall restores the baseline runtime",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.focus.distance(baseline.focus) < 0.3
                    && (runtime.yaw - baseline.yaw).abs() < 0.02
                    && (runtime.distance - baseline.distance).abs() < 0.1
            }),
        ))
        .then(assertions::log_summary("rts_camera_bookmarks summary"))
        .then(inspect::dump_component_json::<RtsCameraRuntime>(
            "rts_camera_bookmarks_runtime",
        ))
        .then(Action::Screenshot("rts_camera_bookmarks_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_headless_intents() -> Scenario {
    Scenario::builder("rts_camera_headless_intents")
        .description(
            "Disable the manual control families, drive the camera through programmatic fly-to intents only, and verify the runtime still updates cleanly.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_cursor_position(world, Vec2::new(720.0, 450.0));
            store_runtime_baseline(world);

            let Some(entity) = camera_entity(world) else {
                return;
            };

            if let Some(mut settings) =
                world.get_mut::<saddle_camera_rts_camera::RtsCameraSettings>(entity)
            {
                settings.controls.pan = false;
                settings.controls.edge_pan = false;
                settings.controls.drag_pan = false;
                settings.controls.zoom = false;
                settings.controls.rotation = false;
                settings.controls.follow = false;
            }

            if let Some(mut follow) = world.get_mut::<RtsCameraFollow>(entity) {
                follow.enabled = false;
            }

            if let Some(mut pane) = world
                .get_resource_mut::<saddle_camera_rts_camera_example_common::ExampleRtsPane>()
            {
                pane.follow_enabled = false;
            }
        })))
        .then(Action::Screenshot("rts_camera_headless_intents_before".into()))
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };

            if let Some(mut input) = world.get_mut::<RtsCameraInput>(entity) {
                input.fly_to_focus = Some(Vec3::new(12.0, 0.0, 12.0));
                input.fly_to_yaw = Some(-0.75);
                input.fly_to_distance = Some(14.0);
                input.fly_to_snap = false;
            }
        })))
        .then(Action::WaitFrames(36))
        .then(assertions::custom(
            "programmatic fly-to works without manual input families enabled",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };

                runtime.focus.xz().distance(Vec2::new(12.0, 12.0)) < 2.5
                    && (runtime.yaw - -0.75).abs() < 0.2
                    && (runtime.distance - 14.0).abs() < 1.0
                    && runtime.focus.distance(baseline.focus) > 4.0
            }),
        ))
        .then(assertions::log_summary(
            "rts_camera_headless_intents summary",
        ))
        .then(inspect::dump_component_json::<RtsCameraRuntime>(
            "rts_camera_headless_intents_runtime",
        ))
        .then(Action::Screenshot("rts_camera_headless_intents_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}
