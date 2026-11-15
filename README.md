<!-- rumdl-disable MD033 MD041 -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="public/warper_banner_dark.jpg">
    <source media="(prefers-color-scheme: light)" srcset="public/warper_banner_light.jpg">
    <img src="public/warper_banner_light.jpg" alt="Warper" width="90%" />
  </picture>
</p>

> **warper** /ˈwɔːpə(r)/ *adj.* — comparative form of *warp*; more warp than the thing being compared to.
> 
> **example:** *"I just wanted a terminal. But warper."*
> 
> **antonym:** *a terminal that asks for your email before it lets you `cd`.*

<p align="center">
  <img src="public/warper_screenshot.png" alt="Warper terminal" width="100%" />
</p>

<p align="center"><sub>Warper is a hard fork of <a href="https://www.warp.dev">Warp</a>.</sub></p>

---

## Why this exists

Warper keeps Warp's terminal and third-party CLI-agent integration. Built-in AI and Warp-hosted services have been removed.

[piiq](https://github.com/piiq) maintains Warper as an everyday terminal. [This repository](https://github.com/piiq/warper) is the home for development, documentation, and support.

---

## What's in

### Terminal

Native Rust app with blocks, panes, tabs, session restoration, command history, shell integration, command palette, themes, keybindings, launch configs, local workflows, and a background image picker.

### Third-party CLI agents

Run `codex`, `claude`, `gemini`, or another supported CLI agent in the terminal. Warper adds a status indicator, optional rich input, and a finish notification.

---

## What's out

If a feature needs an account, a server, a billing entitlement, a team object, a shared session, or any Warp.dev infrastructure, it's out of scope. Specifically removed:

- Built-in AI, Agent Mode, model providers, API-key management, MCP servers, skills discovery, AI indexing, and agent orchestration
- Built-in diff review, its viewer/editor, review comments, and toolbar indicator. Passive Git statistics remain in terminal input.
- Accounts, sign-up, sign-in, SSO, anonymous cloud users, subscriptions
- Billing, plans, credits, usage limits, upgrade prompts
- Warp Drive, team drives, shared-with-me, cloud trash
- Teams, enterprise admin, ACLs, roles, audit, hosted workspaces
- Session sharing, remote control, RTC presence, link collaboration
- Oz, cloud agents, and hosted task orchestration
- Telemetry upload, hosted crash reporting, RudderStack, Sentry, Warp-hosted autoupdate

---

## Local data

Warper settings and terminal state live on disk. Third-party CLI agents manage their own credentials and data. On macOS:

| Thing | Location |
|---|---|
| User settings | `~/.warp-oss/settings.toml` |
| Runtime state, cache, DB | `~/Library/Application Support/dev.warper.Warper/` |
| Logs | `~/Library/Logs/warper.log` |

---

## Building locally

```bash
./script/bootstrap
./script/run
```

Useful checks:

```bash
./script/presubmit
./script/warper_offline_local_smoke
./script/check_warper_static_denylist all-runtime
```

On macOS, `./script/run` produces `target/debug/bundle/osx/Warper.app`. Development instructions live in [AGENTS.md](AGENTS.md).

---

## Releases

Pushing a `warper-vX.Y.Z` tag runs the macOS release workflow. The tag must match the version in `app/Cargo.toml`, with `Cargo.lock` updated in the same commit.

The workflow builds an optimized Apple Silicon (arm64) app and attaches its DMG to a draft [GitHub Release](https://github.com/piiq/warper/releases). Review the draft's notes and publish it on GitHub. The app uses ad-hoc signing and is not Apple-notarized. Apple credentials are not required. Windows, Linux, and Intel Mac release packages are not built.

The workflow can also be rerun manually with an existing release tag. Reruns update draft assets. Published release assets require a new version.

---

## Upstream, license, and credits

This project is independent from Warp.dev. The names "Warp" and "Warp.dev", the related marks, and the hosted services belong to their respective owners. Warper exists because the Warp team released enough of their code to make a fork like this possible. Please don't report Warper-specific bugs against upstream Warp unless they also reproduce there.

The upstream licensing structure is preserved:

- `warpui_core` and `warpui` are MIT-licensed. See [LICENSE-MIT](LICENSE-MIT).
- Everything else in this repository is AGPL v3. See [LICENSE-AGPL](LICENSE-AGPL).

Downstream users and contributors should preserve upstream notices and comply with the applicable licenses.

Open-source dependencies inherited from Warp include:

- [Tokio](https://github.com/tokio-rs/tokio)
- [NuShell](https://github.com/nushell/nushell)
- [Fig Completion Specs](https://github.com/withfig/autocomplete)
- [Warp Server Framework](https://github.com/seanmonstar/warp)
- [Alacritty](https://github.com/alacritty/alacritty)
- [Hyper](https://github.com/hyperium/hyper)
- [FontKit](https://github.com/servo/font-kit)
- [Core Foundation Rust bindings](https://github.com/servo/core-foundation-rs)
- [Smol](https://github.com/smol-rs/smol)

Thanks to all of them.

---
