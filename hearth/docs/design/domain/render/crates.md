# Render: layers and crates

## 1. Layers and crates

### 1.1 The crates

```
hearth-domain-builder            the builder's root: routes between its children
├── hearth-domain-render         configuration to spec; resolution's decisions; the diff        M1
└── hearth-domain-store          M1: resolutions and cache files' records; M2: the rest         M1, M2
hearth-domain-render-vocabulary  render's values: the spec's parts and keys, the diff's changes, the errors  M1

hearth-protocol-builder          the builder's protocol layer: routes its machines, compiles configuration into render's types
├── hearth-protocol-git          git's argument vectors; ls-tree, cat-file and diff records       M1
├── hearth-protocol-tree         M1: the frame, 3.1's decoder, records of kinds 1, 4 and 5        M1, M2
├── hearth-protocol-config       strict JSON to typed documents and declarations, with positions  M1
├── hearth-protocol-template     template and expression text to instruction sequences            M1
└── hearth-protocol-mmdebstrap   simulate's argument vector; Inst lines; Packages stanzas; tails  M1
```

- **`hearth-domain-render`** holds the pipeline, resolution's decisions and the diff, and depends on lib and its vocabulary alone. Outside it are read what crosses (1.2), the spec's records (`spec.md`, section 1), the diff's (`diff.md`, 1.1), the errors (`errors.md`), the limits (`README.md`, 2.3), and what the world's laws and oracle drive (`README.md`, 6.3, 6.5): evaluation, merging, validation and expansion. The rest is its own.
- **`hearth-domain-render-vocabulary`** holds render's values that other crates name (`skein/docs/foundation/programming-model.md`, 4.6): the spec's parts and keys, the diff's changes and the errors, with the values their definitions reach, such as positions, declaration names, filters, export values, the error's class and the outcome. The two crates share one module tree: an item keeps its module path, so `hearth_domain_render_vocabulary::parts::packages::Component` is `hearth_domain_render::parts::packages::Component`, since each of render's modules re-exports its namesake in the vocabulary. A method that reads render's limits or decides, such as whether a changed file's hunks are asked for (`diff.md`, 1.1), stays in render as a free function. Render's limits stay in render: no other crate names them.
- **Who names the vocabulary:** render, the builder's root and protocol layer, the client's printer, and the machines that decode its values: `hearth-protocol-config` a declaration's names, script fragments, placement kinds and firmware, and the fields and faults it refuses with; `hearth-protocol-git` a diff's hunks and a listing's modes; `hearth-protocol-mmdebstrap` the archive's components. None holds a copy of them. The protocol layer translates only where a layer adds something: configuration's documents and declarations, which keep the byte offsets their decoder orders refusals by and are compiled, with their expressions, into render's; the template machine's instructions; the tools' and the store's own vocabularies.
- **The machines depend on lib only** (`skein/docs/foundation/programming-model.md`, 4), and on render's vocabulary where they decode its values, but `hearth-protocol-config`, which composes skein's JSON tokenizer and depends on `skein-json` too, as `skein-llm` does (`skein/docs/design/llm/README.md`). It can't call the template machine, so it hands up a declaration's expressions as text with their positions, and `hearth-protocol-builder` passes each through `hearth-protocol-template` before translating the declaration into render's types.
- **The client runs the same domain in-process** when no daemon answers, or for a local repository (hearth/docs/design/client/README.md, 3.5); in the daemon, the builder's root takes the steward's jobs (hearth/docs/design/domain/steward.md, section 3).
- **The client prints with `hearth-client-protocol-records`'s printer** (hearth/docs/design/client/README.md, 3.2), which writes the spec, the diff and the errors, render's values from its vocabulary, as text and as JSON lines.

### 1.2 What crosses

```
render → protocol                                      protocol → render
List { repository, revision }                          Listed { commit, entries: path, mode, oid, size }
Read { commit, blobs: (oid, as) }                      Read { oid, a document | a declaration | a template | a link's target }
  as: document | declaration | template | link | cached  Cached { oid, size, sha256 }: streamed into the cache
Keep { sha256, bytes }: a rendered file                Kept { sha256 }
Resolve { key, sources, request, variant, apt }        Resolved { key, closure, indices, versions, cost }
Hunks { before: sha256, after: sha256 }                Hunks { lines: (kind, text), left out, binary }: kind is header, context, removed, added or no newline (4.1)
every request                                          Failed { request, class, at, tail }
```

- **Resolution records** go through the builder's root to the store and back (hearth/docs/design/domain/store/records.md): render asks for a key and records a new closure, and never touches a file. In M1 the store's domain is `hearth-domain-store`'s first part, resolutions and cache files' records alone (hearth/docs/design/domain/store/README.md, 2.4; hearth/docs/design/domain/store/records.md, 1).
- **Every request gets one terminal event** (`skein/docs/foundation/programming-model.md`, 5.2); a resolution's deadline is render's to set, and git's commands have none (`README.md`, 2.2).

### 1.3 Evaluation without recursion

Step code has no recursion and no unbounded loop (`skein/docs/foundation/programming-model.md`, section 3), so nothing is evaluated as a tree:

- **An expression** is a flat sequence of operations in postfix order, with forward jumps for `and`, `or` and `… if … else …`, evaluated on a value stack the expression limit bounds.
- **A template** is a flat sequence of instructions (emit text, emit an expression, branch, enter and advance a loop, assign), run by a `for` bounded by the iteration limit, with a stack of loop frames bounded by the nesting limit. It keeps the offset of each line's first byte, which places a fault at its line and column (`errors.md`, 1) without reading the blob again (`README.md`, 3.10), and which its count includes (`README.md`, 2.3).
- **Merging** walks two values with an explicit stack bounded by JSON's depth.

