# Contributing to Warper

[piiq](https://github.com/piiq) maintains Warper as an everyday terminal. The [README](README.md) describes its scope. Report bugs and propose changes in [this repository](https://github.com/piiq/warper/issues).

For bugs, include steps to reproduce, expected and actual behavior, your OS, the build you used, and relevant logs. For feature requests, describe the problem you want to solve.

## Changes

Keep changes focused, explain their effect on terminal behavior, and include relevant validation. Agree on a spec with the maintainer when the behavior needs clarification.

Use the default branch as the PR base. Follow [AGENTS.md](AGENTS.md) for repository conventions and build commands. Run `cargo fmt` and the build, lint, and test checks relevant to the change. `./script/presubmit` is an optional full local suite. CI provides macOS, Linux, and Windows compilation and lint coverage. Report the checks run and their results.

Write the PR title as the squash commit subject and the description as its body. Include the problem, resulting behavior, and relevant validation. Use that title and description when squash-merging.

## Reports

- Report Warper issues to [piiq/warper](https://github.com/piiq/warper/issues).
- Follow [SECURITY.md](SECURITY.md) for private security reports.
- Follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) when participating in the project.
