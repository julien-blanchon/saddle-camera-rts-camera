# `saddle-camera-rts-camera-lab`

Crate-local showcase and verification app for `saddle-camera-rts-camera`.

## Purpose

- verify keyboard pan, edge pan, drag pan, rotation, zoom, and terrain follow in a richer Bevy scene
- keep a moving follow target available for BRP and E2E checks
- expose the shared camera runtime through an on-screen overlay plus reflected ECS components

## Run

```bash
cargo run -p saddle-camera-rts-camera-lab
```

## E2E

```bash
cargo run -p saddle-camera-rts-camera-lab --features e2e -- smoke_launch
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_smoke
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_controls
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_pointer_controls
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_follow_target
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_bookmarks
cargo run -p saddle-camera-rts-camera-lab --features e2e -- rts_camera_headless_intents
```

## BRP

```bash
cargo run -p saddle-camera-rts-camera-lab
uv run --project .codex/skills/bevy-brp/script brp world query \
  bevy_ecs::name::Name saddle_camera_rts_camera::components::RtsCameraRuntime
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle-camera-rts-camera-lab.png
```

Use the reflected type path reported by `brp world list`, not the crate-root re-export name.
