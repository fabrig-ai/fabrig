+++
id = "link-close"
part = "transport"
kind = "docs"
needs = ["link-oauth", "link-llm", "link-forge-setup", "link-forge-connection"]
files = ["skein/docs/development/debt/D016-transport-four-copies.md", "skein/docs/development/debt/README.md", "skein/docs/design/link.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 20
+++

# skein: pay D016

## Goal

The four tasks before this one make D016's **Paid when** hold. Check that they did, and close the item.

Acceptance:

- `rg -n 'skein_link|skein-link' skein/crates/skein-forge-connection skein/crates/skein-forge-setup skein/crates/skein-oauth-accounts skein/crates/skein-llm-connection -g 'Cargo.toml'`
  lists all four.
- `rg -n 'skein_tls::client::Client|tls::Client' skein/crates/skein-forge-connection/src skein/crates/skein-forge-setup/src skein/crates/skein-oauth-accounts/src skein/crates/skein-llm-connection/src -g '!*test*'`
  finds only where a component builds the client it hands to `Link::new`.
- Each of the four has a pin of `close_notify` reaching a server: your report names them.
- `skein/docs/development/debt/D016-transport-four-copies.md` and its README row are gone, and `link.status.md` says it has no gap.
- `skein/docs/design/STATUS.md` holds no sentence about `skein-link`, and lists `link.md` among the documents with no gap.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`, `DESIGN.md`, 2.
- `skein/docs/development/debt/D016-transport-four-copies.md`; `skein/docs/design/link.md`.

## Owns and may change

- Owns the files of this spec's header. This departs from the implementer's text for those status files.
- If a component misses a check, don't mend it here: report `Question:`.
- `D020` names D016 in prose: fix a broken path, and leave the rest of it.

## Steps

1. The acceptance searches.
2. Delete the item and its row; `link.status.md`'s date line and "No gap."; `STATUS.md`.
3. `python3 -I -B tools/citations/citations.py --all`. Commit and push.

## Report

The common note's items. The four pins by name.
