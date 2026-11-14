---
name: integration-test-video
description: Run Warper integration tests with screenshot and video capture when the user asks to record a test, collect screenshots, or inspect recording artifacts.
---

# Integration Test Video Recording

Use this skill for Warper integration test recordings and screenshots.

The relevant implementation lives in:

- `crates/integration/src/bin/integration.rs`
- `crates/integration/src/test/video_recording.rs`
- `crates/integration/tests/integration/ui_tests.rs`
- `crates/warpui_core/src/integration/driver.rs`
- `crates/warpui_core/src/integration/step.rs`
- `crates/warpui_core/src/integration/video_recorder.rs`
- `crates/warpui_core/src/integration/artifacts.rs`
- `crates/warpui_core/src/integration/overlay.rs`

## Command to invoke a test

For a single manually-invoked recording test, prefer the integration binary:

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
cargo run -p integration --bin integration -- test_video_recording
```

That is the command shown by the sample test in `crates/integration/src/test/video_recording.rs`.

If you want the driver to auto-record a test or set of tests, add `WARP_INTEGRATION_TEST_VIDEO`:

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_VIDEO=test_video_recording \
cargo run -p integration --bin integration -- test_video_recording
```

For broader integration test runs, the same env vars work with the normal test runner:

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_VIDEO=test_foo,test_bar \
cargo nextest run --no-fail-fast --workspace test_foo
```

## Environment variables

### `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`

- Set this to `1` when you need real frame capture.
- Use it for screenshot/video workflows and manual visual verification.
- Without a real display, expect recording workflows to be incomplete or unusable.

### `WARP_INTEGRATION_TEST_VIDEO`

This is the main env var that controls driver-managed video recording in `crates/warpui_core/src/integration/driver.rs`.

Behavior:

- Unset or empty: auto-recording is disabled.
- `1` or `all`: auto-record every test in the run.
- Comma-separated test names: auto-record only those tests.

Examples:

```bash
# Record every test in the run
WARP_INTEGRATION_TEST_VIDEO=all
```

```bash
# Record only specific tests
WARP_INTEGRATION_TEST_VIDEO=test_foo,test_bar
```

Tests that call `with_start_recording()` and `with_stop_recording()` record that span without `WARP_INTEGRATION_TEST_VIDEO`. The environment variable records a whole test without changing its code.

### `WARP_INTEGRATION_TEST_ARTIFACTS_DIR`

This controls the root artifact directory used by `TestArtifacts` in `crates/warpui_core/src/integration/artifacts.rs`.

If unset, artifacts go under:

```text
$TMPDIR/warp_integration_test_artifacts
```

Each run gets a timestamped directory:

```text
<artifacts_root>/<test_name>/<timestamp>/
```

This is the main directory to inspect for screenshots, logs, and the final `recording.mp4`.

### `WARP_INTEGRATION_TEST_VIDEO_DIR`

This env var exists in `crates/warpui_core/src/integration/video_recorder.rs` as the lower-level recorder output root helper, defaulting to:

```text
$TMPDIR/warp_integration_video_captures
```

The integration driver writes finalized videos into `WARP_INTEGRATION_TEST_ARTIFACTS_DIR`.

## How overlays work

There is no separate overlay environment variable.

Overlay annotations are produced from the input events the test dispatches while recording is active. The overlay pipeline is implemented in `crates/warpui_core/src/integration/overlay.rs`, and the event capture hooks live in `crates/warpui_core/src/integration/step.rs`.

To get useful overlays in the final video, drive the test with APIs that emit mouse and keyboard events, such as:

- `with_event(...)`
- `with_event_fn(...)`
- `with_click_on_saved_position(...)`
- `with_keystrokes(...)`

Overlay types currently exercised by the sample test:

- mouse click indicators
- drag trails
- keyboard shortcut pills

In practice:

- Mouse down / drag / mouse up events create click and drag overlays.
- KeyDown events create keyboard overlay pills.
- If a test only records frames and never dispatches relevant input events, the resulting video will not show these annotations.

## How to write a test that takes screenshots

Use `TestStep::with_take_screenshot("filename.png")`.

Example pattern:

```rust
TestStep::new("Take screenshot after bootstrap")
    .with_take_screenshot("after_bootstrap.png")
```

The screenshot request is stored during the step and written by the driver after the step renders. The PNG is saved in the test's timestamped artifacts directory.

## How to write a test that records video

### Minimum pattern

1. Use `Builder::new().with_real_display()`.
2. Add a step with `with_start_recording()`.
3. Run the actions/events you want captured.
4. Add a step with `with_stop_recording()`.

Example shape:

```rust
Builder::new()
    .with_real_display()
    .with_step(TestStep::new("Start recording").with_start_recording())
    .with_step(/* actions and events */)
    .with_step(TestStep::new("Stop recording").with_stop_recording())
```

### Mouse and keyboard overlays

Prefer explicit UI-driving steps that emit mouse and key events:

- click with `with_click_on_saved_position(...)`
- dispatch raw mouse events with `with_event(...)` / `with_event_fn(...)`
- send keyboard shortcuts with `with_keystrokes(...)`

For drag overlays, send a sequence like:

- `LeftMouseDown`
- one or more `LeftMouseDragged`
- `LeftMouseUp`

### Optional validation

Use a `with_on_finish(...)` hook to check that a recording test produced the expected files, such as:

- `recording.mp4`
- `recording.log`
- screenshot PNGs

The sample test does exactly that.

## Where the video assets go

The normal output location is:

```text
${WARP_INTEGRATION_TEST_ARTIFACTS_DIR:-$TMPDIR/warp_integration_test_artifacts}/<test_name>/<timestamp>/
```

Common artifacts in that directory:

- `recording.mp4`
- `recording.log`
- any screenshots requested with `with_take_screenshot(...)`

For `test_video_recording`, the sample test expects:

- `after_bootstrap.png`
- `after_commands.png`
- `recording.mp4`
- `recording.log`

If MP4 encoding fails during finalization, the recorder falls back to per-frame PNGs in a sibling directory like:

```text
recording_frames/
```

with files such as:

```text
recording_0000.png
```

## How to review the assets

1. Open the latest timestamped artifact directory for the test.
2. Review `recording.mp4` first to confirm:
   - the UI state is correct
   - recording actually started and stopped in the intended window
   - overlay annotations appear at the right moments
3. Review any PNG screenshots captured by the test.
4. Check `recording.log` if the output looks incomplete or suspicious.
5. If `recording.mp4` is missing, look for fallback frame PNGs.

When summarizing results for the user, include the exact artifact directory path.

## Sample test for video recording

The sample manual test is `test_video_recording`.

It is:

- registered in `crates/integration/src/bin/integration.rs`
- listed in `crates/integration/tests/integration/ui_tests.rs`
- implemented in `crates/integration/src/test/video_recording.rs`
