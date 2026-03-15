# Atlassy Solution Architecture

## Objective

Deliver a Confluence content control pipeline that enables insert, edit, and delete of ADF blocks across pages, while keeping ADF fidelity and structural safety.

The Foundation phases (0-5) established a minimal-change text-replacement pipeline. The Structural phases (6-9) extend the architecture to support structural operations (block insertion, deletion, composition) and multi-page orchestration.

## Canonical Data Model

- ADF is canonical for `fetch`, `patch`, `verify`, and `publish`.
- Markdown is a transient assist format for prose-only edits.
- All publish operations use ADF payloads with scoped patch semantics.

## Block Routing Policy

### Foundation Routes (text replacement)

- `editable_prose`: paragraph, heading, bulletList, orderedList, listItem, blockquote, simple codeBlock.
- `table_adf`: table updates in ADF-native mode, cell text only.
- `locked_structural`: unsupported or high-risk nodes (media, macros/extensions, layout blocks, mentions, status, panels, embeds, and all non-whitelisted types).

### Structural Route Extensions (Phases 6-9)

- `editable_prose` types gain insert/delete capabilities in Phase 6 (D-018). A paragraph can be inserted after a heading, or deleted from within scope.
- `table_adf` gains table creation (insert new table) in Phase 7 and topology changes (row/column add/remove) in Phase 9.
- `locked_structural` is relaxed for block-level insert/delete in Phase 7. Container nodes (panel, expand, layoutSection) become targets for inserting new child blocks or removing existing ones, while the container wrapper itself is preserved. Attribute editing remains locked until Phase 9.

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

## Phase 5.5 Refactoring Scope

Phase 5.5 restructures the type system and pipeline interfaces before adding new behavior. All changes are structural — zero behavior change, all existing tests pass.

### Type Consolidation (3 types to 1)

Three types currently carry the same concept:

- `PatchCandidate { path, value }` — in `atlassy-adf`, no `op` field, always implies replace.
- `PatchOperation { op: String, path, value }` — in `atlassy-adf`, `op` is always `"replace"`.
- `PatchOp { op: String, path, value }` — in `atlassy-contracts`, duplicate of `PatchOperation`.

All three are replaced by a single `Operation` enum in `atlassy-contracts`:

- Phase 5.5: `Operation::Replace { path: String, value: Value }` (only variant).
- Phase 6 adds: `Operation::Insert { parent_path, index, block }` and `Operation::Remove { target_path }`.

`Operation` is the single type flowing from candidate construction through merge, patch, and verify. Data and serialization live in `atlassy-contracts`; application behavior (`apply_replace`, `apply_insert`, `apply_remove`) lives in `atlassy-adf`.

### Pipeline State Addition

`AdfBlockOps` is added to the pipeline state order between `AdfTableEdit` and `MergeCandidates`:

`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> adf_block_ops -> merge_candidates -> patch -> verify -> publish`

In Phase 5.5, `AdfBlockOps` is a no-op pass-through. Phase 6 activates it to process `block_ops` from `RunRequest`.

### Contract Changes

- `RunRequest` gains `block_ops: Vec<BlockOp>` field (always empty in Phase 5.5).
- `MergeCandidatesOutput.changed_paths: Vec<String>` becomes `operations: Vec<Operation>`.
- `PatchOutput.patch_ops: Vec<PatchOp>` becomes `operations: Vec<Operation>`.
- `VerifyInput.changed_paths: Vec<String>` becomes `operations: Vec<Operation>`.

### Verify Restructuring

`verify.rs` is split into focused check functions (Extract Method):

- `check_operation_legality(ops, scope, index)` — scope/route/table guards per operation.
- `check_structural_validity(adf)` — post-mutation ADF validation (stub in Phase 5.5, activated in Phase 6).

Structural validity logic lives in `atlassy-adf` (ADF domain knowledge), called from the pipeline verify state.

## Structural Architecture Extensions (Phases 6-9)

### Block Operation Model

The Structural phases introduce two new operation kinds alongside the existing `Replace`:

- **`Insert`**: add a new ADF block at a specified position within scope. The operation specifies a parent path, an insertion index, and the ADF node to insert. Processed in reverse document order to prevent index shift cascading (D-020).
- **`Remove`**: delete an existing ADF block at a specified path within scope. The operation specifies the target path. Processed in reverse document order.
- **`Replace`**: retained from Foundation. Text-value replacement at leaf paths.

All insert and remove operations require post-mutation ADF schema validation before publish.

### Updated State Contracts (Structural)

#### `patch` (extended)

- Input adds: `operation_kind` per operation (`replace | insert | remove`), `insert_position` for insert ops.
- Output adds: `insert_count`, `remove_count` alongside existing `replace_count`.
- Constraints: operations sorted by descending document position before application. Whole-body rewrite still disallowed.

#### `verify` (extended)

- Adds: post-mutation ADF schema validation check.
- Adds: operation manifest cross-check (each changed path must correspond to a declared operation with matching kind).
- Distinguishes: intentional structural changes (matched by operation manifest) from accidental mutations (flagged as violations).

#### `classify` (extended, Phase 7)

- Adds: `insertable` and `removable` attributes to route labels. Blocks can be simultaneously text-editable and structurally modifiable.
- `locked_structural` nodes gain `insertable: true` for child-block insertion in Phase 7 while remaining `editable: false`.

### Multi-Page Extension (Phase 8)

- **Edit plans**: ordered list of per-page operations with dependency declarations between pages.
- **Rollback semantics**: each page operation is checkpointed. On failure, completed pages are reverted to their pre-operation version.
- **Page hierarchy awareness**: scope resolution can reference parent/child relationships for cross-page content coordination.

## AI Contract Reference

- Machine-oriented state I/O and error contracts are defined in `09-ai-contract-spec.md`.