### 1.4 Limits and what a render holds

- **A render holds at most one of each amount `README.md`, 2.3 bounds:** the revision's listing and its static files' digests, its decoded blobs, the exports of one environment, one target's settings and entries, one evaluation's values, one rendered file, the command's specs and closures, and its errors. It holds no other target's settings (`README.md`, section 3). Of a resolution record it holds the closure's digest, which its layer keys take (`spec.md`, 1.5), and a diff the closures it compares (`spec.md`, 2.7; `diff.md`, 1.1).
- **A diff renders its sides in turn,** keeping of the first only its specs, closures and errors, and, until the second's listing, its decoded blobs. **The second side decodes only the blobs that changed** (`README.md`, 3.10): of the first side's decoded blobs and static files' digests it keeps those the new listing names, by object id and as what they were read, and reads only the others, a blob the first refused among them. It gives what a render of its revision alone gives, within the same 16 MiB: each blob counts in that render's order, kept or read, and when what it holds would pass the limit, the kept blobs not yet counted go first, and one it then needs is read again. So a diff's peak holds one revision's decoded data, as a render's does.
- **The heap is measured, under ceilings** (`skein/docs/foundation/programming-model.md`, 6.3): each fixed scenario's peak stays under a ceiling named beside its test (`skein/docs/foundation/testing-strategy.md`, section 6), and no formula sums a layer. A formula sizes an edge alone (a decoder's retained bound, a decoded record, one template run's workspace), named for what it bounds, `None` on overflow, with a test of its value; the builder's `valid(limits)`, computed from them, is startup's check.
- **The defaults fit a modest machine, by measurement:** the large render, a revision of 4,096 entries, 512 of them documents, declarations and templates, with 64 targets, runs through the builder at the shipped limits under a ceiling of its own (hearth/docs/design/testing/worlds.md, 4).
- **The protocol's machines take `README.md`, 2.3's limits** and the store's bounds (hearth/docs/design/domain/store/records.md, 1); the client's printer (hearth/docs/design/client/README.md, 3.2) prints a record of at most 16 MiB, line endings included, and refuses a longer one, which fails the command. Static and rendered files live in the store's cache, never in the domain, so a small VM's builder renders what a large one does.
- **Between the builder's layers nothing is capped:** the queues between its children grow as they fill, and only the edges refuse (`skein/docs/foundation/programming-model.md`, 6.3). `Limits::queue` sizes the two queues the builder shares with the kernel, `completions` and `submissions`, and nothing else.
- **A watermark holds render behind a client that has stopped reading** (`skein/docs/foundation/programming-model.md`, 7.1). `Limits::watermark`, 30 as shipped, bounds the backlog that `told`, `file_told`, `answers` and `asked` hold: at the watermark the domain's next resume waits, so render makes no more records, and about a watermark of a call's targets wait behind a stuck client. What finishes admitted work is always taken: an event enters the protocol layer, an answer goes down once the printer takes it, and a request reaches io, at any backlog. `calls` stays outside the backlog, since its events wait behind the domain's own next resume. The watermark paces and bounds no memory: a backlog may pass it by what one admitted event makes.
- **A pass is paced:** it takes from each queue what the queue held when the pass reached it and resumes a layer at most `Limits::watermark` times, leaving the rest ready for the next pass. Io's stage paces itself (`skein/crates/skein-io/src/stage.rs`).

### 1.5 What keeps its order

No order here rests on a queue's room (`skein/docs/foundation/programming-model.md`, 7.2): a machine's state or a named predicate keeps each one, and a test holds it.

- **One read at a time.** A git machine, mmdebstrap's pipe reader and list reader, and the JSON decoder have at most one demand out, and make the next once the last is answered.
- **A blob's piece is taken before the next is read.** `cat-file`'s machine stops after each piece until it is told `Next`, so a body waits in git's pipe, never in the builder, and a 64 MiB static file is held a chunk at a time.
- **An outcome answers a request.** A stream's end or fault that comes before a request, or while a piece is being taken, overtakes nothing: the machine holds it for the next request.
- **A pipe's reader gives one thing a delivery:** an `Inst`, a `Fetched` or the pipe's end, and the next demand. Resolution takes one of each from a delivery, and one demand from a list reader's; the pipe's end comes last, and once.
- **A list's end follows its last stanza.** A scan gives at most one stanza; a list with no blank line after its last stanza gives that stanza and then its end, once.
- **A decoder gives one thing a delivery:** the JSON decoder gives the next demand or a document's outcome; a read takes one of each and drops the decoder at its outcome. The outcome comes once, with no read out; `Closed` comes last, and once.
- **Resumed work goes before the next event.** While the protocol layer has work to resume, no io or file event enters it (`work_pending`).
- **One write at a time.** The client's printer has at most one demand out; a `Room` gives one piece within the room it answers, then the next demand or nothing.
- **A record is answered once:** `Printed` once its last piece has gone down, `Refused` with no bytes, or `Failed`; `Client` takes it before the next record.
- **One record at a time.** The printer takes the next record once the last has gone out, on standard output and standard error alike (`takes`), so answers print in the order render gave them.
- **An answer follows its resources' close.** Render hears from a git child once the process and its pipes are closed, so its next request finds none open.
- **A build's report comes whole:** its layers, then the step it failed on, then its reply; the builder's root holds render's reply until the report is out (`holds`, `reporting`).
- **The domain takes a store step's answer before the store's next step,** so nothing waits in the queue above it: a step has answered once it added to what the protocol layer gave above (`store_answered`), and only then does the next index open.
- **Render waits at the watermark, and what it asked for does not.** The domain's next resume waits while the backlog is at the watermark (`admits`, 1.4); nothing else waits on that number.
