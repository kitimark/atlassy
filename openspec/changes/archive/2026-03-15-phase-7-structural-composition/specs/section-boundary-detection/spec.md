## ADDED Requirements

### Requirement: find_section_range identifies all blocks in a section
The `atlassy-adf` crate SHALL provide a `find_section_range()` function that, given an ADF document and a heading path, returns the range of blocks belonging to that heading's section.

#### Scenario: Section with body blocks terminated by same-level heading
- **GIVEN** doc.content = [H1("Intro"), para("text"), H2("Details"), para("d1"), para("d2"), H2("Summary"), para("s1")]
- **WHEN** `find_section_range(adf, "/content/2")` is called (targeting H2 "Details")
- **THEN** it MUST return SectionRange { heading_index: 2, end_index: 5, block_count: 3, block_paths: ["/content/2", "/content/3", "/content/4"] }

#### Scenario: Section terminated by higher-level heading
- **GIVEN** doc.content = [H1("Intro"), H2("Sub"), para("text"), H1("Next")]
- **WHEN** `find_section_range(adf, "/content/1")` is called (targeting H2 "Sub")
- **THEN** it MUST return SectionRange { heading_index: 1, end_index: 3, block_count: 2 }

#### Scenario: Section at end of document (no terminating heading)
- **GIVEN** doc.content = [H1("Only"), para("text1"), para("text2")]
- **WHEN** `find_section_range(adf, "/content/0")` is called
- **THEN** it MUST return SectionRange { heading_index: 0, end_index: 3, block_count: 3 }

#### Scenario: Section with no body blocks (consecutive headings)
- **GIVEN** doc.content = [H2("Empty"), H2("Next")]
- **WHEN** `find_section_range(adf, "/content/0")` is called
- **THEN** it MUST return SectionRange { heading_index: 0, end_index: 1, block_count: 1 }

#### Scenario: Target path is not a heading
- **WHEN** `find_section_range(adf, "/content/1")` is called and the block at index 1 is a paragraph
- **THEN** it MUST return an error

#### Scenario: Target path does not resolve
- **WHEN** `find_section_range(adf, "/content/99")` is called and the index is out of bounds
- **THEN** it MUST return an error

### Requirement: SectionRange provides block paths for Operation generation
The `SectionRange` struct MUST include `block_paths: Vec<String>` containing the JSON pointer paths of all blocks in the section (heading + body), ordered from first to last. Callers use these paths to generate `Operation::Remove` commands in reverse order.

#### Scenario: Block paths are in document order
- **WHEN** a SectionRange is returned
- **THEN** `block_paths` MUST be in ascending index order (heading first, last body block last)
