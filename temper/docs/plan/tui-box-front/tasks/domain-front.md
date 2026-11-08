+++
id = "domain-front"
part = "tui-front"
kind = "feature"
needs = ["forges-linked"]
files = [
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/forges.rs",
  "temper/crates/temper-client-domain/src/tests/playground.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The client root reads `linked`, says whether the person signed in by the printed link, and reads the playground again

## Goal

The client domain's facts for the TUI's welcome and playground in front of a box (`temper/docs/design/client/README.md`, 6.17, *The welcome* and *The playground*; 4.8, *Read once a link is live*). After `forges-linked`, the forges document carries `linked`; the root keeps it, `welcome::signed_in` reads it with the welcome's decision and `me`, and the forge's side reads the playground again when the `forges` watch gains or loses the playground's fake row, so that a TUI sees a playground a browser started or reset.

Acceptance, by `notes/contracts.md`, section 3, *The client*:

1. **`Side::linked(&self) -> Option<bool>`** (`forge.rs`), reached as `root.forge().linked()`: `None` until the `forges` watch delivered a document; then that document's `linked`; `None` again when the rows are dropped (`forges::configure` on a signed-out or out-of-date link setting `State::Unknown`) or the watch ends (`State::Ended`). The decode: `family.rs` gains `pub fn forges_document(bytes, limits) -> Option<(Box<[Row]>, bool)>`, the rows and `linked`; `family::forges` keeps its signature and returns its rows (its readers in tests and `temper-client-tools` unchanged); `forges::told`'s `Told::Changed` arm (about line 900) decodes through it and keeps both.
2. **`welcome::signed_in(root: &Domain) -> bool`** (`pages/welcome.rs`), with the contract's doc comment: `root.forge().linked() == Some(true)`, or `decision(root)` is `Some(FirstRun::Unfinished { .. } | FirstRun::Whole)`, or `me` (`accounts::current_me(root.client())`) names a project. Nothing else reads it here; `tui-welcome` does.
3. **The playground read again** (4.8): in `forges::told`'s `Told::Changed` arm, when the rows before were `State::Rows` and the rows after are `State::Rows`, and one holds a row of `Mode::Fake` and the other none, the side reads the playground again (`playground::read(side, out)`, as a start's answer does). From `Unknown` or `Ended`, or with the fake row on both sides, nothing more is read: the read at the link's first life covers a TUI's start.
4. **Tests,** a failing test first for each (`notes/common.md`, *Tests stay cheap*):
   - `tests/forges.rs`: one new test, `the_forges_document_says_whether_the_printed_link_signed_in`: `linked()` is `None` before the watch delivered; `Some(false)` and `Some(true)` after documents saying so (`w::ForgesParts { forges, linked }`); `None` after the link signs out; and `family::forges_document` returns the same rows as `family::forges` for one document.
   - `tests/playground.rs`: one new table test, `a_fake_row_gained_or_lost_reads_the_playground_again`, rows: first delivery with a fake row (no read), no fake then fake (a read), fake then none (a read), fake then fake (no read), a real row added beside no fake (no read); each counts the playground reads pushed (the `Request::Client { asked: Event::Read { .. } }` whose record reads `w::Read::Playground`, as `read()` there checks).
   - `tests/pages/welcome.rs`: one new table test, `signed_in_reads_linked_the_decision_and_me`, rows: nothing in (false); `linked` true alone (true); `linked` false with an empty store (false); `linked` false with a real forge kept and none connected, the decision unfinished (true); `linked` false with `me` naming a project (true); `linked` false after a restart with a model usable and no forge (unfinished at Forge: true). Build the roots with the file's `opened`, `answer`, `deliver` helpers and a forges document with `linked`.
5. **Status:** delete the **Not yet** bullet naming `domain-front` in `temper/docs/design/client/README.status.md`. No other design line.
6. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-domain`; then, as readers of a change of meaning (the forges watch now reads the playground again), `cargo nextest run -p temper-client-view` and `cargo nextest run -p temper-client-world --test suite -E 'test(/playground|welcome/)'`: a red there that this change made is yours to fix in this task's files, or, if the fix is outside them, report it under `## Left` with its test and log line.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, section 3.
- `temper/docs/design/client/README.md`, 6.17 (*The welcome*, `welcome::signed_in`; *The playground*, *Read again*), 4.6 (`linked`), 4.8 (*Read once a link is live*), 3.5 (*The welcome*, *The store's state*).
- The code: `temper-client-domain/src/forge.rs` (`Side` 67-110, `new` 111-150, accessors 151-260); `forge/forges.rs` (`State` 232, `rows` 223, `configure` 730-765, `told` 880-990, its `Changed` arm about 896-910); `forge/family.rs:274` (`forges`); `forge/playground.rs` (`read` about 197, `told`); `pages/welcome.rs` (`decision` 124-159, `forge_kept` 98, `accounts::current_me`); tests: `tests/forges.rs` (`limits`, `fixture` builders about 292 and 355), `tests/playground.rs` (`read` 42, `the_playground_is_read_once_per_live_link_and_replaced_pages_are_dropped` 58), `tests/pages/welcome.rs` (`opened`, `answer`, `deliver`, `the_welcome_waits_for_its_facts` 316, `step_marks_tick_facts_and_keep_the_address_current` 995).
- `forges-linked`'s field: `temper-client-protocol-wire`'s `w::ForgesParts { forges, linked }` and `Forges::linked()` (generated).

## Owns and may change

Owns the files listed: `Side`'s `linked`, the decode, `signed_in`, the read again, the three tests and the status bullet. Not `temper-client-tui` (`tui-surface`, `tui-welcome`, `tui-front-views` are in it), not `temper-client-view`, not the wire.

Options, each decided:

- **Where `linked` lives:** a field `linked: Option<bool>` of `Side`, set beside `side.forges` in the `Changed` arm; if keeping it inside `forges::State::Rows` reads better, do that and keep `Side::linked`'s signature.
- **`pages.rs`' `copy_rows`** reads `forges::rows` for Ready and Home; leave it as it is.
- **A playground read already pending** when the fake row changes: read again anyway (the side drops the answer of a read it replaced, 4.8).

## Steps

1. The failing tests of acceptance 4, seen failing; then `family.rs`, `forge.rs`, `forges.rs`, `welcome.rs`; the narrowest test after each, e.g. `heavy cargo nextest run -p temper-client-domain -E 'test(/a_fake_row_gained_or_lost/)'`.
2. The status edit; the runs of acceptance 6; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: a TUI can tell the browser used the printed link and sees the playground a browser started), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each test added and the rule broken by hand for it (for instance the read again dropped, or `signed_in` reading `linked` alone), where `linked` is kept, and any reader's red under `## Left`.
