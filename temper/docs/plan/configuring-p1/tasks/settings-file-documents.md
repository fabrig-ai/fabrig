+++
id = "settings-file-documents"
part = "documents-temper"
kind = "feature"
needs = ["jig-documents-wire"]
files = ["temper/crates/temper-settings-form/src/**", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/crates/temper-client-tools/src/limits.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/crates/temper/src/tests/settings_fields.rs", "temper/docs/help/reference/settings-fields.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 90
+++

# temper: a settings file names a charter's documents; ctl shows them

## Goal

A person, or QA, gives a charter its documents through `temper ctl`: a settings file's charter
value carries `"documents": [{"forge", "repository", "branch", "path", "heading"}]`, which ctl
turns into jig's document (connector 0, the branch's resource name) and back, in
`settings apply`, `settings propose`, `settings show charter`, `revision show` and a proposal's
review (temper's `temper/docs/design/protocol/configuration.md`, 13.7, "In a settings file";
`temper/docs/design/client/tools.md`, 5.2 and 5.3).

Acceptance:

- The form (`temper-settings-form`): its `Charter` view gains `documents`; a new
  `Shape::Document` form with its five fields; the settings file's decoder reads them, the
  values' encoder writes them, the wire mapping (`wire.rs`) and ctl's adapter
  (`temper-client-tools/src/jig/settings_adapter.rs`) carry them to and from the client
  contract's `CharterDocument` (`jig-documents-wire`'s), as
  `temper/docs/plan/configuring-p1/notes/contracts.md`, section 4 says: `forge` the host,
  `repository` `OWNER/NAME`, `branch` split on `/` into the name's segments after `branch`,
  connector 0; and back. A stored document whose name is no forge branch renders with `name`,
  its segments, beside `connector`, `path` and `heading`; a settings file that names one so is
  refused as out of shape, naming its path. Bounds: at most `tasks.charter_documents`
  documents, a path of at most 256 bytes, a heading of at most 80, each part of the name within
  the authority's `segment_bytes` and the whole within its `segments`, from the client's limits
  as the form's other bounds are; each refusal names its member's path.
- The form's rows, exactly (lines as given; `Limit`/`Bound` variants are yours):
  - `Form { shape: Shape::Document, name: b"document", choice: Choice::Every, line: b"A document every run of the charter reads in its brief, as the file is at the branch's head when the run is prepared: a rule that lands there reaches the next run." }`
  - the charter's `documents`: `list` bounded by `tasks.charter_documents`, `null: false`,
    `line: b"The documents the charter's runs read, each as a brief section of its own; at most `tasks.charter_documents`, else refused as `shape`."`
  - the document's `forge`: `line: b"The forge's host, as its repositories' names carry it."`;
    `repository`: `line: b"The repository as OWNER/NAME; one adopted in no project gives a section saying so."`;
    `branch`: `line: b"The branch whose head is read, its segments joined by `/`."`;
    `path`: `line: b"The file's path in the repository, at most 256 bytes."`;
    `heading`: `null: true`, `line: b"A Markdown heading's text: the run reads from that heading to the next of its level or above; null reads the whole file."`.
- `temper/docs/help/reference/settings-fields.md` is the page `fields::page` renders, whole: copy
  it from `temper/crates/temper/src/tests/settings_fields.rs`'s failure output, and write nothing
  else there.
- Tests, in `temper-client-tools/src/tests/jig/settings.rs` (its limits table gains the new
  bounds):
  - `a_settings_file_charter_names_documents`: a file whose charter has two documents (one with a
    heading, one branch with a `/`) gives a `ChangeSettings` whose charter carries both, connector
    0 and names `forge HOST OWNER NAME branch SEG...`;
  - `settings_show_charter_renders_its_documents` in text and JSON, the goldens regenerated
    (`SKEIN_CODEGEN_WRITE=1`) and read;
  - `a_document_that_is_no_forge_branch_renders_its_name` and is refused back from a file;
  - each bound one past refused naming its path.
  `heavy cargo nextest run -p temper-client-tools --lib -E 'test(/settings::/)'` passes, its count
  in the report; then `heavy cargo nextest run -p temper-settings-form -p temper-client-tools`
  and `heavy cargo nextest run -p temper --lib -E 'test(/settings_fields::/)'`.
- The settings file's sentence leaves `temper/docs/design/protocol/configuration.status.md`:
  delete "13.7: a settings file's charter carries no `documents`." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 2 and 4.
- `temper/docs/design/protocol/configuration.md`, 13.7; `temper/docs/design/client/tools.md`,
  5.2 (`settings show`), 5.3 (`settings apply`, `settings propose`) and 6.1–6.2 (text and JSON).
- Old code, for reference: the old temper branch decoded `documents` with the branch form
  (`forge` the host, `owner/name`, the branch split on `/`, connector 0) and rendered it back,
  falling back to an opaque form for other names; this plan's fallback is the `name` form above.

## Owns and may change

- Owns the files in the header.
- Read only: jig's client contract and wire (`jig-documents-wire`'s).
- The plan beside this one (reference-m1's `settings-wake-clients`) edits `temper-settings-form`
  and `settings_adapter.rs` too: add your rows and arms beside the charter's others; reformat and
  reorder nothing.
- Pre-approved: if a golden other than the charter's value, revision and review goldens moves
  only because a charter now prints `"documents": []`, regenerate it and list it.

## Steps

1. The view, the form's rows, the decoder, the encoder, the wire mapping, the adapter, the bounds.
2. The tests; run them alone; regenerate the goldens; the page.
3. The status sentence.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-settings-form -p temper-client-tools -p temper --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper-client-shell-cli`; the citations check.
5. Commit (why: ctl gives a charter its documents and shows them); push your branch.

## Report

As the implementer's text says, and also: the goldens regenerated, the bounds' sources, and the
tests' count.
