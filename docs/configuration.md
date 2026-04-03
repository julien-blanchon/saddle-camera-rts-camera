# Configuration

`RtsCameraSettings` is the main per-camera tuning surface for `saddle-camera-rts-camera`.

## `RtsCameraSettings`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `distance` | `RtsCameraDistanceSettings` | see below | positive finite values | Zoom distance limits | Used for clamping target and runtime distance |
| `pitch` | `RtsCameraPitchSettings` | see below | finite radians | Pitch over distance range | Derived every frame from resolved distance |
| `motion` | `RtsCameraMotionSettings` | see below | non-negative finite values | Pan, zoom, rotate speeds and smoothing | `0.0` decay snaps immediately |
| `ground` | `RtsCameraGroundSettings` | see below | per field | Terrain-follow behavior | Probes only `RtsCameraGround` meshes |
| `bounds` | `Option<RtsCameraBounds>` | `None` | `None` or any finite rectangle with `min <= max` | Limits focus XZ travel | Soft mode still finishes with a hard clamp |
| `anchors` | `RtsCameraAnchorSettings` | cursor zoom, focus rotation | enum values below | Cursor-aware zoom and rotation policy | Depends on a successful cursor-ground hit |
| `controls` | `RtsCameraControlFlags` | all enabled | booleans | Runtime gating for control families | Disables handling without removing components |
| `edge_pan` | `RtsCameraEdgePanSettings` | margin `18.0` | `>= 0` | Screen-edge panning threshold | Used by fallback input and example BEI bridge |

## `RtsCameraDistanceSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `min` | `f32` | `8.0` | `> 0` | Minimum allowed camera distance |
| `max` | `f32` | `44.0` | `>= min` | Maximum allowed camera distance |

Pan speed interpolation and pitch interpolation both use the normalized position of the resolved distance within this range.

## `RtsCameraPitchSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `near_angle` | `f32` | `58°` | finite radians | Pitch when distance is at `distance.min` |
| `far_angle` | `f32` | `32°` | finite radians | Pitch when distance is at `distance.max` |

Smaller angles feel more top-down. Larger angles feel more inspectable and character-facing.

## `RtsCameraMotionSettings`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `pan_speed_near` | `f32` | `10.0` | `>= 0` | World-space pan speed at close zoom | Lower values improve near-field precision |
| `pan_speed_far` | `f32` | `34.0` | `>= 0` | World-space pan speed at far zoom | Higher values improve map traversal |
| `zoom_speed` | `f32` | `2.8` | `>= 0` | Distance change per zoom input unit | Applied before distance clamp |
| `rotation_speed` | `f32` | `1.9` | `>= 0` | Keyboard or axis yaw speed in radians per second | Used with `input.rotate` |
| `drag_rotation_speed` | `f32` | `0.009` | `>= 0` | Drag-rotation multiplier per pointer delta pixel | Used with `input.rotate_drag_delta` |
| `focus_decay` | `f32` | `18.0` | `>= 0` | Horizontal focus smoothing | `0.0` snaps immediately |
| `ground_decay` | `f32` | `10.0` | `>= 0` | Vertical focus smoothing | Keep separate from horizontal for slope stability |
| `yaw_decay` | `f32` | `18.0` | `>= 0` | Yaw smoothing | Uses shortest-angle interpolation |
| `distance_decay` | `f32` | `16.0` | `>= 0` | Zoom smoothing | `0.0` snaps immediately |

## `RtsCameraGroundSettings`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Enables downward ground probing | When `false`, focus height comes directly from `target_focus.y` |
| `clearance` | `f32` | `1.2` | any finite value | Offset above the ground hit point | Positive values keep the focus above the terrain surface |
| `probe_height` | `f32` | `256.0` | `>= 0` | Height above focus used to start the downward ray | Needs to clear the tallest terrain under the camera |
| `keep_last_height_on_miss` | `bool` | `true` | `true` or `false` | Retains the previous valid height if the current probe misses | Prevents hard drops when crossing gaps or non-ground meshes |

Interaction notes:

