# WARPER-017: remove built-in editor and unused workspace features

## Summary

Keep Warper focused on terminal sessions and third-party CLI-agent integration. Remove the built-in file editor, file explorer, notebooks, repository file search, workflows, named environment-variable collections, and launch configs. Retain tab configs and use external editors for file-opening actions.

## Behavior

### Removed features and entry points

1. Warper provides no built-in file editor or file explorer. Code and text editor panes, editor tabs, preview tabs, file trees, and file-tree operations such as creating, renaming, moving, and deleting files are removed.
2. Notebook and Markdown file panes are removed, including rendered previews, raw/rendered switching, notebook editing, notebook command blocks, and notebook-specific navigation. Markdown formatting used by retained terminal UI remains available.
3. Repository filename quick-open and search across file contents are removed. Their palette filters, results, sidebar controls, shortcuts, and search settings disappear. Terminal output search, command-history search, tab/session navigation, and action search remain available.
4. The editor/explorer/search sidebar disappears, including its open/close controls, resizing affordances, keyboard focus targets, and saved visibility settings. Removing it leaves no empty panel, reserved space, or button that does nothing. Horizontal and vertical terminal tab layouts remain available.
5. Built-in language-server support is removed, including server discovery, installation, startup, management, logs UI, diagnostics, completion, formatting, references, and definition navigation. Starting Warper, opening a terminal, or switching repositories does not start a Warper-managed language server.
6. Saved-command workflows are removed, including user workflows, project workflows, bundled command recipes, workflow aliases, parameter-entry UI, workflow suggestions, and “Save as Workflow” actions on terminal input and command blocks. Ordinary shell aliases, shell scripts, command history, and terminal completions remain supported.
7. Named environment-variable collections are removed, including their creation and application actions, workflow integration, collection blocks, and collection-specific secret-provider controls. Ordinary shell environment variables, inherited process environments, and environment configuration used by retained terminal features remain supported.
8. Launch configs are removed as a product feature. Saving or loading named layouts spanning windows and tabs, launch-config palette entries, launch-config editing actions, and launch-config settings disappear. Automatic session restoration remains supported and is distinct from launch configs.
9. Standalone project/repository quick-open and editor-oriented project onboarding are removed. Repository selection used by tab configs and Git worktree helpers remains available.
10. Every entry point dedicated to a removed feature disappears from menus, context menus, toolbars, toolbar customization, welcome screens, command palettes, shortcut settings, tooltips, and accessibility actions. “New File,” “Open in Warper,” and internal “Open in New Tab/Pane” file actions disappear. Dragging a file onto terminal input continues to insert its path using the existing terminal behavior.
11. Settings dedicated to the removed features disappear, including the built-in-editor choice, internal file-opening layout, tabbed editor preference, Markdown viewer preference, formatting settings, and explorer/search visibility. The remaining external-editor preference stays accessible without a page advertising removed editor or language-server functionality.
12. Removed features have no invocable requests left. Remove their registered actions, CLI commands and arguments, URI and IPC routes, file-opening registrations, and dispatch handlers wherever those interfaces exist. Old requests are rejected through the normal unknown-command, invalid-argument, or unknown-action behavior; do not retain feature-specific rejection handlers or compatibility endpoints. Requests cannot silently succeed, create empty views, or execute a substitute terminal command. Normal file-opening actions follow the external-opening behavior below.

### Retained tab configs and file opening

13. Tab configs remain available under their existing names and storage paths. Creating, saving, editing, selecting, loading, and deleting tab configs continue to work. Existing terminal pane layouts, directories, shell choices, startup commands, parameters, tab names, and colors keep their existing behavior and file format.
14. Selecting a tab config that declares a `repo` or `branch` parameter opens the existing parameter dialog with repository or branch dropdowns. Repository selection updates the available branches. The repository dropdown retains its “+ Add new repo...” folder-picker action. The tab-config menu retains “New worktree config,” its searchable repository submenu and “+ Add new repo” action, and the flow that creates a worktree tab config and opens a terminal in the new worktree. Removing standalone project quick-open must not remove the repository information or selection controls required by these user-facing flows.
15. Editing or creating a tab-config file opens it in the user's configured external editor. Existing configuration-error actions that open the offending file also use that editor. Existing external-editor selections remain effective, including the environment-editor option; the built-in editor is no longer a selectable target. With no external editor selected, Warper uses the operating system's default application.
16. Normal terminal file links and retained “Open file” actions open files externally, including Markdown files. Existing line and column information is passed to external editors that support it. The user receives a visible error when an external-opening action fails.
17. “Reveal in Finder” and equivalent system file-manager actions remain available. Removing the built-in explorer does not remove operating-system file-manager integration.

