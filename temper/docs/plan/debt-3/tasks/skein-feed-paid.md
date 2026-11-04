+++
id = "skein-feed-paid"
part = "skein-json-feed"
kind = "docs"
needs = ["skein-forge-pumps", "skein-fake-forge-pumps", "skein-oauth-feed", "skein-world-pumps", "temper-json-pumps"]
files = [
  "skein/docs/development/debt/D001-json-whole-buffer-reads.md",
  "skein/docs/development/debt/README.md",
]
effort = "medium"
minutes = 10
+++

# Pay skein D001: the item and its row

## Goal

The item's sites have moved. Check its **Paid when** on the trunk, then delete its file and row.

Acceptance:

- `rg -c 'Tokenizer::new\(' skein/crates skein/testing skein/tests skein/examples -g
  '!**/skein-json/**' | awk -F: '{s+=$2} END{print s}'` reads 19 (41 at base). Those 19, which stay:
  15 read a body as it arrives (`skein-forge-connection` 5, `skein-forge-setup/src/host/connection.rs`
  2, `skein-fake-forge-protocol/src/server.rs` 2 and `src/server/git.rs` 1, `skein/tests/protocol` 3,
  `skein/tests/http` 2); 2 are the tokenizer's own tests (`skein/tests/json`); 2 feed a body in
  pieces (`skein-forge/tests/common/forgejo.rs`, `skein-fake-forge-protocol/src/tests/git.rs`).
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md` and its README row are gone;
  `tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`, whole.

## Owns and may change

- `skein/docs/development/debt/D001-json-whole-buffer-reads.md` (delete) and its row in
  `skein/docs/development/debt/README.md`. `skein/docs/development/debt/reviews/` stays as it is.

Pre-approved: if the count reads otherwise, list each site off that list. A site this plan's tasks
left behind by mistake is the cause: report it under `## Left` and still delete nothing; report
`Question:` with the sites.

## Steps

1. Run the count, and read the sites against the list.
2. Delete the file and its row. `python3 -I -B tools/citations/citations.py --base origin/main`
   and `--all` from the repository root.
3. Commit ("pay skein D001: the item and its row"), pushed.

## Report

As the implementer's text says, and also: the count.