- `clearance` affects both terrain-follow and debug bounds drawing.
- With `keep_last_height_on_miss = true`, the runtime prefers stability over immediate fallback to `target_focus.y`.

## `RtsCameraBounds`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `min` | `Vec2` | `(-24, -24)` | finite values | Minimum X and Z bounds |
| `max` | `Vec2` | `(24, 24)` | finite values with `max >= min` | Maximum X and Z bounds |
| `mode` | `RtsCameraBoundsMode` | `Hard` | `Hard` or `Soft` | Chooses direct clamp or delta compression near the edge |
| `soft_margin` | `f32` | `4.0` | `>= 0` | Width of the slowdown band in soft mode |

### `RtsCameraBoundsMode`

| Variant | Effect |
| --- | --- |
| `Hard` | Clamp the target focus directly into the bounds rectangle |
| `Soft` | Scale pan deltas down as focus approaches the edge, then still finish with a hard clamp |

## `RtsCameraAnchorSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `zoom_anchor` | `RtsCameraZoomAnchorMode` | `Cursor` | `Focus` or `Cursor` | Determines whether zoom tries to preserve the focus point or the cursor-ground hit |
| `rotation_pivot` | `RtsCameraRotationPivotMode` | `Focus` | `Focus` or `Cursor` | Determines which world point should stay visually stable during rotation |

### `RtsCameraZoomAnchorMode`

| Variant | Effect |
| --- | --- |
| `Focus` | Zoom changes distance around the current focus point |
| `Cursor` | When a cursor-ground hit exists, adjust focus to keep that world point under the cursor |

### `RtsCameraRotationPivotMode`

| Variant | Effect |
| --- | --- |
| `Focus` | Rotate around the current focus point |
| `Cursor` | When a cursor-ground hit exists, adjust focus so the cursor-ground point remains stable while yaw changes |

Anchor interaction notes:

- Cursor-based anchors require a valid cursor position and a successful ground hit or fallback plane intersection.
- Cursor anchors still obey bounds after compensation is applied.

## `RtsCameraControlFlags`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `pan` | `bool` | `true` | `true` or `false` | Enables keyboard or axis pan and drag-pan offset application |
| `edge_pan` | `bool` | `true` | `true` or `false` | Enables viewport-edge pan contribution independently from general pan intent |
| `drag_pan` | `bool` | `true` | `true` or `false` | Enables cursor-anchor drag panning |
| `zoom` | `bool` | `true` | `true` or `false` | Enables wheel or axis zoom |
| `rotation` | `bool` | `true` | `true` or `false` | Enables keyboard or drag yaw updates |
| `follow` | `bool` | `true` | `true` or `false` | Enables `RtsCameraFollow` synchronization and follow-offset maintenance |

## `RtsCameraEdgePanSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `margin` | `f32` | `18.0` | `>= 0` | Width in logical pixels of the edge-pan activation band |

`margin = 0.0` effectively disables edge pan even if `controls.edge_pan` is still `true`.

## `RtsCamera`

The controller component is also part of the public control surface.

| Field | Purpose |
| --- | --- |
| `target_focus` | Desired world focus point |
| `target_yaw` | Desired yaw in radians |
| `target_distance` | Desired camera distance |
| `snap` | One-frame request to snap runtime state directly to target |

Useful helpers:

- `RtsCamera::looking_at(focus, eye)`
- `RtsCamera::snap_to(focus, yaw, distance)`

## `RtsCameraBookmark`

| Field | Type | Effect |
| --- | --- | --- |
| `focus` | `Vec3` | Saved world focus point |
| `yaw` | `f32` | Saved yaw in radians |
| `distance` | `f32` | Saved camera distance |

`RtsCameraBookmark::from_runtime(&RtsCameraRuntime)` captures the currently rendered view instead of the unsmoothed target state.

## `RtsCameraBookmarks`

| Field | Type | Effect |
| --- | --- | --- |
| `slots` | `Vec<Option<RtsCameraBookmark>>` | Sparse bookmark table stored on each camera |

Helpers:

