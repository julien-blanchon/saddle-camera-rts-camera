# Architecture

`saddle-camera-rts-camera` keeps the controller state on the camera entity and splits runtime work into visible phases:

1. `ReadInput`
2. `ResolveTarget`
3. `FollowGround`
4. `ApplyBounds`
5. `AdvanceRuntime`
6. `ResolveCollision`
7. `SyncTransform`
8. `Debug`

The phases are exposed as public `SystemSet`s so a consumer can order other gameplay, tool, or presentation systems around them.

## Data Model

The runtime uses three layers of state instead of mutating `Transform` directly from input:

- `RtsCamera`
  Desired state. This is the public control seam.
  - `target_focus`
  - `target_yaw`
  - `target_distance`
  - `snap`
- `RtsCameraRuntime`
  Smoothed resolved state used to render the camera.
  - `focus`
  - `yaw`
  - `pitch`
  - `distance`
  - `ground_height`
  - `last_ground_hit`
  - `last_cursor_anchor`
- `Transform`
  Final camera pose derived from the runtime state in `PostUpdate`

That separation keeps programmatic jumps, follow-target motion, smoothing, bounds, and cursor-anchor compensation on one consistent path.

`RtsCameraBookmarks` lives on the same entity and stores optional saved views. Bookmark recall writes back into `RtsCamera`, so saved views travel through the same bounds, smoothing, and transform-sync pipeline as manual control.

## Input Flow

The runtime accepts input through `RtsCameraInput`.

### Production path

External systems write:

- `pan`
- `edge_pan`
- `zoom`
- `rotate`
- `rotate_drag_delta`
- `drag_pan_active`
- `drag_rotate_active`
- `cursor_position`
- `zoom_to_cursor`
- `fly_to_focus`
- `fly_to_yaw`
- `fly_to_distance`
- `fly_to_snap`
- `set_bookmark_slot`
- `recall_bookmark_slot`
- `recall_bookmark_snap`

The examples and crate-local lab show a `bevy_enhanced_input` bridge that evaluates its context in `Update`, then writes this component after `EnhancedInputSystems::Apply` and before `RtsCameraSystems::ResolveTarget`.

### Fallback path

If the `fallback-input` feature is enabled and the app adds `RtsCameraFallbackInputPlugin`, cameras carrying `RtsCameraFallbackControls` can opt into a raw Bevy keyboard/mouse bridge that writes the same `RtsCameraInput` component. The fallback adapter lives outside the core plugin so the runtime stays headless by default and does not hard-depend on any specific physical input source.

Only cameras tagged with `RtsCameraInputTarget` are candidates for shared pointer input. The active camera with the highest `Camera.order` wins.

## System Flow

### `ReadInput`

- consumer-owned input adapters
- optional `fallback::apply_fallback_controls`

The core plugin does not install any input reader by default. This set exists as a public seam for consumer-owned input adapters, and the optional `RtsCameraFallbackInputPlugin` uses it for the crate's raw RTS-style fallback mapping.

### `ResolveTarget`

- `initialize_added_cameras`
- `apply_programmatic_commands`
- `sync_follow_targets`
- `apply_camera_input`

`initialize_added_cameras` seeds runtime state from the authored target state so new cameras do not jump on their first frame.

`apply_programmatic_commands` resolves one-shot fly-to requests and bookmark save/recall before follow and manual input are processed.

`sync_follow_targets` updates `target_focus` from the tracked entity plus `RtsCameraFollow.offset`.

`apply_camera_input` is the core intent resolver:

- converts general pan and edge-pan intent into world-space movement relative to current yaw
- scales pan speed across the zoom range
- resolves cursor-ground anchors from `viewport_to_world` plus `MeshRayCast`
- supports drag pan by preserving a cursor-ground world anchor
- supports cursor-preserving rotation pivots
- supports cursor-preserving zoom anchors
- updates follow offsets instead of fighting follow mode when follow is enabled

## Terrain-Follow Pipeline

