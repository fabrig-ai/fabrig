# configuring-p1: contracts

The shapes that cross this plan's tasks, binding for every spec that cites this note. A
task builds its half to them and needs no other task's code beyond its `needs`. Names in
`code` are exact; a field's order inside a struct is the implementer's.

## 1. jig's charter documents (jig's `jig/docs/design/domain/engine.md`, 17.9)

Built by `jig-documents-surface`; read by every later documents task.

```rust
// jig/crates/jig-domain-core/src/settings/charters.rs, exported at the crate root
/// A charter's document (domain/engine.md, 17.9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub connector: u16,
    /// The branch's resource name, within the authority's pattern bounds.
    pub name: jig_domain_authority::Name,
    /// 1 to `DOCUMENT_PATH_BYTES` bytes.
    pub path: Box<[u8]>,
    /// 1 to `DOCUMENT_HEADING_BYTES` bytes when present.
    pub heading: Option<Box<[u8]>>,
}
pub struct Charter { /* number, policy, hosts, spend, and: */ pub documents: Box<[Document]> }

// jig/crates/jig-domain-core/src/limits.rs, exported at the crate root
pub const DOCUMENT_PATH_BYTES: u32 = 256;
pub const DOCUMENT_HEADING_BYTES: u32 = 80;
/// The most sections the core plans of its own (domain/engine.md, section 9).
pub const CORE_SECTIONS: u32 = 6;
/// Whether a brief's sections hold the core's own most and a charter's documents (17.9).
pub fn brief_holds_documents(sections: u32, charter_documents: u32) -> bool; // sections >= CORE_SECTIONS + charter_documents
/// The bytes a charter owns beyond its struct: its policy's, as `policy_owned_bytes`, and its documents'.
pub fn charter_owned_bytes(charter: &Charter) -> Option<u64>;

// jig/crates/jig-domain-tasks/src/limits.rs
pub struct Limits { /* ..., */ pub charter_documents: u32 } // 2 in Small and Standard

// jig/crates/jig-domain-authority/src/limits.rs: `name_within` becomes public, exported at the crate root
pub fn name_within(name: &Name, limits: &Limits) -> bool;
```

The core's plan (`jig/crates/jig-domain-core/src/vocabulary.rs`):

```rust
Now::BriefCorePlanned { task: u64, parent: Option<u64>, sections: List<brief::Planned>, documents: Box<[PlannedDocument]> }

/// A charter document's section in a core plan, for the root to give its connector (17.9).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlannedDocument { pub token: Token, pub document: Document }

// jig/crates/jig-domain-core/src/limits.rs
/// The connector section kind the core plans a document under; the root may renumber it.
pub const DOCUMENT_KIND: u16 = 0;
pub const DOCUMENT_PRIORITY: u16 = 5;
```

- For each document of the run's charter version (`charter_of`, at the run's revision, 17.3),
  in the charter's order, while the plan has room after the core's own sections, the core
  pushes `brief::Planned::Connector { connector: document.connector, kind: DOCUMENT_KIND,
  token, size: 0, limit: <brief read_bytes>, priority: DOCUMENT_PRIORITY, required: false }`
  and a `PlannedDocument` with the same `token`. Its tokens are local to the plan, distinct
  from one another: `Token::new(1 + index)`. A document without room is left out of both.
- An application's root binds each `PlannedDocument` to its connector before the plan reaches
  the brief: it may rewrite the matching section's `token`, `kind` and `limit` (temper's does,
  section 3 below).
- jig's start check (in `jig-protocol-config`'s start, and temper's service start): refused
  unless `brief_holds_documents(brief.sections, tasks.charter_documents)`, naming
  `brief.sections` and `tasks.charter_documents`. `jig_domain_core::valid` does not run it:
  fixtures built in code keep their sections, and the plan's room guard leaves documents out.

## 2. jig's stored and wire forms

- **Settings store** (`jig/crates/jig-protocol-store/schema/settings-v1.schema`, in place):
  `record Document { connector: u16 name: list Segment path: bytes 256 heading: option bytes 80 }`
  and `documents: list Document` on `record Charter`, with ceilings no wider than the codec's
  limits check needs (documents at most 8; a name's segments at the ceiling the schema's other
  names use). `Segment` moves above `Document` if the schema needs it.
