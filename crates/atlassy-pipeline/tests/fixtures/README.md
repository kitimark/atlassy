# Fixture Notes

- `prose_only_adf.json`: baseline prose-only fixture for deterministic mapping and prose update checks.
- `mixed_routes_adf.json`: mixed prose/table/locked fixture for route-isolation assertions.
- `table_allowed_cell_update_adf.json`: table-focused fixture for allowed `cell_text_update` integration behavior.
- `table_forbidden_ops_adf.json`: table-focused fixture used for forbidden shape/attribute operation checks.

Formatting fidelity assertions use semantic checks for prose text content in mapped paths.
Non-prose route assertions use strict path-level unchanged checks.
Table-route assertions enforce text-only table cell path updates and reject shape/attribute drift.
