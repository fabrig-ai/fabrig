# The command and an owner's files

What a family's owner keeps in its crate, and the command that turns a
family's texts into its code. Every family's version is frozen and a
layout is named by its fingerprint (`README.md`, section 5), so an owner
keeps its families' current texts alone: no older text, no patch, no
list of versions or layouts. A value at another layout is converted by a
plan from texts the store or the peer holds (`layouts.md`), never from a
text in the repository.

## 1. The crate's files

| Path | What | Whose |
|---|---|---|
| `schema/<family>-v<N>.schema` | the family's current text, `N` its header's frozen version; a fixture's directory may hold several versions (below) | the author's |
| `schema/<family>-v<N>-part<K>.schema` | a split family's further parts, read after the first in their order (8) | the author's |
| `schema/<name>.maps` | the translation between the family's records and the owner's domain, in each crate that translates them (`maps.md`, 2 and 9) | the author's |
| `schema/seed/<family>-v<N>.schema` | an older text a store registers layouts from, read by the store alone: `smith-transcript`'s version 7, the seed for frames written before the registry (temper's `protocol/wires.md`, 3.1) | the author's |
| `src/generated/<stem>.rs`, in the family's codec crate | the records' module (`README.md`, section 4) | the command's (8), or the drift test's in write mode |
| the translation module, in the crate that translates | the maps file's generated translation (`maps.md`, 4 and 7) | the same |
| `bounds/<stem>.bounds`, in the drift test's world | the family's bounds file (`golden.md`, section 1) | the drift test's in write mode |

- **One text per family.** A family a build stores, sends or embeds
  keeps its current text alone in `schema/`, and its generated code is
  that text's alone; the boundary suite's one-version check holds both
  (`tools/crates/design.md`, 3.2). A change of layout is made in that
  text: it moves no version and renames no file. No crate keeps an
  older text, a patch, a list of its wires or its registered families,
  or a list of layouts: the store's registry and a worker's opening
  carry the other layouts' texts (`README.md`, section 5).
- **A codec crate apart from its owner** (`README.md`, section 2) holds
  the family's generated record code alone. The owner's drift test
  (`golden.md`, section 1), which names each family's codec crate,
  writes the code there when asked.
- **Fixtures belong to no build:** `skein/tests/codec`'s `every` texts
  and its other fixtures are stored, sent and embedded by nothing, so
  their `schema/` holds several versions of one family side by side,
  which the layout plans' tests pair (`layouts.md`, 6), and their
  generated files keep their versions in their names (`README.md`,
  section 4).

## 8. The command

`skein-codegen <schema>... [<maps>] <out-dir>`, skein-codegen's binary:

- **It reads the schemas' texts,** concatenated in the order given with
  nothing between them, as one schema, through `skein-codec`'s parser
  and the generator's checks (`run-time.md`, section 1). The first is
  named `<family>-v<N>.schema`.
- **It writes one module,** `<out-dir>/<stem>.rs`, the first schema's
  stem (`README.md`, section 4), making `<out-dir>` where absent, and
  nothing else: the records' module, or, with a last input whose name
  ends in `.maps`, the translation module in its place (`maps.md`, 7).
- **A failure writes nothing:** a usage error, an input that does not
  read, a schema that does not parse or a maps file the generator
  refuses (`maps.md`, 6) exits 2, saying why on stderr.
- **No subcommand:** every argument but the last is an input, whatever
  its name.
- **Its module is the drift test's:** a family's drift test writes the
  same module when `SKEIN_CODEGEN_WRITE=1` asks it and compares it
  otherwise (`golden.md`, section 1), and the merge job runs the command
  on `tools/merge/codecs.txt`'s rows
  (`temper/docs/design/protocol/tooling.md`, 4.1).

## 9. Tests

- **Step tests,** in the binary's own module: its arguments read, the
  inputs then the output directory, too few a usage error, and a first
  input named `patch` read as an input; a schema's records' module
  written alone, nothing else under the output directory; a split
  family's texts concatenated in their order into its first stem's
  file; a stem's hyphens made underscores; and a schema that does not
  read or does not parse writing nothing.
- **What only the binary shows,** in skein-codegen's suite: the maps
  command over split schema texts, writing its first stem's file alone;
  and a maps refusal printed on stderr, its exit failing and no output
  written (`maps.md`, 8).
