---
name: write-product-spec
description: Write a Warper product specification with detailed behavior and testable requirements when the user asks for a product spec, PRD, desired behavior document, or PRODUCT.md file.
---

# Product specifications

Write the desired behavior clearly enough for implementation and review. Describe what the consumer sees, does, and can rely on. For app features, the consumer is the person using Warper. For APIs, protocols, or CLI tools, describe the surface from the caller's perspective.

## Before writing

Read `AGENTS.md`, `README.md`, relevant existing specs, and the code that determines current behavior. Use `specs/README.md` to distinguish current requirements from archived plans.

Establish the requested outcome, affected users or callers, existing behavior, and scope. Use the conversation and code to resolve these where possible. Ask about missing product decisions that materially affect behavior. Keep unresolved questions next to the requirements they affect.

Use design references supplied by the user. Ask for additional design information when a behavior cannot be specified from the available context. Omit sections for references that were not supplied.

## File placement

When the user requests a file, follow an existing spec directory or use `specs/<feature-name>/PRODUCT.md`. Use an issue or WARPER number when one has been supplied or agreed. When the user asks what a spec would look like, draft it in the conversation.

## Structure

Use `Summary` and `Behavior` as the core sections:

- `Summary`: one to three sentences stating the feature or change and its desired outcome.
- `Behavior`: numbered, observable requirements describing the complete requested behavior. Group related requirements under subheadings for changes with several flows or states.

Add sections when they provide information beyond those requirements:

- `Problem`: the current limitation and its effect when the motivation needs explanation.
- `Scope`: boundaries or non-goals that resolve a real ambiguity.
- `References`: supplied designs, issues, or other sources that clarify behavior.
- `Validation`: acceptance scenarios and visible results that help a reviewer verify the requirements. Detailed test commands and implementation test plans belong in a technical spec when one is needed.
- `Open questions`: several unresolved product decisions worth collecting together. A single question can remain beside its requirement.

Omit empty sections. Keep framing short and avoid repeating requirements under several headings. A small change may need a handful of requirements. A complex feature needs enough detail to cover its actual interactions.

## Behavior requirements

Write numbered statements that can be checked through observable results. Describe the trigger, relevant state, user action, and expected result. Distinguish current behavior from the intended change when that distinction helps implementation.

Cover the behavior relevant to the feature:

- Defaults, entry points, and the normal user flow.
- User-visible states and transitions, including empty, pending, error, and cancelled states that the feature can encounter.
- Accepted inputs and responses to invalid inputs.
- Keyboard actions, focus, accessibility, and interactions with adjacent features.
- Persistence, restart, multiple panes or windows, permissions, offline operation, or concurrent changes where the requested behavior depends on them.
- Existing behavior and user data that the change must preserve.

For a feature removal, specify which entry points, controls, settings, and commands disappear, and which related behavior remains. Define the treatment of existing saved state when it affects the user. Do not invent replacement behavior.

Keep requirements grounded in the request and actual code. Include edge cases that affect the feature rather than generating a generic checklist. Describe public input/output formats when they are part of the product. Leave internal types, module boundaries, algorithms, and data flow to implementation planning.

## Example

This example shows the expected structure for removing a review panel while retaining terminal metadata:

```markdown
# Remove diff review

## Summary

Remove the diff review panel and its entry points. Retain the repository diff chip in the terminal input.

## Behavior

1. In a repository with uncommitted changes, the top panel shows no diff review indicator or action.
2. The app provides no viewer or editor for reviewing the diff.
3. The terminal input retains its diff chip and the existing added/deleted line counts.
4. The diff chip provides passive metadata and cannot open a review panel.

## Validation

Open a repository with uncommitted changes. Confirm the top panel has no review control, the terminal input shows the diff chip, and interacting with the chip does not open a viewer.
```

## Keep the spec current

A product spec does not require a companion technical spec. Use `write-tech-spec` when the user requests an implementation plan or the change needs a separate architecture discussion.

Update the product spec when implementation changes the agreed user-facing behavior. The checked-in spec should describe what ships. Run `rumdl check` on changed Markdown files.
