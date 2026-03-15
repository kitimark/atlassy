## ADDED Requirements

### Requirement: Atomic builder functions construct valid ADF leaf and block nodes
The `atlassy-adf` crate SHALL provide pure functions in a `builders` module that construct valid ADF nodes as `serde_json::Value`.

#### Scenario: build_text creates a text node
- **WHEN** `build_text("Hello")` is called
- **THEN** it MUST return `{"type": "text", "text": "Hello"}`

#### Scenario: build_paragraph creates a paragraph with text content
- **WHEN** `build_paragraph("Some text")` is called
- **THEN** it MUST return a paragraph node with a single text child: `{"type": "paragraph", "content": [{"type": "text", "text": "Some text"}]}`

#### Scenario: build_paragraph with empty text creates empty paragraph
- **WHEN** `build_paragraph("")` is called
- **THEN** it MUST return `{"type": "paragraph", "content": [{"type": "text", "text": ""}]}`

#### Scenario: build_heading creates a heading with level and text
- **WHEN** `build_heading(2, "Title")` is called
- **THEN** it MUST return `{"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Title"}]}`

#### Scenario: build_heading validates level range
- **WHEN** `build_heading(level, text)` is called with level outside 1-6
- **THEN** it MUST return an error

### Requirement: Composite builder functions construct valid multi-level ADF structures
The builders module SHALL provide composite functions that use atomic builders to construct tables, lists, and sections.

#### Scenario: build_table creates a table with specified dimensions
- **WHEN** `build_table(2, 3, true)` is called (2 rows, 3 cols, with header)
- **THEN** it MUST return a valid table ADF node with 1 header row (tableHeader cells) and 1 data row (tableCell cells), each containing an empty paragraph

#### Scenario: build_table without header row
- **WHEN** `build_table(3, 2, false)` is called
- **THEN** it MUST return a table with 3 rows of tableCell cells (no tableHeader)

#### Scenario: build_table rejects zero rows or zero columns
- **WHEN** `build_table(0, 2, false)` or `build_table(2, 0, false)` is called
- **THEN** it MUST return an error

#### Scenario: build_list creates an ordered or unordered list
- **WHEN** `build_list(false, &["Item 1", "Item 2"])` is called
- **THEN** it MUST return `{"type": "bulletList", "content": [{"type": "listItem", "content": [{"type": "paragraph", ...}]}, ...]}`

#### Scenario: build_list creates ordered list
- **WHEN** `build_list(true, &["First", "Second"])` is called
- **THEN** it MUST return a node with `"type": "orderedList"` containing listItem children

#### Scenario: build_list rejects empty items
- **WHEN** `build_list(false, &[])` is called
- **THEN** it MUST return an error

#### Scenario: build_section returns heading plus body blocks
- **WHEN** `build_section(2, "FAQ", &[paragraph_value, list_value])` is called
- **THEN** it MUST return a `Vec<Value>` containing [heading(level=2, "FAQ"), paragraph_value, list_value]

#### Scenario: build_section with empty body
- **WHEN** `build_section(3, "Empty Section", &[])` is called
- **THEN** it MUST return a `Vec<Value>` containing only [heading(level=3, "Empty Section")]

### Requirement: All builder output passes structural validity checks
Every ADF structure produced by a builder function MUST pass `check_structural_validity()` when inserted into a valid document.

#### Scenario: Built table passes structural validity
- **WHEN** a table from `build_table(3, 3, true)` is inserted into a document
- **THEN** `check_structural_validity` MUST pass on the resulting ADF