- `set(slot, bookmark)` grows the slot table as needed
- `get(slot)` returns the saved bookmark when present

## `RtsCameraFollow`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target` | `Entity` | placeholder | Entity whose transform drives focus |
| `offset` | `Vec3` | zero | Offset from target position to focus point |
| `enabled` | `bool` | `true` | Toggles follow behavior without removing the component |
| `snap` | `bool` | `false` | If `true`, the runtime requests a snap when follow updates target focus |

Manual pan and cursor-anchor corrections update `offset` when follow mode is active, so the camera can stay in follow mode without fighting user adjustments.

## `RtsCameraInput`

This is the generic intent inbox for external controllers.

| Field | Type | Effect |
| --- | --- | --- |
| `pan` | `Vec2` | Local-space keyboard, action-axis, or tool-authored pan input before yaw rotation |
| `edge_pan` | `Vec2` | Viewport-edge pan contribution before yaw rotation |
| `zoom` | `f32` | Zoom input units for the current frame |
| `rotate` | `f32` | Keyboard or axis yaw input for the current frame |
| `rotate_drag_delta` | `f32` | Pointer-delta contribution for drag rotation |
| `drag_pan_active` | `bool` | Enables cursor-anchor drag pan logic |
| `drag_rotate_active` | `bool` | Enables drag rotation logic |
| `cursor_position` | `Option<Vec2>` | Current logical cursor position for anchor resolution |
| `zoom_to_cursor` | `bool` | Per-frame override that forces cursor-preserving zoom even if `anchors.zoom_anchor` is `Focus` |
| `fly_to_focus` | `Option<Vec3>` | One-shot programmatic fly-to focus target |
| `fly_to_yaw` | `Option<f32>` | Optional one-shot yaw override for the fly-to command |
| `fly_to_distance` | `Option<f32>` | Optional one-shot distance override for the fly-to command |
| `fly_to_snap` | `bool` | Requests an immediate snap for the current fly-to command instead of smoothing |
| `set_bookmark_slot` | `Option<usize>` | Captures the rendered camera view into the requested bookmark slot |
| `recall_bookmark_slot` | `Option<usize>` | Restores a previously saved bookmark into `RtsCamera` target state |
| `recall_bookmark_snap` | `bool` | Requests an immediate snap when recalling a bookmark |

The runtime clears `RtsCameraInput` at the end of each frame, so external writers should repopulate it every frame.

## `RtsCameraFallbackControls`

This optional component powers the crate's explicit raw-input fallback bridge.

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `pan_up` | `KeyCode` | `W` | Adds positive local pan Y while held |
| `pan_down` | `KeyCode` | `S` | Adds negative local pan Y while held |
| `pan_left` | `KeyCode` | `A` | Adds negative local pan X while held |
| `pan_right` | `KeyCode` | `D` | Adds positive local pan X while held |
| `rotate_left` | `KeyCode` | `Q` | Adds positive yaw intent while held |
| `rotate_right` | `KeyCode` | `E` | Adds negative yaw intent while held |
| `drag_pan_button` | `MouseButton` | `Right` | Enables cursor-anchor drag pan while held |
| `rotate_drag_button` | `MouseButton` | `Middle` | Enables pointer-delta rotation while held |
| `zoom_to_cursor` | `bool` | `true` | Uses cursor-preserving zoom whenever no modifier is configured |
| `zoom_to_cursor_modifier` | `Option<KeyCode>` | `None` | Optional modifier that overrides `zoom_to_cursor` on a per-frame basis |
| `enabled` | `bool` | `true` | Disables the fallback bridge without removing the component |

Use this component for examples, prototypes, and lightweight tool scenes. For production gameplay input, prefer a dedicated adapter that writes directly into `RtsCameraInput`.

## `RtsCameraInputTarget`

This marker opts a camera into shared pointer-driven input resolution. When several active cameras carry the marker, the highest `Camera.order` wins.

## `RtsCameraDebug`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | `bool` | `true` | Draws focus, bounds, ground-hit, and cursor-anchor gizmos in the public `Debug` set |