- **Configuration** (`jig/docs/design/protocol/configuration.md`, section 3, `charters`): a
  charter's optional member `"documents": [{"connector": n, "name": [s, ...], "path": s,
  "heading": s}]`, `heading` optional; absent, none. Override names for the limit: wherever
  `tasks.charters` is accepted, `tasks.charter_documents` is, in the same forms.
- **Client wire** (`jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema`, in
  place): `record CharterDocument { connector: u16 name: list bytes path: bytes 256 heading:
  option bytes 80 }` and `documents: list CharterDocument` on `record Charter`; the contract
  (`jig/crates/jig-client-domain/src/contract/settings.rs`) gains
  `pub struct CharterDocument { pub connector: u16, pub name: <the contract's list>
  of Box<[u8]>, pub path: Box<[u8]>, pub heading: Option<Box<[u8]>> }` and
  `Charter.documents`, in the contract's own collection type. Both translates map them one to
  one; their bounds come from `tasks.charter_documents` and the authority's `segments` and
  `segment_bytes`.

## 3. temper's forge and root (temper's `temper/docs/design/protocol/configuration.md`, 13.7)

The forge (`temper/crates/temper-engine-domain-forge/src/boundary.rs`), built by
`forge-document-gather`:

```rust
/// A charter's document to gather as a brief section (protocol/configuration.md, 13.7).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BriefDocument {
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    /// The branch's segments joined by `/`.
    pub branch: Box<[u8]>,
    pub path: Box<[u8]>,
    pub heading: Option<Box<[u8]>>,
}
Event::PlanDocument { section: Token, document: Box<BriefDocument> }
BriefSource::Document // a unit variant: `BriefSource` stays `Copy`
```

- `PlanDocument` registers the section as `PlanBrief` does, its source `BriefSource::Document`;
  every later event and request of the section (`GatherPlanned`, `CutBrief`, `TakeBrief`,
  `DropBrief`, `BriefSized`, `take_brief`) is the one the other sources use. No row outlives
  `TakeBrief` or `DropBrief`.
- The gather: the repository adopted, in any project, under `host`, `owner` and `name`, its
  forge not ended; `Read::Branch` for the head, then `Read::File` at that head; the text, or
  with a heading its part (from the first line `#`... `# HEADING` whose text after its `#`s and
  one space equals the heading byte for byte, to the line before the next heading of its level
  or above); the section's words:

  ```
  At BRANCH's head COMMIT
  TEXT
  [N bytes cut]
  ```

  `COMMIT` as `commit_text` writes it; the last line only when cut, at the last line end
  within the allotment (`GatherPlanned.bytes`, and each `CutBrief`'s), `N` the bytes left out,
  `[more than N bytes cut]` when the read itself was truncated. A cut re-renders the words held,
  never reading again: one commit for the section's whole life.
- A section saying why, as its words (`BriefRead::Got`), each one line:
  `Repository OWNER/NAME on HOST is not adopted.`;
  `No branch BRANCH in OWNER/NAME.`;
  `No file PATH in OWNER/NAME at BRANCH's head COMMIT.` (a directory too);
  `No heading "HEADING" in PATH at BRANCH's head COMMIT.`.
  A read that failed or was refused is `BriefRead::Failed`, as for the other sources.

The root (`temper/crates/temper-engine-domain`), built by `root-document-binding`:

- `ForgeBriefKind::Document(Box<[u8]>)` carries its title's words; `ForgeBriefKind` and
  `BriefKind` stop being `Copy`. `temper-engine-smith`'s section title is `Document ` and
  those words: `OWNER/NAME PATH`, or `OWNER/NAME PATH, "HEADING"`.
- A document's section kind on the connector is `DOCUMENT_KINDS + index`
  (`pub(crate) const DOCUMENT_KINDS: u16 = 16`), `index` its place in the charter's
  documents, so a `GatherPlaced::Missing { kind, .. }` names its document; its limit is
  `root.brief.budgets.document`.
- A document's `Name` is a forge branch when its connector is 0 and its segments read
  `forge HOST OWNER REPO branch SEGMENT...` (`temper/docs/design/domain/forge.md`, section 2);
  any other is answered `Missing` at its gather, the forge not asked: its section is titled
  `Document PATH` and says its read failed.
- `BriefBudgets.document: u32`, 4,096 in `small`, override `root.brief.budgets.document`;
  `root.brief.sections` 8 in `small`. The root's required forge sections replace optional core
  sections only, never a document.

## 4. temper's settings file (temper's `temper/docs/design/protocol/configuration.md`, 13.7)

Built by `settings-file-documents`. A charter's value in a settings file and in its rendering:
`"documents": [{"forge": F, "repository": "OWNER/NAME", "branch": B, "path": P, "heading": H}]`,
`heading` optional, `F` the forge's host; ctl turns it into connector 0 and the name
`forge F OWNER NAME branch` followed by B's segments (B split on `/`), and back. A stored
document whose name is no forge branch renders `"name": [segments]` with its `connector`,
`path` and `heading`, and a settings file naming one that way is refused as out of shape.
