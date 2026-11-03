# Request: pay four small debt items, one per tree

From fabrig/debt, 2026-11-03 (the user's continuous tech-debt process; source of truth:
each tree's `docs/development/debt/`). Four independent parts, one per tree, each sized S. Each
part pays exactly one item, so that the item's **Paid when** holds, and nothing else. A part
that needs a design change, a public interface change or a stored-format change stops and asks.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the
  command reads 0 (or the item's target) after;
- keeps behaviour unchanged unless the item says otherwise; tests that pinned removed code go
  with it, and the rest pass unchanged;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in
  the same branch (the README's rule: git keeps the history);
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0).

## 1. jig D019: copied lint configuration

`jig/docs/development/debt/D019-lint-config-copied.md`. Test crates and the two shells copy their
`[lints.clippy]` block (40 manifests, 16 variants); 8 allow the `disallowed_*` lints. One shared
table per kind of crate; a world's real exception becomes a scoped `#[expect(..., reason = ...)]`;
no manifest allows a `disallowed_*` lint; each extra `clippy.toml` says why it differs or goes.
Only manifests, `clippy.toml` files and scoped `#[expect]`s change. If a world truly needs wall
clock or threads, stop and ask rather than allow the lint.

## 2. skein D024: the echo example keeps its stop in flags

`skein/docs/development/debt/D024-echo-template-flags.md`. In `skein/examples/echo/protocol/`,
the layer's shutdown and the connection's `ended`, `closed` and `call` flags become states, as
`skein/docs/design/programming-model.md` section 5.4 asks. The example is the template every service
copies, so its shape should read as the model's. Its tests keep passing unchanged.

## 3. smith D004: the session decides on a tool's name

`smith/docs/development/debt/D004-session-knows-wait.md`. No code in `smith-domain-session` matches
a tool name (against `smith/docs/design/domain/session.md`, section 1): the opener names the withheld
call in its opening, and `Event::Answered` and `Event::AnsweredAndYield` become one event. If those
events are stored or cross a wire, stop and ask before changing their shape.

## 4. temper D006: `temper forge reconcile` is dead

`temper/docs/development/debt/D006-forge-reconcile-dead.md`. The command only refuses, and its
524-line host has no product caller; `setup.orgs` is dead too. Remove the command, the
`reconcile` module of `temper-forge-setup-shell`, and the `setup.orgs` / `orgs` limits and setup
places, and close the three status entries the item names.
