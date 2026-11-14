# Warper development

## Product scope

- Keep terminal sessions, shell integration, blocks, command history, tabs, panes, session restoration, themes, keybindings, local workflows, launch configs, file editing, and third-party CLI-agent integration working.
- Built-in AI, model providers, MCP management, agent orchestration, and the diff review viewer/editor are removed. Git diff statistics remain passive terminal metadata.
- Accounts, billing, hosted Warp Drive, teams, session sharing, Oz, telemetry uploads, hosted crash reporting, Warp autoupdate, and remote feature-flag fetching are removed.
- User requests set development priorities and the scope of upstream ports.
- Preserve the everyday terminal's settings, history, session state, and existing storage paths. Keep test artifacts separate from user data.

## Repository map

- `app/src/terminal/`: terminal views, sessions, shell integration, and third-party CLI-agent support.
- `app/src/workspace/` and `app/src/pane_group/`: windows, tabs, panes, and layout.
- `app/src/settings/` and `app/src/settings_view/`: local settings and their UI.
- `app/src/code/`: file editing and language-server integration.
- `app/src/git_status/`: repository statistics used by terminal chips and branch selection.
- `crates/warpui/`, `crates/warpui_core/`, and `crates/warpui_extras/`: UI framework and rendering.
- `crates/editor/`: text buffers and editing.
- `crates/persistence/`: SQLite schema and migrations.
- `crates/integration/`: integration tests using the Builder/TestStep framework.
- `specs/`: current requirements and archived design history. Follow the current index in [specs/README.md](specs/README.md).

## Implementation conventions

- Inspect the worktree before editing; other work may be present. Keep changes focused on the request.
- Use existing dependencies and components. Read `.agents/skills/warp-ui-guidelines/SKILL.md` before UI changes.
- Name UI context parameters `ctx` and put them last, before a trailing closure. Remove unused parameters and update callers.
- Prefer imports over long path qualifiers, inferred types where clear, inline format arguments, and exhaustive enum matches.
- Create `MouseStateHandle` values during construction and reuse them across renders. Creating fresh handles during rendering breaks mouse interactions.
- Keep `TerminalModel` lock scopes short. Check callers before acquiring a lock; nested locks on the same model can freeze the terminal. Pass an existing locked reference when appropriate.
- Keep Rust unit tests in separate `*_tests.rs` files or `mod_test.rs`, included through a test module at the end of the implementation file.
- Feature flags live in `crates/warp_features/src/lib.rs`; compile-time feature declarations live in `app/Cargo.toml`, with registration in `app/src/lib.rs`. Add gates when they serve the requested change.
- Update directly affected docs and examples with implementation changes.

## Build and validation

Use the pinned toolchain in `rust-toolchain.toml`.

- `./script/bootstrap`: platform setup.
- `./script/run`: build and run; creates an app bundle on macOS.
- `cargo check -p warp --features gui --all-targets`: compile the app and test targets.
- `cargo fmt --all -- --check`: Rust formatting.
- `cargo nextest run -p warp -E 'test(<name>)'`: focused app tests.
- `./script/presubmit`: optional full local formatting, lint, and test suite.
- `./script/warper_offline_local_smoke`: local-first smoke checks.
- `./script/check_warper_static_denylist all-runtime`: checks the removed hosted-product surfaces.
- `rumdl check <file>`: changed Markdown files.

Run checks that exercise the changed behavior, including compilation, relevant tests, and applicable formatting and lint checks. Use CI for platform-specific validation. Report the checks run and their results.

## Specs and collaboration

- Create specs or design documents when the user requests or approves them.
- Keep product requirements current as implementation changes. Use `specs/README.md` to identify current requirements.
- Use neutral branch names such as `fix/...` and `feat/...`.
- Write the PR title as the squash commit subject and the description as its body. Describe the problem, resulting behavior, and relevant validation. Use that title and description when squash-merging.
