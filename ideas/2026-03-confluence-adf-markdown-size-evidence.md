# Confluence ADF vs Markdown Size Evidence (Public Sample)

## Status

Incubating (evidence note)

## Scope

Public benchmark source:

- Site: `https://xilinx-wiki.atlassian.net/wiki`
- Dataset: first 5 pages from space `A`
- Page set size: 5 pages

## Measurement Method

- Fetch listing with `GET /wiki/rest/api/content?spaceKey=A&type=page&limit=5`.
- For each page, fetch both `body.storage` and `body.atlas_doc_format`.
- Storage bytes: UTF-8 byte length of `body.storage.value`.
- ADF bytes: UTF-8 byte length of compact JSON serialization of `body.atlas_doc_format.value`.
- Markdown bytes: derived from storage XHTML via HTML-to-Markdown conversion pipeline (`markdownify`), then measured as UTF-8 byte length.
- Ratios: `adf_bytes / storage_bytes` and `adf_bytes / markdown_bytes`.

## Page Mapping and Size Comparison

| Page ID | Title | Storage Bytes | Markdown Bytes (Derived) | ADF Bytes | ADF/Storage | ADF/Markdown |
|---|---|---:|---:|---:|---:|---:|
| 17989819 | Linux LTTEmac Flat | 3,331 | 2,368 | 5,183 | 1.56x | 2.19x |
| 18841602 | Zynq-7000 AP SoC - Base TRD execution from 32 Bit ECC Proxy System Tech Tip | 55,403 | 28,206 | 72,158 | 1.30x | 2.56x |
| 18841603 | 2017.1 U-boot Release Notes | 2,486 | 1,603 | 8,434 | 3.39x | 5.26x |
| 18841604 | KCU105 SGMII over LVDS design creation using board flow | 7,264 | 3,825 | 16,433 | 2.26x | 4.30x |
| 18841607 | sdiaud Driver | 3,090 | 1,675 | 5,305 | 1.72x | 3.17x |

Aggregate totals:

- Total storage bytes: 71,574
- Total markdown bytes (derived): 37,677
- Total ADF bytes: 107,513
- Overall ADF/storage ratio: 1.50x
- Overall ADF/markdown ratio: 2.85x

## Structural Verbosity Signals

- Tables are present in the sample (18 table nodes total).
- Media is present and frequent (28 media nodes total).
- Extension nodes are present (7 total), which are high-risk for lossy markdown round-trips.
- Pages with high table/media density show stronger payload inflation and higher editing complexity.

## Plain Problem Points

- ADF is consistently heavier than storage/markdown representations in this public sample.
- Full-page ADF retrieval and update paths increase token pressure and payload-risk during AI-assisted editing.
- Markdown remains better for human readability and token efficiency, but fidelity-critical blocks still need strict preservation.
- Table/media/extension-heavy pages are high-risk for conversion drift and should avoid full-body rewrite workflows.

## Implications for Atlassy

- Keep ADF as canonical for planning, patching, and publish.
- Use Markdown assist only for prose-friendly editing contexts.
- Preserve fidelity-critical blocks with lock/preserve policy.
- Prefer section-level retrieval and minimal patch planning.
- Add dry-run fidelity checks before any publish step.

## PoC Measurement Checklist

- Capture baseline and optimized runs for the same edit intent.
- Measure `context_reduction_ratio` and `scoped_section_tokens`.
- Track `conflict_rate`, `edit_success_rate`, and structural preservation risk flags.
- Report per-page outliers to drive block policy adjustments.

## Limitations

- Markdown is derived via conversion pipeline, not returned as native markdown from Confluence REST.
- Dataset is a small public sample (5 pages) and should be expanded across more page types for generalization.
