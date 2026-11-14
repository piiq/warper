# FAQ

## Do I need a Warp account?

No. Warper has no account system. There is no sign-in, no sign-up, no anonymous cloud user.

## Where does my data live?

See the [Local data section of the README](README.md#local-data) for storage paths.

## Does Warper have a built-in AI agent?

No. Run a third-party CLI agent such as Codex, Claude Code, or Gemini CLI in the terminal. Configure models, credentials, tools, and skills in that agent.

## Is this an official Warp.dev build?

No. Warper is an independent hard fork. It uses Warp's open-source code under its existing licenses (AGPL v3 for most of the repo, MIT for `warpui_core` and `warpui`). It is not affiliated with, endorsed by, or sponsored by Warp.dev.

## Where do I report bugs?

Report bugs in [this repository's GitHub issues](https://github.com/piiq/warper/issues).

## Does Warper send anything over the network?

Warper has no model-provider integration, telemetry upload, hosted crash reporting, remote config fetch, Drive sync, or autoupdate ping. Programs you run in the terminal, including third-party CLI agents, make their own network requests.

## Can I use Codex, Claude Code, or Gemini CLI inside Warper?

Yes. Run `codex`, `claude`, or `gemini` in the terminal. Warper adds a status indicator, optional rich input, and a finish notification.

## Why is the binary named `warp-oss`?

The executable name is inherited from upstream. The app ships as `Warper.app` with bundle ID `dev.warper.Warper`.

## Will the server, account system, or Drive ever come back?

No. Warper has removed these features. They are available in upstream [Warp](https://www.warp.dev).

## Can I run Warper on Linux or Windows?

The code and bundling scripts cover macOS, Linux, and Windows. macOS is the primary development platform. Report platform-specific problems in this repository.
