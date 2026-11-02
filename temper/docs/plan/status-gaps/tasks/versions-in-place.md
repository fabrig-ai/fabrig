+++
id = "versions-in-place"
part = "versions"
kind = "docs"
needs = []
files = [
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/protocol/store.status.md",
  "temper/docs/design/domain/forge.md",
]
effort = "medium"
minutes = 15
+++

# temper's versions stay at 11 in place: the status entries go

## Goal

`temper/docs/design/protocol/README.status.md` (9) and `store.status.md` (4) record as a
departure that `STORE_VERSION` and `CLIENT_VERSION` are 11, "moved before the first release
where the rule keeps them in place". The design settles the value: both stay at their current
number, 11, until the first release (`temper/docs/design/client/README.md`, 4.6: "It stays at 11
until the first release"; `temper/docs/design/protocol/README.md`, 9: "each of temper's three
versions keeps its current number"; `protocol/store.md`, 4: `STORE_VERSION` "stays at its current
value"). The code holds 11 (`temper-protocol-store/src/lib.rs`, `STORE_VERSION: u32 = 11`;
`temper-client-protocol-wire/src/lib.rs`, `CLIENT_VERSION: u32 = 11`), and the schema has
changed in place at v11 since (`git log -- temper/crates/temper-protocol-store/schema/`). The
entries record history, which a status never holds (`AGENTS.md`, 9): no code changes and no
stored format moves. Remove them.

This is the lead's call on the request's item 4: the design settles the values, so no question
is asked and no number changes.

Acceptance:

- `README.status.md` keeps its other entries word for word; its "Departs" list loses the
  sentence that begins `9: \`STORE_VERSION\` and \`CLIENT_VERSION\` are 11` (through
  `temper-client-protocol-wire/src/lib.rs\`).`), and nothing else.
- `store.status.md`'s "Departs" loses its last sentence, `4: \`STORE_VERSION\` is 11, moved
  ...by name.`; the entry `2, 4: no \`Demo\` (17) ...` stays word for word.
- `temper/docs/design/domain/forge.md`, line ~422: `in temper's small store under its tag 16
  (\`STORE_VERSION\` 3), and` loses its stale parenthesis, `(\`STORE_VERSION\` 3)`, and nothing
  else of the file changes (the number is `protocol/store.md`'s to state, and it is 11).
- Both status files' date lines are today's (`date -u +%F`).
- No code, golden, schema or other document changes.

## Inputs

- `temper/docs/plan/status-gaps/notes/common.md`.
- `AGENTS.md`, 9 (a status's form).
- The design sections quoted above, read only.

## Owns and may change

- `temper/docs/design/protocol/README.status.md` and `store.status.md`: the sentences above and
  the date lines.
- `temper/docs/design/domain/forge.md`: the one parenthesis above. Other branches change this
  file far from line 422 (sections near line 1171-1216); touch nothing else in it.
- `temper/docs/design/STATUS.md`: its summary names no version departure (checked); if
  `rg -n 'STORE_VERSION|CLIENT_VERSION|version' temper/docs/design/STATUS.md` shows one, drop that
  clause only and say so in the report.

Pre-approved: if another document under `temper/docs/design/` still states a temper version
number other than 11 for `STORE_VERSION` or `CLIENT_VERSION` as true now
(`rg -n 'STORE_VERSION|CLIENT_VERSION' temper/docs/design`), list it in the report under
`## Left` and do not edit it. `client/README.md` 4.6's list of what each `CLIENT_VERSION` number
was given for is design text the designer keeps: leave it.

## Steps

1. Make the three edits; set the two date lines.
2. `git diff --stat` shows the three files alone.
3. No build and no test: nothing compiled or read by a test changes (temper's tests read
   `docs/help/`, not these files; check with `rg -n 'README.status|store.status|domain/forge.md'
   temper --glob '*.rs'` and, if a test reads one, run that test and report it).
4. One commit, saying why (the design keeps both versions at 11 until the first release; the
   entries recorded history); push your branch.

## Report

As the implementer's text says, and also the `rg` results of the two checks above.
