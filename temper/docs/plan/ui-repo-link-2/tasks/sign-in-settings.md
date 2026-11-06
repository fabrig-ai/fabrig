+++
id = "sign-in-settings"
part = "settings"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/tests/suite/forge.rs",
  "temper/crates/temper-forge-setup/src/ini.rs",
  "temper/crates/temper-forge-setup/src/machine.rs",
  "temper/crates/temper-forge-setup/src/prepare.rs",
  "temper/crates/temper-forge-setup/src/problem.rs",
  "temper/crates/temper-forge-setup/src/sign_in.rs",
  "temper/crates/temper-forge-setup/src/spec.rs",
  "temper/crates/temper-forge-setup/src/tests/**",
  "temper/crates/temper-forge-setup-shell/src/prepare.rs",
  "temper/crates/temper-client-view/src/forge/repository.rs",
  "temper/crates/temper-client-view/src/tests/repository_link.rs",
  "temper/docs/help/box.md",
]
effort = "high"
minutes = 100
+++

# A box's Forgejo sign-in settings: a moved port, the refused pair, the session, an empty name

## Goal

Four repairs a code review found in `ui-repo-link`. A moved temper port reaches Forgejo's source. `machine.owned.public` beside `machine.owned.sign_in` is refused. An owned Forgejo's session lasts an hour. An empty OWNER or NAME has no link.

Acceptance:

1. **A moved port.** A later `temper forge prepare` on a document with `machine.owned.sign_in` set, once `state` is written and as its last act, lists the sources; with `temper` listed, it runs `update-oauth` with that source's id and temper's current discovery address (`sign-in.md`, 9.9, *A moved port*). A first preparation, a document without `machine.owned.sign_in`, and a list with no `temper` run no update. Each of the two commands is under `checked.limits.deadline`.
2. **Still never fails the set-up.** Any ending of either command, a non-zero exit, a signal, a deadline, a spawn failure, leaves the preparation going: it ends `Ok`, with one line on its stderr, holding no secret. A termination signal to the preparation itself still stops it, as `setup.md`, 6.3 says.
3. **The flags, against the binary.** On the builder's Forgejo (`skein_throwaway_forgejo`'s, as `temper/crates/temper/tests/support/forgejo_sign_in.rs` finds it), by a throwaway run: `add-oauth`, then `update-oauth` as the spec builds it, then read the source back. The report says the flags it found, that `--key` and the rest stayed as they were, and whether `update-oauth` fetches the address it's given and what it leaves when that address doesn't answer.
4. **The pair refused.** `machine.owned.public` beside a set `machine.owned.sign_in` refuses the start, whichever way the document is read, with exactly this line:
   `application.machine.owned.sign_in: cannot be set with application.machine.owned.public; a box's sign-in through temper is not proved behind a proxy`
   The member's `bound` says so, and `temper/docs/help/reference/configuration.md` is regenerated.
5. **The session.** Every owned `app.ini` holds `[session]\nSESSION_LIFE_TIME = 3600\n`, directly after `[mailer]\nENABLED = false\n`. Check the key against the same binary, and say how in the report.
6. **An empty name.** `web_address` answers `None` when `owner` or `name` is empty, so neither a signed-in nor a plain address is built and the words stay plain.
7. **The sentence.** `temper/docs/help/box.md`, after "Another box: empty directories; keep the first running.", gains this one, verbatim: `A box trusts every program listening on this machine's loopback, since browser cookies ignore ports.` The page stays within its bound.
8. The three **Not yet** entries this plan wrote for it are gone: `sign-in.status.md`'s that begins `9.2, 9.9: a box's later preparation`, `setup.status.md`'s that begins `6.2: `[session]``, and `client/forge.status.md`'s that begins `5.4, *A repository's link*: an empty OWNER`. Every other entry stays, the process story's on 9.9 among them.

## Inputs

