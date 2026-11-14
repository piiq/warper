# WARPER-016: remove diff review

## Summary

Remove the built-in diff review feature, including its top toolbar indicator and diff viewer/editor. Keep the terminal input's diff statistics chip as passive repository information.

This spec supersedes the diff review requirements in earlier Warper specs, including [the archived WARPER-012](../archive/WARPER-012/PRODUCT.md) and the requirement to preserve built-in diff inspection in WARPER-015. File editing outside diff review remains supported.

## Behavior

1. The top toolbar contains no diff review button, diff icon, or associated additions/deletions counts, regardless of whether the active repository has changes.

2. Removing the toolbar control leaves no empty slot or reserved spacing. It is also absent from toolbar customization.

3. The built-in diff viewer/editor is unavailable in every presentation, including panels, panes, tabs, and maximized views.

4. All entry points into diff review are removed, including menu items, command palette actions, keyboard shortcut entries, welcome actions, and clickable diff badges.

5. No replacement viewer, external application launch, redirect, or fallback is introduced.

6. The terminal input's diff statistics chip remains available under its existing visibility and customization rules. Its file count, additions, deletions, formatting, placement, and refresh behavior are preserved.

7. The retained chip is passive. Clicking it does not open anything or change focus. It has no review-specific hover treatment, pointing-hand cursor, tooltip, or shortcut hint.

8. The chip continues to reflect the repository associated with its terminal. Switching directories, repositories, tabs, or panes preserves its existing update behavior. Its existing behavior for clean repositories, unavailable statistics, and non-repository directories is preserved.

9. Existing diff statistics outside the top toolbar and terminal input retain their current placement and visibility settings, but any action that opens diff review is removed.

10. Review-specific interactions disappear with the viewer, including diff navigation, comparison selection, review comments, copying or sending review comments, and editing or Git operations offered through that surface.

11. Settings for the removed feature disappear, including controls for showing the review button and its toolbar statistics. Shared editor settings remain available where they still affect supported editing features.

12. Previously saved layouts and preferences cannot restore the removed viewer or toolbar control. Restoring a workspace leaves no blank review panel or reserved review space, and preserves its terminal sessions.

13. Any remaining externally invocable request to open diff review fails immediately with a clear message that the feature has been removed.

14. Terminal commands such as `git diff`, branch and repository chips, pull request links, and file editing outside diff review retain their existing behavior.
