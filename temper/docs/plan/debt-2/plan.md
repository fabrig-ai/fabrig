# debt-2: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt items, one per tree, each paid
on its own branch so that its **Paid when** holds. The four read none of each other's code, so
each is its own part and slice, released as soon as its tasks are written. `notes/common.md`
holds the standing rules every task names; `notes/contracts.md` the shapes that cross jig
D018's tasks.

## Before the tasks

This cut wrote the design each task follows, pushed with the specs:

- **skein, `ui.md` 15:** `Limits::EMPTY` (no capacity: an empty tree without scenes, which a
  caller sets the fields it means over) and a public `Node::text_bytes` (the bytes the
  builder charges a node); the table's `text_per_node` row names the image address the code
  already counts. `ui.status.md` records both as **Not yet**.
- **jig, `client/domain/client.md` 4.4 (new):** without a link. `Client::unlinked`'s client is
  stepped by its own pair, `step_unlinked` and `fire_unlinked`, and asks `UnlinkedRequest`
  (`Told`, `Authorized`, `Down`, `SignOut`); a linked client's `Request` holds no `Down` and
  no `SignOut`; a pair given the other mode's client is a fault; an application root keeps
  the two modes' requests under their own variants. `client.status.md` records it as **Not
  yet**.
- **temper:** no design text changes. `protocol/configuration.md` (1, 3) already says the
  document names no forge and holds no secret, and `protocol/credentials.md` (2.1, 5) that a
  forge's access is its records'; the two status entries D005 names record the code's side.

## Parts and tasks

1. **skein-ui-limits** (skein D046; `skein/docs/design/ui.md, 15`), slice 1.
   - `tasks/skein-ui-empty-limits.md`, medium, ~45 minutes, needs nothing: the constant and
     the public count; skein's two literals and `memory.rs`'s two hand counts move onto them;
     the status's 15 words, the item and its row go; jig's D022 and D023 stop waiting on it.
2. **jig-client-modes** (jig D018; `jig/docs/design/client/domain/client.md, 4.4`), slice 2.
   - `tasks/jig-client-unlinked-requests.md`, interface, high, ~50 minutes, needs nothing:
     `UnlinkedRequest`, `step_unlinked`, `fire_unlinked` as additions in `jig-client`; `step`
     and `fire` keep serving both modes through one private mapping.
   - `tasks/jig-unlinked-callers.md`, high, ~110 minutes, needs the first: jig's two roots
     (the test domain, ops' client domain) gain `Unlinked { request }`, and every jig kit and
     world that steps an unlinked client uses the new pair.
   - `tasks/temper-unlinked-callers.md`, high, ~120 minutes, needs the first, beside the jig
     one: the same for temper's two roots (`temper-client-domain`, `temper-client-tools`'
     `Root`), its linked hosts' arms for `Unlinked`, and its tests and gallery.
   - `tasks/jig-linked-requests.md`, high, ~45 minutes, needs both callers: `Request` loses
     `Down` and `SignOut`, `step` and `fire` serve the linked mode alone, the native shell's
     four `panic!`s go (two arms deleted, two impossible cells as `unreachable!`), the
     status's 4.4 words, the item and its row go.
3. **temper-forge-records** (temper D005; `temper/docs/design/protocol/credentials.md, 5`),
   slice 3.
   - `tasks/temper-fixtures-records.md`, high, ~130 minutes, needs nothing: every
     `temper-service` fixture connects its forges by records (rows and secrets seeded from
     fixture constants), none by `api_token` or `webhook_secret`; product code untouched.
   - `tasks/temper-file-credentials-gone.md`, high, ~80 minutes, needs the first: the fields,
     `file_access`, `file_api`, the credential members' refusal paths and the harness's
     `old_form` go; `credentials.status.md`'s Departs goes; `configuration.status.md`'s
     Departs 6 is replaced by what still names the document's forges; the item and its row.

The smith part (D002) follows once its designer's batch holds together.

## Critical path

So far: jig's chain, `jig-client-unlinked-requests` → `temper-unlinked-callers` →
`jig-linked-requests` (~50 + 120 + 45 = ~215 implementer-minutes); temper's chain
`temper-fixtures-records` → `temper-file-credentials-gone` (~210) runs beside it, and
`skein-ui-empty-limits` (~45) alone.