Terrain probing is intentionally generic:

- a downward `MeshRayCast` is fired from above the **smoothed** runtime focus position
- only entities marked `RtsCameraGround` participate
- hit height is converted into desired focus height by adding `ground.clearance`
- `ground.keep_last_height_on_miss` determines whether the last valid height is retained when no ground mesh is hit

The runtime stores the resolved height in `RtsCameraRuntime.ground_height` and smooths vertical motion separately from horizontal focus motion. That avoids the common jitter where raycast height and lateral smoothing fight each other.

The debug set exposes both:

- `last_ground_hit`
- `last_cursor_anchor`

which makes BRP and screenshot-based debugging much easier when tuning terrain-follow behavior.

## Bounds Pipeline

Bounds operate on focus XZ, not camera eye position.

- `Hard`
  Clamp `target_focus` directly into the allowed rectangle.
- `Soft`
  Compress pan deltas near the edge before the final clamp pass.

`ApplyBounds` still performs a final hard clamp even in soft mode so programmatic teleports, follow-target motion, or cursor-anchor compensation cannot push focus outside the legal rectangle.

When follow mode is active, clamp corrections are also applied to `RtsCameraFollow.offset` so the next `sync_follow_targets` pass does not immediately reintroduce the out-of-bounds position.

## Smoothing Model

The crate uses frame-rate-independent exponential decay.

- horizontal focus smoothing:
  `motion.focus_decay`
- vertical terrain smoothing:
  `motion.ground_decay`
- yaw smoothing:
  `motion.yaw_decay`
- zoom smoothing:
  `motion.distance_decay`

`0.0` means snap immediately, not freeze. That behavior matters for consumers that want per-channel smoothing disabled without rewriting the runtime.

Yaw smoothing uses shortest-angle interpolation so values wrap cleanly across `-PI..PI`.

## Pitch Model

Pitch is derived from zoom distance, not integrated separately.

- near distance:
  `pitch.near_angle`
- far distance:
  `pitch.far_angle`

The resolved `distance` value determines pitch every frame, which keeps zoom and pitch coupling stable and predictable.

## Collision Avoidance

After the runtime state is advanced, `ResolveCollision` prevents the camera eye from clipping through ground meshes.

- A backward ray is cast from the smoothed focus along the camera's look-back direction
- Only entities marked `RtsCameraGround` participate
- If terrain is closer than `runtime.distance + collision.clearance`, the runtime distance is clamped to `hit_distance - clearance`, floored at `collision.min_distance`
- The collision system only ever reduces runtime distance — it never increases it, so the target-state smoothing path remains authoritative for zoom-out

This is the same "spring arm" pattern used by Unreal Engine's camera boom. It runs after `AdvanceRuntime` so it sees the fully smoothed state, and before `SyncTransform` so the corrected distance is what reaches the final `Transform`.

## Final Transform

`SyncTransform` runs in `PostUpdate` before transform propagation and writes the final `Transform` from:

- smoothed focus
- smoothed yaw
- derived pitch
- smoothed distance

Keeping this step in `PostUpdate` means consumer systems can order world simulation before camera resolve in `Update`, while anything that depends on the final rendered pose can read it after transform propagation in the same frame.

## Why The Public Sets Exist

- `ReadInput`
  Lets consumers install their own input bridge, automation writer, or the optional fallback input plugin before target resolution.
- `ResolveTarget`
  Lets gameplay systems update follow targets, bookmarks, alerts, or tool focus points before smoothing.
- `FollowGround`
  Lets projects insert custom terrain or ground-height authoring before or after the default probe.
- `ApplyBounds`
  Lets projects enforce additional map or scenario limits.
- `AdvanceRuntime`
  Lets downstream systems read the smoothed runtime state before render sync.
- `ResolveCollision`
  Lets projects replace or augment camera-terrain collision avoidance.
- `SyncTransform`
  Lets presentation systems order around the final camera pose.
- `Debug`
  Lets projects disable or order gizmo output cleanly.
