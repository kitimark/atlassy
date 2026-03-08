# Sandbox Page Inventory

Page inventory for KPI experiment sandbox pages. All pages are children of the primary sandbox page (`131207`).

## Pages

| Page | ID | Parent | Title | Headings | Table | Locked Blocks | Size Estimate |
|------|----|--------|-------|----------|-------|---------------|---------------|
| P1 | 131373 | 131207 | KPI Experiment - Prose Rich v4 2026-03-08 | Introduction, Details, Summary | No | No | ~500-1000 ADF bytes |
| P2 | 131387 | 131207 | KPI Experiment - Mixed Prose Table v4 2026-03-08 | Overview, Data | Yes (3x3 text cells) | No | ~1000-2000 ADF bytes |
| P3 | 327877 | 131207 | KPI Experiment - Locked Adjacent v4 2026-03-08 | Context, Notes, References | No | Yes (expand, panel, rule) | ~800-1500 ADF bytes |

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
- Expand macro (type `/expand` in Confluence editor) with hidden content inside.
- Heading "Notes" + paragraph adjacent to the locked expand block.
- Image attachment or Jira macro (type `/image` or `/jira`) as a second locked block.
- Heading "References" + closing paragraph.

## Heading Naming Rules

- Heading selectors are exact-match (`heading:<text>` must equal the heading text exactly).
- Keep headings on a page distinct to avoid ambiguous intent in operator workflows.
- Good: "Introduction", "Details", "Summary".
- Avoid accidental drift: if a heading changes, update manifests using that selector.

## Target Paths

> **Note**: Auto-discovery is implemented. Explicit `target_path` is optional for route-specific modes; paths below are reference baselines for validating discovery output.

Discovered during Phase 2 scoped fetch spike on 2026-03-08 at commit `18b7c633bf8a3ceae9185e19f4806ba1a61f06db`.

| Page | Prose Target Path | Table Cell Target Path | Notes |
|------|-------------------|----------------------|-------|
| P1 | `/content/1/content/0/text` (Introduction), `/content/5/content/0/text` (Details), `/content/9/content/0/text` (Summary) | N/A | All target prose nodes under editable prose routes |
| P2 | `/content/1/content/0/text` (Overview), `/content/3/content/0/text` (Data prose) | `/content/4/content/0/content/0/content/0/content/0/text` (R1C1) | Table root at `/content/4`, post-table prose at `/content/5/content/0/text` |
| P3 | `/content/1/content/0/text` (Context), `/content/4/content/0/text` (Notes), `/content/7/content/0/text` (References) | N/A | Expand at `/content/2`, panel at `/content/5`, rule at `/content/8` |

## Scoped Fetch Spike Results (current v4 pages)

| Page | Full ADF Bytes | Scoped ADF Bytes | Context Reduction | Scope Resolution |
|------|---------------|-----------------|-------------------|------------------|
| P1 | 1238 | 443 (`heading:Introduction`) | 64.2% | OK |
| P2 | 1869 | 1652 (`heading:Data`) | 11.6% | OK |
| P3 | 930 | 344 (`heading:Notes`) | 63.0% | OK |

## Notes

- Current page IDs correspond to v4 revalidation created on 2026-03-08.
- Target paths are discovered during Phase 2 spike via `jq` inspection of fetch state output.
- Bootstrap injects a heading + paragraph scaffold; pages are then seeded with pattern-specific ADF content.
- `panel` wrappers are `locked_structural` but inner paragraphs are `editable_prose`.
- The route classifier uses a catch-all: anything not in the 7-type prose whitelist or table family is `locked_structural`.
