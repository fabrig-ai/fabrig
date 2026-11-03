+++
id = "open-default-box"
part = "open-default-box"
kind = "feature"
needs = ["web-rows"]
files = [
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper/tests/fake_backends/walk/here_key.rs",
  "temper/crates/temper/tests/fake_backends/walk/open_default.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 90
+++

# On the fake box, Open Default by the browser's client bounds, then a reload that still receives

## Goal

No test walks the welcome's Open Default against a real `temper box`. Add it to the owned
welcome walk on `box --fake-backends` (the process tier's
`forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`): once the walk has reached Ready,
a native client built with the browser's client bounds presses Open Default and reaches the
Chats page, and a second such client (the reload) still receives its watches. It fails with the
web client's old `rows: 3` and passes on the trunk, where `web-rows` made it 8.

Acceptance:

- The walk's live check runs two native-client sessions against the running box, each with
  `Limits { client: temper_client_shell_web::limits::client(),
  ..temper_client_shell_terminal::limits::native() }`:
  1. *The press:* at `Address::Welcome { step: Step::Ready }`, "Open Default" shows enabled;
     pressed, the client reaches the Chats page (heading "Chats", level 1) with Default the
     chosen project.
  2. *The reload:* a fresh client shows Status "Live" and no Status "Behind" on the Chats page,
     the walk's forge on the Forges page, and the walk's model key's account on the Accounts
     page, then Live and not Behind again.
- Run with `rows: 3` restored in `temper-client-shell-web/src/limits.rs` (an uncommitted edit,
  reverted after), the story fails in session 1 or 2 at a step that waits on a delivered watch,
  and its message names the page and what was missing. With the trunk's `rows: 8` it passes.
- The story's CPU stays within `[process-temper]`'s 60 s per test (`tools/tiers/budgets.toml`);
  the figures before and after this change are reported.
- The `2:` item of `testing.status.md`'s **Not yet** is gone.

## Inputs

- `temper/docs/plan/open-default/notes/common.md`: the fault and the rules.
- `temper/docs/design/domain/testing.md`, 2: the paragraph ending *... the walk's forge on the
  Forges page and its key on the Accounts page.* (this cut wrote its last sentence); 2.6 for
  the process tier.
- `temper/docs/design/client/README.md`, 2: the paragraph *Both shells receive eight entries
  ...*.
- `temper/crates/temper/tests/support/playground.rs`: `Judge` and `reload`, the model for a
  native client spawned in a `real::World` against a running box, signed in by ctl's kept
  token (`Start::Token`), its pages read from `client.tree()` (read only).
- `temper/tests/client/tests/suite/welcome.rs` (the press of Open Default and the Chats page
  after it) and `forges_edges.rs` (reading "Open Default"'s disabled state) for what the pages
  show (read only).

## Owns and may change

- **`fake_backends/walk.rs`:** `Walk::run` keeps its signature and behaviour; a sibling takes
  the live check that `boxed::run_ctl` already accepts (today `|_binary| ()`):

  ```rust
  /// `run`, then `check` while the box still serves, given the configuration
  /// directory and the client root that ctl signed in under.
  fn run_then(&self, acts: Vec<Act>, check: impl FnOnce(&Path, &Path)) -> Seen
  ```

  `run` becomes `self.run_then(acts, |_, _| ())`. Declare `mod open_default;` beside
  `mod here_key;`.
- **`fake_backends/walk/open_default.rs`** (new), module doc citing
  `temper/docs/design/domain/testing.md, 2` and `temper/docs/design/client/README.md, 2`:

  ```rust
  /// The browser's client bounds over the native transport (client/README.md, 2).
  fn browser_bounds() -> temper_client_shell_terminal::limits::Limits

  /// Session 1, the press; then session 2, the reload. Panics naming the page
  /// and the missing node when a session does not reach its end within WITHIN.
  pub(super) fn open_and_reload(config: &Path, client: &Path)
  ```

  Each session is a `real::World` with one `NativeClient` and a referee shaped as
  `support/playground.rs`'s `Judge` (a page enum, a deadline from `support::WITHIN`, the tree's
  names in its failure message), spawned as `reload` spawns it: the engine's address from the
  configuration's `/application/clients/listener/socket`, the client root opened with
  `skein_shell::open_root`, the directory ctl's walk signed in under (find it from
  `support/ctl.rs`'s `Call`), `Start::Token`, a seed of your choice. A press is
  `client.act(node, skein_ui::Act::Press)` on the node found by role and name; the client is
  closed at each session's end and the world drained (`Host::is_empty`). The finds for Default
  as the chosen project, the forge's row and the key's account are yours: one each that only a
  delivered watch draws, read as `tests/client`'s suites read those pages.
- **`fake_backends/walk/here_key.rs`:** the story's `walk.run(acts)` becomes
  `walk.run_then(acts, open_default::open_and_reload)` (or a closure calling it); its acts and
  assertions stay as they are.
- **Status, after the code:** in `temper/docs/design/domain/testing.status.md`, remove the
  **Not yet** item that begins `2: the owned welcome walk's Open Default` through `here_key.rs`).`,
  so the bullet begins `- **Not yet:** 2.4: GitHub's real-loop stories`; re-wrap that first
  line only. This departs from the implementer's text ("Not yours: any `docs/design/`"): this
  one status edit is the task's.

Pre-approved:
- If the check closure of `run_ctl` turns out to run after the box stops, move the two
  sessions to a `ctl::Step` the walk runs after its last act, or to the nearest point where the
  box still serves, and say which.
- If the Ready page offers no "Open Default" once ctl's `forge setup` has adopted `hello`, or
  Open Default is refused (a project limit, a name taken), stop: that is a question, with what
  the page showed.
- If the story's CPU passes 60 s with the sessions in, measure each session alone and report;
  do not move or split the story.

## Steps

1. `run_then` and the empty check; the story unchanged passes:
   `tools/tiers/run temper process -- -E 'test(=fake_backends::walk::here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box)'`
   (check the count is 1; this builds `temper` and `fake-claude` and caches Forgejo as the tier
   does). Note its CPU with `tools/tiers/run --cpu <new dir> temper process -- -E '...'`.
2. `open_default.rs` and the call in `here_key.rs`. With `rows: 3` restored (uncommitted): the
   story fails as the acceptance says; keep its message. Revert to the trunk's file (`git
   checkout` that one file) and run the story again: it passes; then its CPU figures.
3. `cargo fmt`; `cd temper && python3 -I -B scripts/old_forms.py`; `heavy cargo clippy -p
   temper --all-targets -- -D warnings`.
4. The status edit; the citations check (`notes/common.md`); commit (the sessions and the
   walk's change; the status) and push.

## Report

As the implementer's text says, and: the failing message with `rows: 3`, the story's CPU before
and after (and each session's alone, if the CPU option applied), the finds you chose, and where the
sessions run in the box's life.
