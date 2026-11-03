+++
id = "ctl-connected"
part = "ctl"
kind = "feature"
needs = ["pasted-list", "wire-connected"]
files = [
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/forges/**",
  "temper/crates/temper-client-tools/src/table.rs",
  "temper/crates/temper-client-tools/src/jig/rows.rs",
  "temper/crates/temper-client-tools/src/tests/forge_connection.rs",
  "temper/crates/temper-client-tools/src/tests/fixtures.rs",
  "temper/crates/temper-client-tools/src/tests/help.rs",
  "temper/crates/temper-client-tools/src/tests/landing.rs",
  "temper/crates/temper-client-tools/src/tests/golden/**",
  "temper/crates/temper-client-shell-cli/src/arguments.rs",
  "temper/crates/temper-client-shell-cli/src/tests/keys.rs",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/help/forgejo.md",
  "temper/docs/help/forge.md",
  "temper/docs/help/github.md",
  "temper/docs/help/party.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 120
+++

# ctl connects a Forgejo by its bot's tokens, and replaces a forge's key or tokens

## Goal

`temper ctl forge connect [FORGE] forgejo FILE` is refused today as not offered, and no `forge
replace` exists. Offer both as `temper/docs/design/client/tools.md`, 5.5 writes them: a tokens
file read once and sent as one carried bundle, a Replace by a key file or a tokens file, each
printing `connecting` and the wait that follows. The help says so.

Acceptance:

- `forge connect`'s form 2, `FORGE forgejo FILE --key KEY`, reads FILE as a key file is read
  (private, within `PASTED_PAYLOAD`), parses it as a tokens file and sends `Connect { forge:
  Making::Kept(FORGE), how: How::ForgejoTokens }`; a new last form, `forgejo FILE --api HOST:PORT
  [--tls NAME] --git URL [--web URL] [--git-program FILE] --key KEY`, sends it for a new forge,
  `ForgeAsk::Connected(ManagedForge { .. })` built as form 6 builds `Managed`'s. Both print
  `connecting`, then the call that follows, `forge list --until connected:FORGE` (for a new
  forge, as form 6 prints it), and exit 0 once the request is durable; refused, exit 2 in
  `connect_refusal`'s words.
- **The tokens file** (tools.md, 5.5, *A pasted value is read from FILE*): one item a line,
  `NAME VALUE`, NAME one of `api`, `git` and `webhook_secret`, each at most once, one space, then
  VALUE, its bytes to the line's end, nonempty; no other line (an empty one among them). A file
  that breaks it is a usage error, exit 1, nothing sent, naming the line's number and what is
  wrong (`tokens file line 2: unknown name`, `... a name twice`, `... no value`), never a value. A
  connect needs `api` and `git` (`tokens file names no git`, exit 1). The bundle is
  `temper_client_domain::forge::forges::pasted_tokens`.
- **`forge replace FORGE FILE --key KEY`**, a new tool after `forge connect` in the table: FILE
  whose first line is `-----BEGIN RSA PRIVATE KEY-----` is a key, sent as one `Pasted` item
  (`pasted_app_key`, as `github-app` sends one); any other FILE is a tokens file, with `api`,
  `git` or both (`tokens file names no token`, exit 1, without either); then `Replace { forge }`
  with the value carried; it prints `connecting` and `forge list --until connected:FORGE`, exit 0
  once durable; refused, exit 2. Its line: "Replace a forge's App key, or a Forgejo's tokens";
  its notes say what tools.md's row says, the tokens file's form, and that a value is never
  printed.
- `forge connect`'s notes: their last sentence becomes "`connected already` (exit 2): the forge
  holds a key or tokens; forge replace replaces them." and they gain the tokens file's form; the
  form-2 and new-form arguments' help words describe a tokens file, never "Not offered".
- The CLI shell reads FILE for form 2, the new form and `forge replace` (`arguments.rs`'
  `key_file`), within `PASTED_PAYLOAD` (`key_bytes`).
- `temper/docs/help/reference/ctl-forge.md`, the help goldens and `tests/help.rs`' `USAGES`
  match the table: the reference page and goldens regenerated with `SKEIN_CODEGEN_WRITE=1`, never
  by hand; `USAGES` gains `forge connect FORGE forgejo FILE --key KEY` as it is and the new
  form's line, and a `forge replace FORGE FILE --key KEY` line; `GOLDENS`' positions follow the
  table.
- The help pages, word for word (the lead's text; this departs from "Not yours: `docs/help/`"):
  - `temper/docs/help/forgejo.md`, *The mode.*: the sentence "Connected, a bot's tokens given by
    hand, is not offered: `forge connect FORGE forgejo` is refused." becomes "Connected, a
    user's two tokens given by hand in a tokens file, is temper help forge's."
  - `temper/docs/help/forge.md`: the bullet that begins "**Connected:** GitHub by its App"
    becomes "**Connected:** GitHub by its App; a Forgejo by its admin's token, or by a user's two
    tokens: `forge connect forgejo FILE`, FILE a tokens file, one `NAME VALUE` line each for
    `api`, `git` and, for hooks to temper's listener, `webhook_secret`; a managed Forgejo can make
    and adopt a first repository as it connects." Under *Limits*, the first bullet becomes "A
    forge's key or tokens are replaced by `forge replace FORGE FILE`: a GitHub App's new key, or a
    tokens file of the same Forgejo user. A second connect is refused with `connected already`
    (exit 2). Another user's tokens are another forge: removed and connected again, a forge is
    new, with a new number." and the bullet "temper does not connect a Forgejo by a user's token:
    only as its site admin." goes.
  - `temper/docs/help/github.md`, *If not:*: "`credential file must contain 1 to 4089 bytes`"
    names the bound ctl now prints, and "`connected already` (exit 2): the forge holds a key,
    which this build cannot replace." becomes "`connected already` (exit 2): the forge holds a
    key; `forge replace FORGE FILE` replaces it."
  - `temper/docs/help/party.md`: "`forge connect` reads a forge's key or token (temper help
    forge)" becomes "`forge connect` and `forge replace` read a forge's key or tokens (temper
    help forge)".
- `temper/docs/design/client/tools.status.md`, **Not yet**, 5.5: "area 6's `forge connect
  [FORGE] forgejo FILE`, refused as not offered (`forges/connection.rs`), and `forge replace` and
  `forge forget`, with no request on the wire." reads "area 6's `forge forget`, with no request
  on the wire."

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1 and C2.
- `temper/docs/design/client/tools.md`, 5.5 (area 6's table: `forge connect ... forgejo FILE`,
  `forge replace`; *A pasted value is read from FILE*; *No pasted value is echoed*), 6.3 (exit
  statuses), 15 (*A tokens file's layout*).
- `temper/docs/design/client/README.md`, 4.6 (`Replace`, `ForgeRefusal`).

## Owns and may change

- `temper-client-tools`: `src/forges.rs` (`CONNECT`'s form 2, its new form, its notes; a
  `REPLACE` tool), `src/forges/**` (`connection.rs`: the form-2 refusal at `:198-205` goes; the
  tokens file's parser, in a module of its own; `request()`, `owns`, `told` for the tokens connect
  and the Replace; a `replace` beside them), `src/table.rs` (the tool after `CONNECT`), and the
  tests: `tests/forge_connection.rs`
  (`unoffered_ways_exit_two_without_reading_or_sending_a_value` keeps the ways still not offered
  and loses form 2; new tests: a kept and a new tokens connect, their bundles decoded with
  `decode_pasted` to the file's items; each broken file, exit 1, naming its line, nothing sent; a
  replace by key and by tokens, their bundles; a replace refused `not connected`, exit 2; the
  `--json` of each), `tests/fixtures.rs` (argv), `tests/help.rs`, `tests/golden/**`.
- `temper-client-shell-cli/src/arguments.rs` (`key_file`, `key_bytes`) and `src/tests/keys.rs`
  for the new files' reads.
- The help pages and the status entry, as the acceptance gives them; the reference page
  regenerated.

Pre-approved: the table's tool indices after `forge connect` move by one; every positional list in
`tests/help.rs` (or a fixture) follows them. If `temper-docs`' rules refuse a help sentence above
(a placeholder, a word), keep its meaning in the nearest words they take and give both in the
report. No change to `temper-client-domain`, the wire or the view.

## Steps

1. The tokens file's parser and its tests: `heavy cargo nextest run -p temper-client-tools --lib -E
   'test(forge_connection)'`.
2. The connect's forms, the CLI shell's reads: the same, and `-p temper-client-shell-cli --lib`.
3. `forge replace` and its tests; the help regenerated: `heavy cargo nextest run -p
   temper-client-tools --lib`.
4. The help pages; `heavy cargo nextest run -p temper-docs`.
5. The status edit; the citations check.
6. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-tools -p
   temper-client-shell-cli --all-targets -- -D warnings`; their focused tests, and `heavy cargo
   check --tests` for `temper`, `temper-client-shell-terminal` and `temper-client-world`. Commit
   the code and tests, then the help and status text; push.

## Report

As the implementer's text says, with the regenerated files' `git diff --stat` and the table's
moved indices.

## Answers

2026-11-03. May the task change `src/jig/rows.rs` and `src/tests/landing.rs` only to shift tool
indices after `forge connect`? Yes: both are now in `files`. `REPLACE` stays right after
`CONNECT` (37) in `table::TOOLS`, as the acceptance says. In `jig/rows.rs`, every `*_AT`
constant past 37 (`EXPORT_AT` 40 becomes 41, and so on) goes up by one, with its comment, and
nothing else changes; what names the constants (`root.rs`, the CLI shell's `host.rs`,
`arguments.rs` and `host/input.rs`, the tests) follows without an edit. In `tests/landing.rs`,
`help(Some(56))` and `help(Some(57))` become 57 and 58, or name `rows` constants if there are
any. No other edit to either file. The report lists the moved indices.