### Terminal behavior and saved state

18. Terminal sessions, shell integration, command execution, blocks, scrollback, command history, completions, SSH, tabs, split panes, session restoration, themes, backgrounds, keybindings, and terminal settings retain their existing behavior.
19. Third-party CLI agents continue to run as terminal programs. Their detection, status, rich input, finish notifications, and toolbar configuration remain supported. Removing Warper's file tools does not restrict the tools or file access of those programs.
20. Git branch selection, worktree support, and passive repository statistics remain available. Filesystem access required by terminal behavior, tab configs, configuration loading, or Git information remains supported.
21. Quit and close warnings continue to protect running terminal commands. Editor-specific unsaved-change warnings and save/discard controls disappear with the editor.
22. There is no requirement to restore, convert, migrate, export, or recover saved editor panes, notebook panes, workflows, environment collections, launch configs, or settings dedicated to the removed features. Drop support for those saved objects without adding replacement panes or a compatibility workflow. Do not reset the terminal database or user settings to accomplish the removal.
23. Existing terminal history, retained settings, tab-config files, terminal session state, and storage paths remain intact. User-authored files, repositories, shell configuration, and third-party CLI-agent data are not deleted. Removing a product feature does not authorize deleting the user's workflow or launch-config files from disk.

## Scope

Remove the implementations, background services, settings, feature flags, and dependencies that exclusively support the removed features. Hiding controls or retaining permanent no-op handlers does not satisfy this removal. Shared text input, rendering, filesystem, and repository functionality needed by retained terminal features stays under those features.

Tab configs and automatic session restoration remain part of the terminal. The removed launch-config feature is the separate system for named configurations containing multiple windows and tabs. Internal code shared with tab configs must remain usable without preserving launch configs as a product feature.

Preservation requirements apply to the reachable tab-config and worktree flows described above. Unreachable legacy actions and dialogs do not need to be retained or exposed as new UI.

Reconcile directly affected product documentation, examples, and development instructions with this scope during implementation. Keep historical specifications as historical records. No redesign of terminal input, tab layouts, shell integration, or third-party CLI-agent flows is included.

## Validation

- Inspect horizontal and vertical tab layouts, menus, context menus, toolbar customization, shortcut settings, palettes, and settings. Confirm that none exposes the removed features and that the former sidebar leaves no reserved space or focus target.
- Create a tab config from a split terminal tab, edit its file externally, and reopen it. Confirm the existing layout, directories, shell choices, startup commands, names, and colors.
- Open a tab config declaring `repo` and `branch` parameters from the tab-config menu. Confirm that the parameter dialog renders both dropdowns, repository selection refreshes the branches, and “+ Add new repo...” opens a folder picker and selects the chosen repository.
- Open the tab-config menu in each tab layout and choose “New worktree config.” Search and select a repository, and separately exercise “+ Add new repo.” Confirm that the flow saves a tab config, creates a Git worktree, and opens its terminal in the new worktree.
- Open a malformed tab config's file through its error action. Confirm that it opens externally and remains editable. Confirm visible failure reporting when an external editor cannot be launched.
- Open text, code, and Markdown file links from terminal output. Confirm external opening and supported line/column navigation. Reveal a file in the operating-system file manager and drag a file into terminal input.
- Search terminal output and command history, navigate tabs/sessions, and invoke terminal actions through action search. Confirm that file-search and workflow results are absent.
- Run a long-running command, an alternate-screen application, an SSH session, and a third-party CLI agent. Confirm terminal input, completions, CLI-agent rich input and status, Git information, and running-command close warnings.
- Quit and restart a terminal-only workspace with session restoration enabled. Confirm retained tabs, panes, directories, and history. Confirm that startup and repository changes do not launch language servers or initialize the removed feature services.
- Inspect action registrations, CLI help and argument parsing, URI and IPC routing, file-opening registrations, and dispatch handlers. Confirm that removed features have no invocable entry points or compatibility handlers. Old requests receive the normal unsupported-request error.
