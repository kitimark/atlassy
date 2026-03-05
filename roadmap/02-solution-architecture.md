# Atlassy Solution Architecture (v1)

## Objective

Deliver a minimal-change Confluence update pipeline that keeps ADF fidelity while reducing token usage.

## Canonical Data Model

- ADF is canonical for `fetch`, `patch`, `verify`, and `publish`.
- Markdown is a transient assist format for prose-only edits.
- All publish operations use ADF payloads with scoped patch semantics.

## Block Routing Policy (v1)

- `editable_prose`: paragraph, heading, bulletList, orderedList, listItem, blockquote, simple codeBlock, rule.
- `table_adf`: table updates in ADF-native mode, cell text only.
- `locked_structural`: unsupported or high-risk nodes (media, macros/extensions, layout blocks, mentions, status, panels, embeds, and all non-whitelisted types).

## Pipeline State Contracts

### `fetch`

- Input: page ID, edit intent, optional heading/block scope.
- Output: scoped ADF subtree, page version, node-path index.
- Invariants: full page body is avoided unless scope resolution fails.

### `classify`

- Input: scoped ADF subtree.
- Output: node manifest with route labels (`editable_prose`, `table_adf`, `locked_structural`) and lock reasons.
- Invariants: every node is assigned exactly one route.

### `extract_prose`

- Input: `editable_prose` nodes.
- Output: Markdown assist payload plus stable mapping (`md_block_id <-> adf_path`).
- Invariants: no locked or table nodes are converted to Markdown.

### `md_assist_edit`

- Input: Markdown assist payload and edit intent.
- Output: edited Markdown plus declared changed block IDs.
- Invariants: no new top-level block types; no block-boundary expansion outside mapped prose blocks.

### `adf_table_edit`

- Input: `table_adf` nodes and edit intent.
- Output: candidate table node updates.
- Invariants: v1 allows cell text updates only; no row or column add/remove; no merge/split; no table attribute changes.

### `merge_candidates`

- Input: prose candidates and table candidates.
- Output: unified candidate node set keyed by ADF paths.
- Invariants: path uniqueness is enforced; cross-route conflicts fail fast.

### `patch`

- Input: original scoped ADF + candidate node updates.
- Output: path-targeted ADF patch operations and candidate page ADF.
- Invariants: unchanged nodes are preserved; whole-body rewrite is disallowed.

### `verify`

- Input: original scoped ADF and candidate page ADF.
- Output: pass/fail report with diagnostics.
- Invariants: ADF schema valid, locked-node fingerprint unchanged, no out-of-scope mutations, route policy respected.

### `publish`

- Input: verified candidate page ADF and current page version.
- Output: published version or failure artifact.
- Invariants: one scoped rebase retry on version conflict, then fail fast with diagnostics.

## Failure and Fallback Policy

- If Markdown conversion confidence is low, skip `md_assist_edit` and use direct ADF edit path for targeted prose nodes.
- If `verify` fails, block publish and return deterministic failure reasons.
- If publish conflicts after one rebase retry, return a reviewer artifact instead of repeated retries.
