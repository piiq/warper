# WARPER-018: move tab to new window

## Summary

Add a “Move Tab to New Window” context menu item that moves an existing tab and all its panes into a separate window. Running terminal sessions continue in the new window.

## Behavior

1. The tab context menu offers “Move Tab to New Window” when the source window contains at least two tabs. Place it with the existing tab movement items.
2. The item is available in both horizontal and vertical tab layouts. In vertical tabs, it appears in the group header and pane row context menus alongside the existing tab actions. Invoking it from a pane row moves the entire containing tab.
3. The item is absent when the source window has one tab. Moving a tab does not close the source window.
4. Selecting the item moves the tab whose context menu was opened, including when that tab is inactive. It creates one new window containing that tab, with no additional empty tab or terminal pane.
5. All panes move together, preserving their split arrangement, relative sizes, pane names, and active pane. A maximized pane retains its maximized state.
6. Terminal shells and running commands continue without restarting or receiving a close request. Working directories, command blocks, scrollback, pending input, and alternate-screen application state are preserved. Moving the tab does not display a close or running-process confirmation.
7. The moved tab retains its custom name and displayed color. The new window uses the current tab layout preference and inherits the source window's vertical tabs panel visibility.
8. The new window opens visibly on the source window's display, using the source window's size subject to available screen space. It becomes the active window and gives keyboard focus to the moved tab's previously active pane, ready for typing.
9. The source window preserves the order and contents of its remaining tabs. Moving an inactive tab preserves the source window's selected tab. Moving the active tab selects its preceding tab, or the first remaining tab when there is no preceding tab.
10. The context menu closes after selection. The item supports the context menu's existing keyboard navigation and activation. Dismissing the menu without selecting the item leaves the tab in place.
11. Moving a tab is not recorded as a tab closure. Reopening a closed tab does not duplicate the moved tab, and closing the source window afterward does not terminate sessions in the new window. Closed Settings panes retained for Undo are discarded when their tab moves; closed terminal panes remain restorable.
12. With session restoration enabled, a normal quit and restart restores the resulting window and tab arrangement under Warper's existing restoration rules. The moved tab appears in the destination window without a duplicate in the source window. This does not change how running processes survive an application restart.
13. If the move cannot complete, Warper displays an error explaining that the tab could not be moved. The tab and its live contents remain usable in the source window; no successful move is reported.

## Scope

This change adds the context menu item. Dragging tabs out of windows, moving tabs into existing windows, extracting individual panes, moving the final tab, new keyboard shortcuts, command palette entries, and new settings are outside this change. The action works locally without an account or network connection.

## Validation

- In each tab layout, move an inactive named and colored tab from a window with three tabs. Confirm that exactly one destination window opens, the tab retains its name and color, and the source preserves its selected tab and remaining order.
- Move the active first tab and then an active tab with a preceding neighbor. Confirm that the source selects the first remaining tab and the preceding tab respectively. With one tab remaining, confirm that the menu item is absent.
- Move a split tab containing a running command and an alternate-screen application. Confirm that processes continue, terminal state is retained, split geometry is preserved, and typing reaches the previously active pane. Repeat with a maximized pane.
- In vertical tabs, invoke the action from a pane row and confirm that the whole group moves. Confirm inherited panel visibility. Use keyboard menu activation and separately dismiss the menu without selecting an item.
- Close the source window after a move and confirm that the destination remains usable. Confirm that reopening a closed tab does not recreate the moved tab. Quit and restart with session restoration enabled and verify the resulting window arrangement without duplicated tabs.
- Close the destination with the native window close button. Confirm that it closes and the source window remains usable.
- Move a tab during an SSH file upload. Confirm that upload progress, password prompts, and cancel controls work in the destination after closing the source window.
- Move a tab containing closed Settings and terminal panes retained for Undo. Confirm that Undo restores the closed terminal pane without consuming a stale Settings entry.
- Exercise a failed move and confirm that an error is visible and the original tab remains usable in the source window.
