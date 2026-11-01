+++
id = "cl-ctl"
part = "rf"
kind = "feature"
needs = ["cl-wire"]
files = [
  "temper/crates/temper-client-tools/**",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/help/reference/ctl-playground.md",
]
effort = "high"
minutes = 90
+++

## Goal

`temper ctl` makes, removes and resets: `forge make`, `forge remove` and `playground
reset`, each one keyed request of the forge family (`notes/contracts.md` T4 to T6, T8),
printing its answer in text and `--json` as the other forge tools do. The process tier and
agents then make a forge by its kind and number it from the answer, instead of naming the
document's forge 3.

- `forge make here|github [--key KEY]` and `forge make forgejo API GIT [--web WEB] [--key
  KEY]` send `make_forge`; they print `made: forge N` (exit 0), `refused: full` or
  `refused: not offered` (exit 2), `unknown` (exit 1), or the entrance refusal as other
  tools print it.
- `forge remove FORGE [--key KEY]` sends `remove_forge`; it prints `removing: forge N`
  (exit 0), `refused: no such forge` or `refused: the playground's forge` (exit 2),
  `unknown` (exit 1).
- `playground reset [--key KEY]` sends `end_playground`; it prints `reset` (exit 0),
  `refused: not offered` (exit 2), `unknown` (exit 1). `playground start` is unchanged.
- A call kept by the engine and printed again after a restart of the tool (`Told::Kept`)
  prints as `kept: forge-make ...` / `forge-remove` / `playground-reset`, as
  `demo-start`'s does.
