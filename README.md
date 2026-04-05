# Saddle RTS Camera

Reusable RTS, builder, and tactics camera for [Bevy](https://bevyengine.org).

Target-state camera model with smoothing, terrain probing, bounds, cursor-aware zoom and rotation pivots, and follow-target support. Bevy-only and project-agnostic — no game states, UI deps, physics deps, or project imports.

## What It Is For

- Perspective RTS and city-builder cameras
- Map editors, scenario editors, and god-game navigation
- Keyboard pan, edge pan, drag pan, wheel zoom, and yaw rotation
- Terrain-aware focus height using Bevy mesh ray casts
- Zoom-anchor and rotation-pivot policies driven by focus or cursor-ground hits
- Bounded camera travel with hard or soft map limits
- Camera collision avoidance against terrain to prevent clipping
- Follow-target views for alerts, spectating, and points of interest
- Programmatic fly-to commands and bookmark save/recall for RTS workflows

## Quick Start

```toml
[dependencies]
saddle-camera-rts-camera = { git = "https://github.com/julien-blanchon/saddle-camera-rts-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_rts_camera::{
    RtsCamera, RtsCameraGround, RtsCameraInputTarget, RtsCameraPlugin, RtsCameraSettings,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RtsCameraPlugin::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Terrain"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.22, 0.30, 0.20))),
        RtsCameraGround,
    ));

    commands.spawn((
        Name::new("RTS Camera"),
        RtsCamera::looking_at(Vec3::ZERO, Vec3::new(-18.0, 18.0, 18.0)),
        RtsCameraSettings::default(),
        RtsCameraInputTarget,
    ));
}
```

The runtime is driven through the `RtsCamera` target state plus the `RtsCameraInput` inbox. Consumers can mutate those public components directly or feed them from their own input layer.

That same input surface also handles one-shot fly-to commands plus bookmark save/recall, so minimaps, alerts, and editor tooling can reuse the same smoothing path as manual pan/zoom/rotate controls.

`RtsCameraInput` keeps general pan intent separate from edge-pan intent so `RtsCameraControlFlags::pan` and `RtsCameraControlFlags::edge_pan` can be toggled independently at runtime.

For always-on tools and examples, `RtsCameraPlugin::always_on(Update)` is the convenience constructor.

## Public API

| Type | Purpose |
| --- | --- |
| `RtsCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `RtsCameraSystems` | Public ordering hooks: `ReadInput`, `ResolveTarget`, `FollowGround`, `ApplyBounds`, `AdvanceRuntime`, `ResolveCollision`, `SyncTransform`, `Debug` |
| `RtsCamera` | Main controller component containing desired focus, yaw, distance, and snap requests |
| `RtsCameraRuntime` | Smoothed runtime state: focus, yaw, pitch, distance, ground height, last ground hit, last cursor anchor |
| `RtsCameraBookmark` / `RtsCameraBookmarks` | Captured camera views for RTS-style bookmark save/recall |
| `RtsCameraSettings` | Top-level tuning surface for distance, pitch, motion, bounds, terrain follow, collision, anchors, and control flags |
| `RtsCameraBounds` / `RtsCameraBoundsMode` | Hard or soft travel limits for focus XZ |
| `RtsCameraGround` | Marker for meshes that participate in terrain-follow and cursor-ground ray casts |
| `RtsCameraFollow` | Optional follow-target seam with offset and snap behavior |
| `RtsCameraInput` | Public intent inbox for pan, zoom, rotate, drag, and cursor position |
| `RtsCameraInputTarget` | Opt-in marker for which camera should consume shared pointer input |
| `RtsCameraFallbackControls` | Explicit raw-input fallback path for examples, prototypes, or tool scenes |
| `RtsCameraDebug` | Enables focus, anchor, ground-hit, and bounds gizmos |
| Messages | `RtsCameraBookmarkStored`, `RtsCameraBookmarkRecalled`, `RtsCameraFlyToApplied` |

## Input Model

- **Production path:** Use your own input adapter and write into `RtsCameraInput`.
- **Included fallback:** `RtsCameraFallbackControls` is a raw Bevy-input bridge for minimal examples and lightweight tools.
- **`bevy_enhanced_input`:** The examples show a BEI integration that keeps the runtime crate itself Bevy-only.

Only cameras tagged with `RtsCameraInputTarget` participate in shared pointer input. If several active cameras carry the marker, the highest `Camera.order` wins.

## State Model

The crate deliberately separates:

- Desired state in `RtsCamera`
- Smoothed resolved state in `RtsCameraRuntime`
- Final rendered transform written in `PostUpdate`

That split keeps smoothing, cursor-anchor adjustments, follow offsets, and programmatic teleports on the same code path.

## Examples

Run from the `examples/` directory:

```bash
cd examples
cargo run -p saddle-camera-rts-camera-example-basic
```

| Example | Purpose |
| --- | --- |
| `basic` | Minimal fallback-controller setup on flat terrain |
| `bookmarks` | Battlefield command posts with RTS-style bookmark recall and overwrite |
| `terrain_follow` | Uneven ground with focus-height gizmos |
| `cursor_zoom` | Focus zoom by default, Alt for cursor-preserving zoom |
| `bounds` | Soft bounds behavior with visible clamp loop |
| `follow_target` | Moving follow target with manual offset adjustment |
| `enhanced_input` | `bevy_enhanced_input` bridge writing into `RtsCameraInput` |

Every example now ships with a live `saddle-pane` control surface so distance limits, cursor zoom policy, pan speeds, and debug gizmos can be tuned at runtime.

## Known Limitations

- The runtime currently targets perspective cameras only.
- Terrain follow uses Bevy mesh ray casts against `RtsCameraGround` meshes. If a project already owns a richer terrain query, it can bypass the built-in probing by writing the desired focus height into `RtsCamera`.
- There is intentionally no built-in UI hover gate. Consumers should disable `RtsCameraInputTarget` or stop writing `RtsCameraInput` when UI should own the pointer.

## Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)

## License

[MIT-0](LICENSE) — use freely, no attribution required.
