# Sandbox Page Inventory

Page inventory for KPI experiment sandbox pages. All pages are children of the primary sandbox page (`131207`).

## Pages

| Page | ID | Parent | Title | Headings | Table | Locked Blocks | Size Estimate |
|------|----|--------|-------|----------|-------|---------------|---------------|
| P1 | 65934 | 131207 | KPI Experiment - Prose Rich | Introduction, Details, Summary | No | No | ~500-1000 ADF bytes |
| P2 | 98323 | 131207 | KPI Experiment - Mixed Prose Table | Overview, Data | Yes (3x3 text cells) | No | ~1000-2000 ADF bytes |
| P3 | 131227 | 131207 | KPI Experiment - Locked Adjacent | Context, Notes, References | No | Yes (expand macro, media or extension) | ~800-1500 ADF bytes |

## Route Classification Expected

| Page | `editable_prose` | `table_adf` | `locked_structural` |
|------|-------------------|-------------|---------------------|
| P1 | heading, paragraph, bulletList, orderedList, blockquote | None | None |
| P2 | heading, paragraph | table, tableRow, tableCell | None |
| P3 | heading, paragraph | None | expand, mediaSingle/media or extension |

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

- All headings on a given page must be distinct (no substring overlaps).
- Avoid heading names that are substrings of other headings on the same page (the scope resolver uses `text.contains()` matching).
- Good: "Introduction", "Details", "Summary" (no substring overlaps).
- Bad: "Overview", "Project Overview" (one is a substring of the other).

## Target Paths

Discovered during Phase 2 scoped fetch spike at commit `0e69067`.

| Page | Prose Target Path | Table Cell Target Path | Notes |
|------|-------------------|----------------------|-------|
| P1 | `/content/1/content/0/text` (Introduction), `/content/5/content/0/text` (Details), `/content/9/content/0/text` (Summary) | N/A | All nodes editable_prose |
| P2 | `/content/1/content/0/text` (Overview), `/content/4/content/0/text` (Data) | `/content/5/content/1/content/1/content/0/content/0/text` ("0 percent" cell) | Table at `/content/5` |
| P3 | `/content/1/content/0/text` (Context), `/content/5/content/0/text` (Notes), `/content/9/content/0/text` (References) | N/A | Expand at `/content/3`, panel at `/content/7`, rule at `/content/10` |

## Scoped Fetch Spike Results

| Page | Full ADF Bytes | Scoped ADF Bytes | Context Reduction | Scope Resolution |
|------|---------------|-----------------|-------------------|------------------|
| P1 | 2406 | 88 (heading:Introduction) | 96.3% | OK |
| P2 | 2416 | 80 (heading:Data) | 96.7% | OK |
| P3 | 2272 | 81 (heading:Notes) | 96.4% | OK |

## Notes

- Page IDs are filled after `create-subpage` execution.
- Target paths are discovered during Phase 2 spike via `jq` inspection of fetch state output.
- Bootstrap injects a heading + paragraph scaffold; user replaces/extends content via Confluence UI.
- `panel` wrappers are `locked_structural` but inner paragraphs are `editable_prose`.
- The route classifier uses a catch-all: anything not in the 7-type prose whitelist or table family is `locked_structural`.