- `temper ctl help`'s goldens and the two reference pages generated from the table match.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4 (MakeForge),
  T5 (RemoveForge), T6 (EndPlayground), T8; `notes/cl-wire.md` 1 to 3 (the wire's names and
  each answer's outcome).
- `temper/docs/design/client/tools.md`, 1 and 5.5 (area 6, the forge's tools).
- The draft `temper/docs/design/draft/playground-reset.md`, sections 3, 4 and 6.2.
- The existing tools to follow: `src/forges.rs` (`DEMO_START`, `SET_UP`, `send`, `told`,
  kept restore), `src/forges/connection.rs` (a forge call with its own module), `src/demo.rs`
  (an answer's text and JSON).

## Owns and may change

Owned: `temper/crates/temper-client-tools/` (its `src/`, `src/tests/`, `src/tests/golden/`).

May change, generated alone: `temper/docs/help/reference/ctl-forge.md` and
`temper/docs/help/reference/ctl-playground.md`, rewritten by `SKEIN_CODEGEN_WRITE=1` and the
`temper` package's `every_ctl_group_page_is_what_the_table_renders` test from the table; no
hand edit of any help page (the lead's, `notes/cl-progress.md`, request 4).

Not yours: `temper-client-domain` (build every ask and decode every answer from the wire,
`temper_client_protocol_wire::v1`, in tools; take bounds from
`temper_client_domain::forge::wire_limits` as `connection.rs` does), the binary `temper`
(only its test is run).

Options, pre-approved:

- If the table's argv kinds cannot say `here|github|forgejo` followed by forgejo's two
  positionals, make two forms of one tool (as `forge connect` has four): `forge make
  here|github [--key KEY]` and `forge make forgejo API GIT [--web WEB] [--key KEY]`.
- The JSON records' names: `forge-make`, `forge-remove`, `playground-reset`; their outcome
  keys `made` (`{"forge": N}`), `full`, `not_offered`, `removing` (`{"forge": N}`),
  `no_such_forge`, `playground`, `reset`, `unknown`, `entrance_refused`, in the shape
  `demo.rs`'s JSON has.
- `forge remove`'s `no such forge` reads the wire's `connect_refused { why: no_such_forge }`
  (`notes/cl-wire.md`, 3); any other outcome for these three calls prints `unknown` (exit 1).

## Steps

1. **Table rows** in `src/forges.rs` beside the others, area 6, `Keyed::Optional`:
   - `MAKE`, name `forge make`, line "Make a forge by its kind, to set up or connect by
     the number it is given", prints "`made: forge N`, then `forge setup N` or `forge
     connect N` to make", result `forge-make`; positional `kind` `here|github|forgejo`,
     for forgejo positionals `api` (`API`, "The Forgejo's API address, HOST:PORT", at most
     259 bytes) and `git` (`GIT`, "Its git address, a URL", at most 1024), flag `web`
     (`WEB`, "Its web address, a URL, when not the git address's", at most 1024); notes:
     "here is the Forgejo temper box runs on this machine; github is GitHub through
     temper's App. `refused: not offered` (exit 2): this deployment offers no such forge
     (here needs temper box). `refused: full` (exit 2): the deployment keeps as many forges
     as it may; remove one first.\n".
   - `REMOVE`, name `forge remove`, line "Remove a forge whole: its records, its
     repositories from their projects, its box group", prints "`removing: forge N`, then
     `forge list` no longer lists it", result `forge-remove`; positional `forge` (`FORGE`,
     1 to 65535); notes: "Its changes are cancelled. An owned Forgejo goes with its
     directory; a managed Forgejo's bot and hooks and a GitHub App stay on the forge, to
     delete there. The playground's forge is removed by playground reset alone.\n".
   - `PLAYGROUND_RESET`, name `playground reset`, line "Leave the playground: its project,
     its forge and its account removed", prints "`reset`", result `playground-reset`;
     notes: "It returns once the playground is gone; the store then holds no playground,
     as before it started. `refused: not offered` (exit 2): no playground runs.\n".
   Put them in `table.rs`' `TOOLS` after `ACCEPT` (make, remove) and after `DEMO_START`
   (reset); the `forge` group's line becomes "Forges: read them, make one, set one up,
   connect it, accept an installation, read what it reaches, remove one", the
   `playground` group's "The playground: start it, or leave it".
2. **A module** `src/forges/making.rs` with `owns(call) -> bool`, `send(state, client,
   env, called, out)` and `told(state, env, told, out) -> bool`, called from
   `forges::send`/`forges::told` as `connection`'s are, keeping its pending calls by key
   in `Forges` (a map of its own, as `connection` keeps its). The asks: `w::Request::
   MakeForge(w::MakeForge { choice })`, `w::Request::RemoveForge(w::RemoveForge { forge
   })`, `w::Request::EndPlayground`, deployment scope, the call's `--key` named as
   `forges::send` names it. A forgejo `api` or `git` that is empty is refused before
   sending ("invalid forge call", exit 1), as `decode_requesting` would refuse it.
   `Told::Kept` restores the call from the kept request's bytes (decoded with the wire),
   so its answer prints with `kept: `.
3. **Text and JSON** of each answer in `making.rs` (or a `src/making.rs` beside
   `demo.rs`), in `demo.rs`'s style: text lines as the Goal says; JSON with the outcome
   keys of the options.
4. **Tests** in `src/tests/forges.rs` (or a new `src/tests/making.rs`), each through the
   tool's root as `demo_start_answers_print_their_words_json_and_status` does:
   `forge_make_sends_each_kind_and_prints_made_with_its_number`,
   `forge_make_refusals_print_and_exit_two`,
   `forge_make_forgejo_refuses_an_empty_address_before_sending`,
   `forge_remove_prints_removing_and_its_refusals`,
   `playground_reset_prints_reset_and_not_offered`,
   `a_kept_make_prints_again_with_kept`. Add help goldens `help_forge_make.txt`,
   `help_forge_remove.txt`, `help_playground_reset.txt` to `GOLDENS`, the usages to
   `USAGES`, and regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   temper-client-tools --lib -E 'test(/tests::help::/)'`, then without the variable.
   `heavy cargo nextest run -p temper-client-tools --lib`.
5. **Reference pages:** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E
   'test(=tests::commands::every_ctl_group_page_is_what_the_table_renders)'`, then without
   the variable; `git diff --stat temper/docs/help` shows the two pages alone. Then `heavy
   cargo nextest run -p temper-docs` (the compiled pages' bounds).
6. `cargo fmt`; `heavy cargo clippy -p temper-client-tools --tests`; old forms; commit and
   push.

## Report

- The usage lines as printed, and the two reference pages' diff in short.
- Whether the table took the forgejo form as one tool or two.
