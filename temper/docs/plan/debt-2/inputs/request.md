# Request: pay four debt items, one per tree (second round)

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth:
each tree's `docs/development/debt/`). Four independent parts, one per tree, two sized S and two M. Each
part pays exactly one item, so that the item's **Paid when** holds, and nothing else. A part
that needs a design change, a breaking public interface change or a stored-format change stops
and asks (additions to a public interface are fine).

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the
  command reads 0 (or the item's target) after;
- keeps behaviour unchanged unless the item says otherwise; tests that pinned removed code go
  with it, and the rest pass unchanged;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in
  the same branch (the README's rule: git keeps the history);
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something
  else (for example a test that checks an old field is refused) is exempted, not changed.

## 1. jig D018: the native shell panics on requests only the unlinked client makes

`jig/docs/development/debt/D018-native-shell-unlinked-panics.md`. The linked client's requests
become a type without `Down` and `SignOut`, so the native shell's `panic!` arms go. If the split
changes a wire shape, stop and ask.

## 2. skein D046: skein-ui offers no `Limits` constructor and hides its node accounting

`skein/docs/development/debt/D046-ui-limits-and-accounting.md`. Add the constructor for a tree
without scenes and expose the per-node byte accounting, as additions to skein-ui; move skein's own
callers onto them. jig's and temper's callers stay as they are: they are jig D022/D023 and
temper's literals, paid in a later round (update those items' text to say D046 is paid, so they
no longer wait on it).

## 3. smith D002: three owners still hold lifecycles in flags and `Option`s

`smith/docs/development/debt/D002-lifecycles-in-flags.md`. `HostIo`, the agent service's `Stages`
and the local `Domain`'s delivery save each hold their lifecycle as a state enum with its own
transitions and `follow` (`skein/docs/foundation/programming-model.md`, 5.4). One part per owner is
fine. Behaviour and stored formats unchanged.

## 4. temper D005: the Forgejo file credentials live on for test fixtures

`temper/docs/development/debt/D005-forgejo-file-credentials.md`. Remove the file-credential path
from temper-service (`api_token`, `webhook_secret`, `file_access`, `file_api`, the refusals that name
gone fields), move the fixtures onto forge records, and close the two status entries the item
names. The settings document's accepted shape may shrink only where the item says the fields are
already gone from the design; otherwise stop and ask.
