---
name: write-tech-spec
description: Write a Warper technical specification when the user asks for an implementation plan or architecture document tied to the current codebase.
---

# Technical specifications

Read `AGENTS.md`, the relevant product requirements, and the implementation before proposing changes. Cite the current files and interfaces that constrain the design. Historical documents under `specs/archive/` and `specs/upstream/` are background, not current requirements.

## Structure

Use these sections:

1. Context: describe the problem, the existing implementation, and relevant files and interfaces. Reference product requirements for user-visible behavior.
2. Proposed changes: identify modules, types, APIs, data flow, state transitions, and ownership that will change. Explain material tradeoffs using the code's existing patterns.
3. Testing and validation: map numbered product behavior requirements to concrete tests or verification steps. Cover affected terminal workflows and state which checks require manual use or visual inspection.

Add a flow, diagram, risks, unresolved questions, or work split when it helps explain the implementation. Omit empty sections and repeated material.

Reference product behavior without duplicating it. Scale the plan to the work; a straightforward change does not need a technical spec.

When the user requests a technical spec file, write `TECH.md` beside the associated `PRODUCT.md`, or use an agreed directory under `specs/`. Tickets, mocks, and external services are optional inputs supplied by the user. A product spec does not require a separate technical spec.

Update the document when implementation changes the design. Run `rumdl check` on changed Markdown files.
