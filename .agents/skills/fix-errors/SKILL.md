---
name: fix-errors
description: Diagnose and fix Warper Rust build errors, lint failures, and test failures when compilation or validation fails.
---

# Fix build and test failures

Use the pinned toolchain in `rust-toolchain.toml` and the validation commands in `AGENTS.md`. Read the failing output and current code before changing either the implementation or tests.

For the native app, start with a focused check:

```bash
cargo check -p warp --features gui --all-targets --locked
cargo nextest run -p warp -E 'test(test_name)'
```

Use `./script/presubmit` for the complete repository checks. Its Clippy commands treat the completer separately:

```bash
cargo clippy --workspace --exclude warp_completer --all-targets --tests -- -D warnings
cargo clippy -p warp_completer --all-targets --tests -- -D warnings
```

When changing a function signature or enum, update all callers and platform-specific matches. Remove obsolete imports and parameters. Preserve `local_fs`, OS, and shell distinctions still present in the affected code.

Browser/WASM builds have been removed under `specs/WARPER-013/TECH.md`; do not add that target to a fix or validation workflow.

For test implementation and registration details, use the Rust unit test or Warper integration test skill. Report failures caused by unavailable toolchain components or dependencies separately from source failures; required checks must produce a result.
