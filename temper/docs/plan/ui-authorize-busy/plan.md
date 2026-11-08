# ui-authorize-busy: plan

This is red-72's repair, after red-71, for the same story. `temper-client-browser-world`'s `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` fails at its last step under load. After the server's restart, the Authorize page draws "temper couldn't sign you in just now" (`Word::AuthorizeBusy`), not the forge's "Authorization received". A failing run takes about 12 s and a passing one about 2 s, and one rerun alone failed too. So it's load and time, not order (`temper/docs/plan/ui-authorize-busy/inputs/request.md`).

## Parts and tasks

- **repair** (direct, slice 1, no walk): task `repair` (`tasks/repair.md`), high effort, about 180 minutes, needing nothing. It finds which answer draws the busy page and the bound behind it, and holds that bound in a focused test, red first. It decides product or story at the lowest place and fixes it there. The story keeps its checks. The task runs the story by name, alone, under a CPU load it makes: up to five runs before the fix, and at least five after it, all passing.

## Critical path

`repair` alone. It starts at the first push, and the plan lands with it.

## Landed decision

The repair is the product's, in `jig-client-protocol-link`. Logical
cancellation stopped counting watch openings and cleanup exchanges while
they still occupied the browser's fetch table. Authorization could then
be lost locally before any authorize HTTP request reached the server.
Every emitted exchange now keeps transport credit until `Reply` or
`Lost`, so temper's existing first-send wait holds authorization for room.
The focused stream table holds that boundary with both successful and
failed late HTTP terminals. The browser story's checks are unchanged.

The provider-readiness hypothesis below was not the cause repaired. No
250 ms retry or 20 s provider wait was added. People's startup refusal,
including the mint and approval paths, remains separate work in
`jig/docs/design/protocol/client.md`, section 15; the provider's existing
departure is recorded in its status, section 18.

## Planning assumptions

- **One task, as the request asks.** The cause is settled only by reading the restarted server's log under load, and this call runs on a machine that builds nothing. Splitting the work would only guess.
- **The 10 s is likely the story's own `WITHIN`,** not a product deadline. `WITHIN` is 10 s (`temper/tests/client-browser/src/lib.rs`), and it bounds the step that fails. 12 s is the passing run's 2 s plus that wait. The busy page can come at once, and the step then waits out `WITHIN` for a heading that never comes. The spec says to check this from the log's times before hunting for a 10 s deadline.
- **The likeliest cause, from the lead's reading, is people's start after the restart.** jig's provider asks people `Whose`, and passes people's `Again::NotReady` on as *not ready* (`jig/crates/jig-protocol-engine-clients/src/provider.rs`, `authorize_reply`). People answers *not ready* until its restore opens. The exchange's credential is checked against the secrets store, not people, so the client's link can be live first. A loaded machine stretches the start until the authorization lands inside it. jig's design gives `Authorize` *not ready* only "while the engine stops" (`jig/docs/design/protocol/client.md`, 18): the code departs from it. This is unverified, and the spec lists the other sources of *lost* and *busy* with a pre-approved fix for each.
- **The decision leans to the product's.** A person whose box has just restarted can press "Sign in with temper" on Forgejo at once, as the story does. So a box that is only starting must not refuse the sign-in (the request's rule). The default fix is the provider's: it waits for people's start, asking `Whose` again every 250 ms for up to 20 s, then answers *not ready*. 20 s stays below the server's 30 s connection stall bound. People's own *not ready* before its restore opens stays, as its design and test hold. The story-only option exists for a cause that a person's box never meets: the story then visits temper's page and waits for `welcome()` after the restart.
- **The focused test lives in jig's engine-clients world.** `authorization_while_stopping_is_not_ready` becomes a table: stopping (*not ready* at once), people starting then open (*go*), and people starting past the wait (*not ready*). Its second row is today's red.
- **The load is a throwaway CPU burner,** `nproc` busy loops for the length of each run. The request's example, a parallel focused build, would put a second heavy command beside the story's, and the lab's rules allow one at a time.
- **No design is written ahead.** The design already says what the fix makes true (`jig/docs/design/protocol/client.md`, 18; `temper/docs/design/protocol/sign-in.md`, 9.3). The task makes three small edits, as the request allows: it extends client.md 18's *not ready* sentence with the wait, and replaces red-71's sentences in `temper/docs/design/protocol/sign-in.status.md` (9.3) and `temper/docs/design/STATUS.md` with the cause found.
- **No `notes/common.md`.** With one task, the request's `For every task of this plan` section serves as the common note, and the spec cites it.
- **A protocol's meaning changes freely here** (the request's `For every task of this plan`): the provider's *not ready* now means stopping, or past the wait. No compatibility is kept, and no question is asked.
- **Left outside this plan:** the same pass-through of people's *not ready* happens in two other places. One is the Google return's mint (`jig/crates/jig-protocol-engine-clients/src/mint.rs`, `MintRefusal::NotReady`), whose 503 shares the busy page's words. The other is the approval sign-in (`approval.rs`). The story reaches neither. If the cause found is people's start, these are the next places a person could meet the same refusal, and the task's report names them for after-landing.

## Overlaps

These overlaps could not be avoided:

- **`ui-ask-drop/close`** owns `temper/docs/design/STATUS.md` and `temper/docs/design/**/*.status.md`. This task edits only red-71's sentences in `sign-in.status.md`, 9.3, and the matching sentence in `STATUS.md`.
- **The other options' overlaps.** The default fix avoids both:
  - *lost* through temper-service's admission would edit `temper/crates/temper-service/**`, which `ui-ask-drop/stock` changes, and near `core-after-project/why-log`'s temper-service files;
  - the client link's own *busy* or *lost* would edit `jig/crates/jig-client-protocol-link/src/component.rs`, `sign_in.rs` or `jig/tests/client/tests/suite/authorize.rs`, which `tui-link-colours/jig-expiry` changes.

  The spec allows the smallest edit there, named in the report. Its default home for a link fix is the link's `authorize.rs`, which no draft changes.

The default fix's files, `jig/crates/jig-protocol-engine-clients/src/**`, `jig/tests/engine-clients/**` and `jig/docs/design/protocol/client.md`, are changed by no draft beside this one that is still open. `ui-account-again/redeclare` touched the provider's crate (`translate/accounts.rs`) and is merged.
