---
name: remove-feature-flag
description: Remove a local Warper feature flag when the user has decided to keep or delete its behavior permanently.
---

# Remove a feature flag

Confirm the requested final behavior from the task and current implementation. Removing a flag can mean keeping the enabled branch or deleting the feature; follow the maintainer's decision.

Check these locations:

- The feature definition and default features in `app/Cargo.toml`.
- The `FeatureFlag` variant, local enablement lists, and descriptions in `crates/warp_features/src/lib.rs`.
- The compile-time registration in `app/src/lib.rs`.
- Runtime checks, `cfg` attributes, keybinding predicates, settings, and tests that reference the flag.

Search both the Rust variant and Cargo feature names. Remove the obsolete branch and flag-specific interfaces instead of retaining a disabled stub. Keep code shared by supported behavior.

Update directly affected documentation and run the relevant native checks from `AGENTS.md`. Feature removal does not need an upstream team approval or release-channel rollout.
