---
name: warp-ui-guidelines
description: Follow Warper UI conventions when adding or changing client UI code. Read before implementing UI changes.
---

# Warper UI guidelines

Reuse the app's shared components and theme accessors when implementing UI changes.

## Reuse button themes

Button colors come from shared `ActionButtonTheme` implementations in `app/src/view_components/action_button.rs` and parallel `Theme` implementations in `crates/ui_components/src/button/themes.rs`. Themes include `PrimaryTheme`, `SecondaryTheme`, `NakedTheme`, and `DangerPrimaryTheme`.

Use an existing theme unchanged when styling a button. Check the available themes before adding overrides.

Changing a shared theme affects every button that uses it. Edit a shared theme when the user has explicitly confirmed that the shared component itself needs to change.

Avoid these styling patterns:

- Adding a theme such as `FooPrimaryTheme` that delegates to `PrimaryTheme` and overrides a single method such as `text_color`.
- Hard-coding `ColorU::new(...)` instead of using `appearance.theme()` accessors (`accent`, `font_color(bg)`, `foreground`, etc.).
- Setting `should_opt_out_of_contrast_adjustment` to `true` to force a specific label color.
- Naming a theme after a feature or view (`FooPrimaryTheme`, `BarSubmitTheme`) rather than a design-system role.
