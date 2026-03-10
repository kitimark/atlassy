# Sandbox Page Inventory

Page inventory for KPI experiment sandbox pages. All pages are children of the primary sandbox page (`131207`).

## Pages

| Page | ID | Parent | Title | Headings | Table | Locked Blocks | Size Estimate |
|------|----|--------|-------|----------|-------|---------------|---------------|
| P1 | 1212417 | 131207 | KPI Experiment - Prose Rich v6 20260310T210423Z | Introduction, Details, Summary | No | No | ~1000-1500 ADF bytes |
| P2 | 1245185 | 131207 | KPI Experiment - Mixed Prose Table v6 20260310T210423Z | Overview, Data | Yes (3x3 text cells) | No | ~1500-2200 ADF bytes |
| P3 | 1245199 | 131207 | KPI Experiment - Locked Adjacent v6 20260310T210423Z | Context, Notes, References | No | Yes (expand, panel, rule) | ~900-1200 ADF bytes |

## Route Classification Expected

| Page | `editable_prose` | `table_adf` | `locked_structural` |
|------|-------------------|-------------|---------------------|
| P1 | heading, paragraph, bulletList, orderedList, blockquote | None | None |
| P2 | heading, paragraph | table, tableRow, tableCell | None |
| P3 | heading, paragraph | None | expand, panel, rule |

## Content Design

### P1: Prose-Rich (Pattern A)

- Heading "Introduction" + 2-3 paragraphs.
- Heading "Details" + bullet list + blockquote.
- Heading "Summary" + closing paragraph.
- No tables, no macros, no media.

### P2: Mixed Prose+Table (Pattern B)

- Heading "Overview" + introductory paragraph.
- Heading "Data" + paragraph + 3x3 table with text cells + paragraph after table.
- Table must have simple text content in all cells (for `target_path` targeting).

### P3: Locked-Adjacent (Pattern C)

- Heading "Context" + introductory paragraph.
- Expand block with hidden paragraph content.
- Heading "Notes" + paragraph adjacent to the locked expand block.
- Info panel block plus horizontal rule as additional locked structural blocks.
- Heading "References" + closing paragraph.

## Heading Naming Rules

- Heading selectors are exact-match (`heading:<text>` must equal the heading text exactly).
- Keep headings on a page distinct to avoid ambiguous intent in operator workflows.
- Good: "Introduction", "Details", "Summary".
- Avoid accidental drift: if a heading changes, update manifests using that selector.

## Target Paths

> **Note**: Auto-discovery is implemented. Explicit `target_path` is optional for route-specific modes; paths below are reference baselines for validating discovery output.

Discovered during Phase 2 scoped fetch spike on 2026-03-10 at commit `217a942383a6c8784d4c20b65377e990a1db0422`.

| Page | Prose Target Path | Table Cell Target Path | Notes |
|------|-------------------|----------------------|-------|
| P1 | `/content/1/content/0/text` (Introduction), `/content/5/content/0/content/0/text` (Details blockquote), `/content/7/content/0/text` (Summary) | N/A | Headings at `/content/0`, `/content/3`, `/content/6`; bullet list at `/content/4` |
| P2 | `/content/1/content/0/text` (Overview), `/content/3/content/0/text` (Data prose), `/content/5/content/0/text` (post-table prose) | `/content/4/content/0/content/0/content/0/content/0/text` (R1C1) | Table root at `/content/4` |
| P3 | `/content/1/content/0/text` (Context), `/content/4/content/0/text` (Notes), `/content/7/content/0/text` (References) | N/A | Expand at `/content/2`, panel at `/content/5`, rule at `/content/8` |

## Scoped Fetch Spike Results (current v6 pages)

| Page | Full ADF Bytes | Scoped ADF Bytes | Context Reduction | Scope Resolution |
|------|---------------|-----------------|-------------------|------------------|
| P1 | 1233 | 321 (`heading:Introduction`) | 74.0% | OK |
| P2 | 1901 | 1662 (`heading:Data`) | 12.6% | OK |
| P3 | 1007 | 363 (`heading:Notes`) | 64.0% | OK |

## Notes

- Current page IDs correspond to v6 revalidation created on 2026-03-10.
- Target paths are discovered during Phase 2 spike via `jq` inspection of fetch state output.
- Bootstrap injects a heading + paragraph scaffold; pages are then seeded with pattern-specific ADF content.
- `panel` wrappers are `locked_structural` but inner paragraphs are `editable_prose`.
- The route classifier uses a catch-all: anything not in the 7-type prose whitelist or table family is `locked_structural`.