## Overlaps

Checked against the brief's `Drafts beside this one`, 2026-11-04:

- `skein-ui-empty-limits` and jig D018's tasks touch no file a draft beside this one is
  changing.
- `temper-file-credentials-gone` cannot avoid fabrig-scale's unmerged tasks: one line of
  `temper-service/src/tests/limits.rs` (`brief-room`), the `old_form` hunk of
  `temper/crates/temper/tests/support/owned.rs` (`init-standard`, `reference-trim`), and one
  sentence of `protocol/configuration.status.md`'s Departs entry (`brief-room`,
  `init-standard`, `reference-trim`, `stock-scale` edit its other entries). The item names
  those files, so the overlap is the item's. `temper-fixtures-records` keeps off
  `tests/limits.rs` and `tests/startup.rs` for that reason.
- Within this plan, `temper-unlinked-callers` and the two D005 tasks share no file:
  `temper/tests/client/src/fixtures.rs` and `temper-service/src/tests/` but
  `setup_github_render.rs` are D005's.

## Calls made

- **`Limits::EMPTY` is the constructor** D046 asks for: a `pub const` base that callers
  update with struct syntax (`Limits { nodes, depth, ..Limits::EMPTY }`), as skein's own
  `limits.rs` test already writes its cases. A function of the ten tree fields would make
  callers spell all ten again, positionally; a `Default` would hide that zero is a capacity.
- **D046's Measure reads 32 after, not 0:** the request keeps jig's 25 and temper's 7
  literals for a later round (jig's D022, temper's literals); skein's 2 go.
- **D046's accounting is `Node::text_bytes`,** the builder's own count made public, not a new
  function: it already is the one the builder charges, and skein's own `memory.rs` test
  copies it twice.
- **D018 is split by mode in the client, not in its callers' roots.** The survey found the
  application roots (two in jig, two in temper) choose the mode at run time and wrap the
  client's requests in their own; splitting each root by mode is temper-scale and beyond the
  item. So each root gains one `Unlinked` variant (`notes/contracts.md`), and temper's two
  linked hosts that handed `Down` and `SignOut` to the native shell, which panicked, now
  reach an `unreachable!` for `Unlinked` instead: the same fail-stop, outside the native
  shell. No wire shape changes: `jig_client::Request` crosses none.
- **D018 is staged in four tasks,** so each branch stays in its tree and the trunk builds
  between them: the additions first, then jig's and temper's callers side by side, then the
  removal. One task across both trees would be ~150 minutes on the path but a branch in two
  trees.
- **D018's other two `panic!`s become `unreachable!` with their reasons** (`shell.rs`, io
  events the shell's io never asks for; `files.rs`, a file terminal with no request in
  flight): the Measure counts them, and programming-model.md 5.4 spells an impossible cell so.
  A type that makes the io case unrepresentable would reach `skein-io` or temper's CLI
  transport; the task may make either unrepresentable inside the native shell alone.
- **D005's refusals:** every path naming a forge's credential members goes (`api_token` and
  `app_key`, as the Paid when says, and `webhook_secret` and `user`, which
  `credentials.status.md`'s entry names too); the arms for skein's access refusals become one
  `unreachable!`, since no settings temper prepares carry an access. The other
  `application.forges[N]` paths (`api`, `log_origins`, `number`, and the startup checks'
  `application.forges`) belong to `forge::settings`' document-forge list, which the product
  passes empty and the fixtures use to give their forges' endpoints; moving every fixture's
  endpoints onto the shipped path as well would move about a hundred tests' starts. So
  `configuration.status.md`'s entry 6, which named both, is replaced by one naming that list
  alone, a narrower gap the code still has.
- **D005 keeps `owned::adopt`:** it is not a no-op (it adopts the repository through the bot
  for the process worlds), and the Paid when does not name it; D008 rewords its comment.
  `startup::Forge` loses its two credential fields too: nothing constructs or parses them.
- **No D005 document shape shrinks:** the document already refuses `application.forges` as
  an unknown member, so the request's stop-and-ask does not apply.
