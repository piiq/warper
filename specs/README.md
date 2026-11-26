# Warper specifications

Warper is maintained for `piiq`'s everyday terminal use. Current product direction comes from the maintainer's requests, [README.md](../README.md), and the applicable specifications below.

## Current requirements

- [WARPER-001](WARPER-001/PRODUCT.md): local terminal operation without Warp-hosted services.
- [WARPER-002](WARPER-002/PRODUCT.md): persistent macOS launch permissions.
- [WARPER-003](WARPER-003/PRODUCT.md): local support and application identity.
- [WARPER-004](WARPER-004/PRODUCT.md): application branding.
- [WARPER-006](WARPER-006/PRODUCT.md), [WARPER-007](WARPER-007/PRODUCT.md), and [WARPER-011](WARPER-011/PRODUCT.md): retained security, terminal crash prevention, save, and packaging behavior from earlier port audits. Their old candidate-selection rules are historical.
- [WARPER-013](WARPER-013/TECH.md): remove the browser WASM target.
- [WARPER-014](WARPER-014/PRODUCT.md): alternate-screen background behavior.
- [WARPER-015](WARPER-015/PRODUCT.md): remove built-in AI while retaining third-party CLI-agent integration.
- [WARPER-016](WARPER-016/PRODUCT.md): remove built-in diff review while retaining passive terminal diff statistics.
- [WARPER-017](WARPER-017/PRODUCT.md): remove editor, explorer, notebooks, file search, workflows, environment collections, and launch configs; retain tab configs and external file opening.

Specifications are numbered in chronological order. Later specifications take precedence where requirements conflict. Keep earlier specifications unchanged when adding new requirements. Historical specifications do not authorize restoring removed features.

## Historical material

- `archive/WARPER-001/`: the original network-removal implementation plan and obsolete OpenRouter verification checklist.
- `archive/WARPER-005/`: the removed OpenRouter agent design.
- `archive/WARPER-009/`: the removed OpenRouter file-edit workflow's port plan.
- `archive/WARPER-012/`: the removed review-comments clipboard feature.
- `upstream/`: imported upstream designs and previous porting audits. They describe earlier source revisions and product assumptions, not current Warper commitments.

Keep new specifications under an agreed `specs/<id>/` directory. Product specs describe observable behavior; technical specs describe implementation and verification when needed.