- `temper/docs/design/protocol/sign-in.md`, 9.2, 9.9.
- `temper/docs/design/protocol/setup.md`, 6.2, 6.3, 8's `owned.*` table, 13.1.
- `temper/docs/design/client/forge.md`, 5.4, *A repository's link*.
- `temper/docs/plan/ui-repo-link-2/notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- Crosses to jig for one variant (this departs from a branch staying in its tree; the plan holds two tasks): `jig/crates/jig-protocol-config/src/problem.rs` gains `What::Because(&'static [u8])`, a refusal whose words its reader gives. Its readers take one arm each, the words as they are: `temper-forge-setup/src/problem.rs`, `temper-service/src/startup/problems.rs` (both its tables), `temper-fake/src/read.rs`, and whatever else the compiler names.
- Readers and goldens: `temper-forge-setup/src/tests/golden.rs`' two `APP_INI_*` constants gain the two lines. `temper/crates/temper/tests/suite/forge.rs`' strings ending in `[mailer]` stay true, which is why the section goes after it.
- `temper-service/src/tests/forge_sign_in.rs`, `box_entries`: its settings hold both `public` and `sign_in`, a pair no document can now give. Default: leave it, for the service takes settings already checked; say so in the report. If a test there fails for it, give the fixture the record's own origin.
- **A change of meaning runs its readers' tests:** `app.ini`'s bytes change for every owned Forgejo, and a later preparation runs two commands more. Run `temper-forge-setup`, `temper-forge-setup-shell`, `temper-service`, `temper-client-view`, `temper-docs`, `temper-client-tools` and `temper`'s focused tests. A fake `forgejo` script that answers `admin auth list` or `update-oauth` badly must not change any story's ending: that's acceptance 2.
- Options, each pre-approved:
  - **If `update-oauth` names its id or its address otherwise,** build what the binary takes, and correct `sign-in.md`, 9.9's flags in the same words. Say so in the report.
  - **If `update-oauth` rewrites a member it wasn't given,** pass that member too, the same value `source_add` gives; never a secret. Say so.
  - **If the binary reads no `SESSION_LIFE_TIME` under `[session]`,** use what it reads for the session's lifetime, and correct `setup.md`, 6.2. Say so.
  - **If `update-oauth`, given an address that doesn't answer, changes nothing and exits non-zero,** that's acceptance 2 and needs nothing more. If it stores the address anyway, the same.
  - The fixture for a suite story lacks a provider: build `machine.owned.sign_in` and `engine.clients.provider` as `temper init --box` writes them (`temper/crates/temper/tests/support/forgejo_box_sign_in.rs` reads such a document).

## Steps

Failing tests first. Sketches:

```rust
// temper-forge-setup/src/sign_in.rs
/// The first column of the row whose second names `temper`; `source_present` becomes its `is_some`.
#[must_use]
pub fn source_id(list: &[u8]) -> Option<&[u8]>;

// temper-forge-setup/src/spec.rs
/// Points the box's source at temper's current discovery address (sign-in.md, 9.9).
#[must_use]
pub fn source_update(owned: &Owned, place: &Place, id: &[u8], discovery: &[u8]) -> Spec;

// temper-forge-setup/src/prepare.rs
impl Prepare {
    /// A box's later preparation points its source here, last (setup.md, 6.3).
    #[must_use]
    pub fn with_source_address(self, discovery: Option<Box<[u8]>>, deadline: Duration) -> Self;
}
pub enum Request {
    // ...
    /// One of the box's two source commands; no ending of it fails the preparation.
    Source { spec: Spec, deadline: Duration },
    /// One whole line for the preparation's stderr.
    Note { line: Box<[u8]> },
}
```

- The machine's later path ends at `prepare.rs`, about 350 (`self.end(Ok(()))`): that one goes through the source's phases. A first preparation ends at `Phase::Finish`, unchanged. `Source` is answered `Event::Ran`, `Note` `Event::Done`. The line is `forge source: discovery address not updated: ` and the masked `tail`, on one line.
- The host, `temper-forge-setup-shell/src/prepare.rs`: `Request::Source` runs `command::run_until` with the clock's now plus `deadline`. `Error::Interrupted` still returns `Err(Failed::Command(..))`; any other `Error` answers `Event::Ran` with a failed exit and the error's words in `stderr`, so the machine holds the one decision.
- `temper/crates/temper/src/forge_sign_in.rs`: `pub(crate) fn discovery(bytes: &[u8], named: &Named) -> Result<Vec<u8>, String>`, the origin and the path, taken from the two copies in `run` and `source`, which both call it.
- `temper/crates/temper/src/forge.rs`, about 239, `Prepare::new(..)`: `.with_source_address(owned_sign_in.then(|| discovery(..).ok()).flatten(), checked.limits.deadline)`. A discovery that can't be made is no update, as acceptance 2.
- `temper-forge-setup/src/machine.rs`, in the `if sign_in.is_some()` block, about 502: `public.is_none()` or the `What::Because` push at `p.member(b"sign_in")`. `MEMBERS`' `sign_in` `bound` gains `; not with public`.
- `temper-client-view/src/forge/repository.rs`, `web_address`, its first lines: both forms pass through it (`forge_sign_in.rs`, `open_repository`).

Test sketches:

- `temper-forge-setup/src/tests/sign_in.rs`: `source_id` on the Forgejo table `auth_list_finds_temper_only_in_the_sources_name_column` holds, with the same refusals; `tests/spec.rs`: `source_update`'s arguments, `--work-path` and `--config`, no secret.
- `tests/prepare.rs`, in the `later` world: `a_later_preparation_points_a_listed_source_at_the_current_address`; `a_source_not_listed_is_left_to_its_step`; `no_ending_of_the_source_commands_fails_the_preparation`, a table of `Exit::Code(1)` and `Exit::Signal(9)` for each command; `a_first_preparation_runs_no_source_command`; `without_an_address_a_preparation_runs_no_source_command`.
- `tests/machine.rs`: `sign_in_refuses_machine_owned_public_at_its_path`, the whole line. `tests/golden.rs`: `every_owned_app_ini_holds_the_session_lifetime`, for each `FirstSignIn` and none.
- `temper/crates/temper/tests/suite/forge.rs`, on a `forgejo` script that logs its arguments, as the file's others do: `a_later_prepare_gives_a_listed_source_the_current_address`, run twice with the clients' port edited between, naming each address; `a_refused_update_leaves_prepare_succeeding_with_one_line`.
- `temper-client-view/src/tests/repository_link.rs`: `an_empty_owner_or_name_has_no_address`, for both forms.
- Each new test bites: break the rule once, see it fail, restore, and name it in the report.

Then the implementer's text's closing steps. Commit and push.

## Report

The implementer's text's report, and: the flags and fields acceptance 3 found, how acceptance 5 was checked, and the `box_entries` choice.
