+++
id = "link-decide"
part = "link"
kind = "feature"
needs = ["link-record"]
files = [
  "temper/crates/temper-engine-domain-forge/src/link.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/link.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 70
+++

# The forge connector keeps, replaces and clears its link

## Goal

Give `Event::Owner` and `Event::PersonOf` their bodies, as `temper/docs/design/domain/forge.md`, 4.6 says: the connector remembers the deployment's sole owner the root hands it at each start, clears every link whose person isn't that owner, and makes, keeps, replaces or clears a forge's link from what a setup run read of the person, under a box alone, never to temper's writer, logging what the section names. Adoption skips the link's user. The types are `link-record`'s (`notes/contracts.md`, *The link*).

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), because the design already says what this task builds.

Acceptance:

1. A new module `src/link.rs` holds the two bodies, dispatched from `domain.rs`'s `Event::Owner` and `Event::PersonOf` arms (replacing `link-record`'s no-ops); the owner the root handed is volatile, a field of `Domain`, `None` until handed.
2. One new table test, `a_link_follows_the_sole_owner_and_what_the_forge_says` in `src/tests/link.rs`, each row a starting link, the connector's `under_box`, the forge's kind and mode, the owner handed, one `PersonOf` (or none), and what follows: the record's `person` after, whether a `Request::Save` of the record went out, and the `Request::LinkNoted` notes. Its rows:
   - under a box, owner P, an owned Forgejo, `Is(7)`, no link: link `(7, P)`, saved, no note;
   - the same again with link `(7, P)`: unchanged, nothing saved, no note;
   - link `(7, P)`, `Is(9)`: link `(9, P)`, saved, `Replaced`;
   - link `(7, P)`, `Gone`: unchanged, nothing saved, `Gone`;
   - a GitHub forge, link `(7, P)`, `NotTheirs`: no link, saved, `Cleared`; with no link, `NotTheirs`: nothing saved, no note;
   - a GitHub forge, `Is(7)`, no link: link `(7, P)`, saved;
   - `Is(W)`, W the forge's retained writer: unchanged, nothing saved, `Writer`;
   - not under a box: `Is(7)` makes no link, nothing saved, no note;
   - no owner handed (or `Owner { party: None }`): `Is(7)` makes no link;
   - a managed or fake Forgejo: `Is(7)` makes no link;
   - link `(7, P)`, then `Owner { party: Some(Q) }`: no link, saved, `Cleared`; `Owner { party: None }` likewise; `Owner { party: Some(P) }`: unchanged, nothing saved.
3. Adoption's collaborators skip the link's user beside temper's writer (`adoption.rs`, `human_collaborators`), a new row of the adoption table in `src/tests/adoption.rs` (or of the test there that holds the writer's exclusion): a forge whose link names user 7 returns no collaborator 7.
4. The entry *4, 4.6: adoption seeds the link's user, and nothing keeps, replaces or clears a link ...* is gone from `temper/docs/design/domain/forge.status.md`, the others unchanged.
5. Each rule bites once: drop the writer check, the `under_box` check, and the owner comparison, each in turn, and see its row fail.
6. Focused tests pass in `temper-engine-domain-forge` and `temper-engine-forge-world`; clippy clean for `temper-engine-domain-forge`. This changes what a shared item means (the record's `person` now moves): run `temper-engine-domain`'s focused tests too.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The link*: it binds.
- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/domain/forge.md`, 4.6, whole, and section 4's *People the forge knows*.
- `temper/docs/design/domain/root.md`, 9.1 (when the root hands the owner).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** `link-record`'s `forges.rs` and its `save` path are read, not changed: a link changed is the whole record saved through the module's existing save (as `connect::bot_login` saves a changed record). If `forges::save` needs a one-line `pub(crate)` to be called from `link.rs`, make it.

**Options, each with its default:**

- **Which forges link:** default, a Forgejo of `Mode::Owned` and a GitHub forge of any mode, kept (not ended); every other forge is told nothing and answers nothing.
- **Owner arriving during the start:** the root hands `Owner` in the start's link step, before `Restored` (`root.md`, 9.1). If a `Request::Save` can't go out before `Restored`'s decision (the restore asserts no write, or the root's start drops it), keep the owner when it arrives and clear the links in `restored`'s decision, and say so; the row of acceptance 2 then sends `Restored` after `Owner`.
- **Two links naming one person on two forges** are fine: one per forge, each set alone.
- **A `PersonOf` for a forge the connector doesn't keep:** nothing, no note.
- **A link replaced while the old user is someone's seed:** nothing more; the lookup shadows the new user's seed, and the old one is a stranger again or its seed (4.6, *A link replaced*).

## Steps

1. `link.rs`: `owner(d, party, out)` and `person_of(d, forge, answer, out)`, each saving the changed record and pushing `Request::LinkNoted` where 4.6 says it's logged.
2. The dispatch arms in `domain.rs`; `human_collaborators`' skip.
3. The table test and the adoption row; the bites.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-forge-world -p temper-engine-domain`. Commit ("temper-engine-domain-forge: keep a forge's link from the sole owner and what the forge says of the person"), push.

## Report

As the implementer's text says, with each bite's broken rule and failure.
