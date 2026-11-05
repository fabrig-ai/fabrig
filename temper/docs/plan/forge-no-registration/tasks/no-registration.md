+++
id = "no-registration"
part = "no-registration"
kind = "feature"
needs = []
files = ["temper/crates/temper-forge-setup/src/ini.rs", "temper/crates/temper-forge-setup/src/sign_in.rs", "temper/crates/temper-forge-setup/src/prepare.rs", "temper/crates/temper-forge-setup/src/tests/golden.rs", "temper/crates/temper-forge-setup/src/tests/sign_in.rs", "temper/crates/temper/src/forge.rs", "temper/crates/temper/src/forge_sign_in.rs", "temper/crates/temper/tests/suite/forge.rs", "temper/crates/temper/tests/support/forgejo_sign_in.rs", "temper/crates/temper/tests/support/owned.rs"]
effort = "high"
minutes = 90
+++

# temper: an owned Forgejo that registers no one through temper

## Goal

`TEMPER_FORGE_NO_REGISTRATION`, read where `temper forge prepare` and `temper forge sign-in` run, closes account-making on a forge `application.sign_in.forges` names. Unset, every byte stays as it is. Then show, at the pinned Forgejo 17.0.1, what those settings do: nothing here has been run, and the process tier is the check for what Forgejo does.

Acceptance:

- Unset or empty, `ini::text` and the printed settings are today's bytes: `sign_in_enables_only_external_registration_with_the_member_login`'s bytes and every other golden stand, only its argument's spelling changing.
- Set to anything non-empty, a forge named by `sign_in.forges` gets exactly this after `[service]\nDISABLE_REGISTRATION = true\n` and before `[mailer]`:

  ```
  [oauth2_client]
  ENABLE_AUTO_REGISTRATION = false
  USERNAME = preferred_username
  ACCOUNT_LINKING = login
  ```

  A forge not named gets today's text, the variable or not.
- A later `temper forge prepare` with the variable set writes the same `app.ini`, byte for byte.
- `temper forge sign-in` with the variable set prints `LINKS_ONLY_SETTINGS` in place of `SETTINGS`.
- The story `an_owned_forgejo_signs_its_member_in_through_temper` passes, grown as in step 4, on a real Forgejo 17.0.1. Its proof is your run: **the queue's end checks do not run the process tier**, so your report holds the command, the count and the story's printed figures.
- No test reaches a real host. The story's `Browser` already asserts loopback.

**If what the story shows at 17.0.1 is not (1), (2) and (3) of step 4, end with `Question:`**, naming what Forgejo did: the status, the address it ended on, which of the link page's forms it served, and the log's lines. This is the request's own ending, not a failure of yours. Don't make the story fit: push the code, the goldens and the leg as far as it went, red, and say so.

## Inputs

- `temper/docs/plan/forge-no-registration/inputs/request.md`
- `temper/docs/design/draft/dogfood-hacks.md`, 2.8: what this builds. Written and final; its status is written too.
- `temper/docs/design/protocol/setup.md`, 6.2 and 6.8; `temper/docs/design/protocol/sign-in.md`, 9.4 and 9.8: what stays as it is, and the story's start. **These two stay as they are.**
- `temper/docs/design/domain/testing.md`, 2.6: a variant grows an existing story, never a copy of its setup.
- `temper/crates/temper/tests/support/forgejo_sign_in.rs`, whole: the story you grow.
- `skein/testing/skein-throwaway-forgejo/src/instance.rs`: `exchange`, `Outgoing`, `admin`, `stop` (an attached instance keeps its scratch).

## Owns and may change

- **Owns:** the files in `files`.
- **Readers, call sites only:** `ini::text` is called by `prepare.rs` and six times by `tests/golden.rs`; `sign_in::instructions` by `forge_sign_in.rs`, `tests/suite/forge.rs` (two) and `tests/sign_in.rs` (two); `Prepare::with_sign_in` by `forge.rs` alone. `temper-forge-setup-shell`'s tests build `Prepare::new` with no `with_sign_in` and need nothing. No other crate names these.
- **Markdown:** no test-read Markdown names these settings (`rg ENABLE_AUTO_REGISTRATION temper/docs/help` finds nothing), so `docs/help/` stays closed. The variable is a hack's switch: it goes in no help page and no `Settings` table, as `TEMPER_R8_WAIT_CHECK` doesn't.
- **Options:**
  - If `TEMPER_FORGEJO` or `TEMPER_FORGEJO_RUNNER` is unset in your login shell, or `--version` names anything but `forgejo version 17.0.1` and `forgejo-runner version v13.2.1`: preparation holds both to the pin (`names_forgejo`) and refuses another, so the story can't run. On the machine this plan was cut on, they name 17.0.0 and 13.2.0; builders weren't checked. Push the rest and end with `Question:` naming what they say. Fetch and install nothing yourself, and no `temper forge fetch`: it reaches a real host.
  - If Forgejo's forms need a CSRF field, take it from the page that served the form; if they don't, send none. Either way, say so in the report.
  - If the token's `write:admin` can't delete or make `sam` through the API, use Forgejo's command line (`admin user delete`, `admin user create`) with the story's work path and configuration, as `owned_command` does.
  - If the leg's cookie handling needs more than dropping Forgejo's cookies, use a second `Browser` signed in by a fresh printed link (`connect` shows how a spent link is replaced), and say so.
  - If the story goes past `process-temper`'s `cpu_test_s` (60 s, `tools/tiers/budgets.toml`): it is red. Ask `AGENTS.md`'s three questions, in order, then report the measurements as a `Question:`.

## Steps

### 1. The settings

`temper-forge-setup` is a step crate: it reads no environment. In `sign_in.rs`:

```rust
/// What a person's first sign-in through temper does on their forge
/// (protocol/sign-in.md, 9.4; draft/dogfood-hacks.md, 2.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstSignIn {
    /// Makes their account.
    MakesAccount,
    /// Makes none: an account already there links once, by its password.
    LinksOnly,
}
impl FirstSignIn {
    /// The settings `instructions` prints.
    #[must_use]
    pub const fn settings(self) -> &'static [u8] { /* every variant listed */ }
}
pub const SETTINGS: &[u8] = /* unchanged */;
pub const LINKS_ONLY_SETTINGS: &[u8] = b"[oauth2_client]\nENABLE_AUTO_REGISTRATION = false\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[service]\nDISABLE_REGISTRATION = true\n";
```

`instructions` gains a last parameter, `first: FirstSignIn`, and prints `first.settings()`.

`ini::text`'s last parameter becomes `sign_in: Option<FirstSignIn>`: `None` writes nothing, as `false` did; `Some(MakesAccount)` writes today's bytes; `Some(LinksOnly)` writes the block above, with no `ALLOW_ONLY_EXTERNAL_REGISTRATION`.

In `prepare.rs`, `enum Registration` goes: the field is `sign_in: Option<FirstSignIn>`, `with_sign_in(self, sign_in: Option<FirstSignIn>)`, and `ini()` passes it on.

### 2. The variable

In `crates/temper/src/forge_sign_in.rs`, read by both commands, so that the printed settings follow the same variable:

```rust
/// A dogfood hack's switch (temper/docs/design/draft/dogfood-hacks.md, 2.8).
pub(crate) fn first_sign_in() -> sign_in::FirstSignIn {
    // non-empty TEMPER_FORGE_NO_REGISTRATION: LinksOnly; otherwise MakesAccount
}
```

`forge.rs`'s preparation: `.with_sign_in(<the named test as it stands>.then(super::forge_sign_in::first_sign_in))`. `forge_sign_in::run` passes `first_sign_in()` to `instructions`.

### 3. The goldens and the wiring

- `tests/golden.rs`: a new test beside `sign_in_enables_only_external_registration_with_the_member_login`, built the same way from `APP_INI_WITH_RUNNER`, its registration bytes written by hand.
- `tests/sign_in.rs`: `LinksOnly` prints `LINKS_ONLY_SETTINGS` and no `ALLOW_ONLY_EXTERNAL_REGISTRATION`; the existing `SETTINGS` assert stands with `MakesAccount`.
- `support/owned.rs`: `pub fn prepare_no_registration(directory, forge) -> Output` beside `prepare`, sharing its command, with the variable set.
- `tests/suite/forge.rs`, one focused test on `support::owned::scripted` and `forge_sign_in::configure`, as `forge_sign_in_prints_the_owned_command_without_its_private_secret` starts:
  1. prepare with the variable unset; read `app.ini`: today's bytes;
  2. prepare with it set: the block above follows `[service]`, and no `ALLOW_ONLY_EXTERNAL_REGISTRATION`;
  3. prepare with it set again: the whole file is the same (the request's "a later prepare");
  4. prepare with it empty: the same whole bytes as (1);
  5. `forge_sign_in::command(directory).env("TEMPER_FORGE_NO_REGISTRATION", "1")` prints `LINKS_ONLY_SETTINGS`.

  Compare with `assert!(a == b)`, not `assert_eq!`: `app.ini` holds the scratch's secrets, and they stay out of logs.

### 4. The story

Grow `run_owned` in `support/forgejo_sign_in.rs`, with no new `#[test]` and no copy of its setup (`temper/docs/design/domain/testing.md`, 2.6). Between `member_named(&instance)` and `connect(..)`, one leg, a function of its own citing `temper/docs/design/draft/dogfood-hacks.md, 2.8`. `serve` stays up throughout; no forge is connected yet.

`Browser` and `sign_in` change shape only as the leg needs:

- split `sign_in` at `opened`: the printed link's half, and `through_temper(browser, temper, forge) -> io::Result<Landed>`, which opens `/user/oauth2/temper`, authorizes, follows the callback and returns the status, address and body it ended on. The first sign-in's asserts stand;
- `Browser::form(url, fields)`, posting `application/x-www-form-urlencoded` to Forgejo;
- dropping Forgejo's cookies and keeping temper's (`jig-`), so each sign-in below starts signed out of Forgejo and signed in to temper, spending no link.

The leg:

```
stop Forgejo; prepare_no_registration; run and attach it again

(2) sam's account, made above by the first sign-in, keeps signing in:
    through_temper ends signed in, not on /user/link_account;
    a read only the signed-in account gets names sam

delete sam through the admin API; confirm 404

(1) through_temper ends on /user/link_account:
    /api/v1/users/sam is still 404

(3, no way to register) that page serves a form for /user/link_account_signin and none
    for /user/link_account_signup; POST /user/link_account_signup, with fields a
    registration would need, answers 403; POST /user/sign_up answers 403;
    /api/v1/users/sam is still 404

make sam through the admin API: login sam, sam@example.com, a throwaway password,
must_change_password false

(3, offered linking) through_temper ends on /user/link_account, with the password form:
    the wrong password links nothing: sign-in again still ends on the link page;
    the right one links: the read names sam

(3, once; 2 again) through_temper ends signed in, with no link page, and the read names sam
```

Then `connect` runs as it does, on the restarted Forgejo. Add the leg's time to the story's `eprintln!`.

`app.ini` after `owned::run` carries `Running::start`'s appended log section, so compare no `app.ini` bytes in this leg: that is step 3's.

### 5. Finishing

- **Bites:** the golden with one byte changed; the focused test with `first_sign_in` always `MakesAccount`; the story with the leg's `prepare_no_registration` swapped for `prepare`, which must go red at (1), by auto-registering. The report names each.
- `cargo fmt`; `(cd temper && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p temper-forge-setup -p temper --all-targets -- -D warnings`.
- `heavy cargo nextest run -p temper-forge-setup`, and `-p temper --test suite -E 'test(/forge/)'`.
- `heavy cargo check -p temper-forge-setup-shell -p temper-box -p temper-service -p temper-fake -p temper-playground --tests`.
- `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=an_owned_forgejo_signs_its_member_in_through_temper)'`. Before you grow it, run it once for its figures. If the grown story's `whole` passes 40 s, measure its CPU: `tools/tiers/run --cpu <new dir> temper process -- -E 'test(=an_owned_forgejo_signs_its_member_in_through_temper)'`.
- `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

`Done:` or `Question:` as above. For each of (1), (2) and (3): what Forgejo answered, in statuses and addresses. Whether Forgejo's forms needed a CSRF field. The story's figures before and after, and the machine. The bites. Which options you took. Anything the entry in `temper/docs/design/draft/dogfood-hacks.md`, 2.8 says that the run showed otherwise: it isn't yours to edit, so the report says it.

## Answers

### 2026-11-05

**Question:** Forgejo 17.0.1 still serves a form for `/user/link_account_signup` on its link page; investigate a stronger configuration, or change the required no-signup-form behaviour?

**Answer:** Neither yet. What the expectation becomes is the user's, and they can't weigh it on what run 1 saw: the story stopped at a string match on the form's `action`, so whether that form can register anyone is unobserved. This run gets those facts, changes no expectation, and ends with `Question:`, as the spec's own ending says, unless the form is gone.

Start from the pushed branch `forge-no-registration/no-registration`. The links-only block, the variable and every other acceptance line stand. Add no setting to the block.

1. **The assertion stays, and moves last.** In `support/forgejo_sign_in.rs`, `link_page` keeps its other asserts and prints what it found, and drops `assert!(!signup, "link page offers no registration")`. The form's presence is kept from each link page, and the same assert, with the same message, is the last statement of `no_registration` before `Ok(instance)`. The story stays red while the form is served; the rest of the leg now runs ahead of it.
2. **What the form holds.** `link_page` prints, for the form whose `action` is `/user/link_account_signup`: the `name` of each `input`, `button`, `select` and `textarea` in it, and its text with the tags stripped. Print no `value`: `_csrf` and the like stay out of logs.
3. **Registration's POSTs and the linking.** With step 1 done, the leg's existing asserts decide: `POST /user/link_account_signup` and `POST /user/sign_up` answer 403, `sam` stays 404, the admin makes `sam`, the wrong password links nothing, the right one links, and linking happens once. Change none of them. If one fails, that is the finding: stop there and report it, with the status, the address and the log's lines. Whichever way it goes, say whether each POST needed a CSRF field, or that it is unobserved.
4. **What gates the form, by reading and not by trying.** Run `"$TEMPER_FORGEJO" embedded view templates/user/auth/link_account.tmpl`, and the same for `templates/user/auth/signup_inner.tmpl`; `embedded list` finds them if those names differ. This reaches no host. Report the conditions around the signup tab and form, and which settings they read, verbatim. If the binary has no such subcommand, say so and leave it `unknown`. Try no other `app.ini` setting: a stronger configuration is the user's call once this is known.

End with `Question:` for the user unless the form is gone, naming the test `an_owned_forgejo_signs_its_member_in_through_temper`, the assertion `link page offers no registration`, and the request's line "with no way to register" (`temper/docs/plan/forge-no-registration/inputs/request.md`, Tests, 3). Give what 2, 3 and 4 showed, then these options, with your recommendation:

- **Accept:** the assertion becomes what the request's line asks: the form has no field a registration needs, and both POSTs answer 403 with no account made. `draft/dogfood-hacks.md`, 2.8's "closes the form" is reworded to match.
- **Stronger:** a setting or template that the reading in 4 shows removes the form, with its cost.

This is the expected ending of this run, not a failure of it: the foreman takes the `Question:` as it stands. Leave `temper/docs/design/draft/dogfood-hacks.md` as it is, and report what the run showed against it.

2026-11-05 (fabrig/temper, the plan's requester). **Accept.** "No way to register" means no field a registration needs, both POSTs answering 403 and no account made, which the run proved at Forgejo 17.0.1. Assert exactly that, and reword `dogfood-hacks.md`'s "closes the form" to match. No custom template: maintaining one costs more than an empty wrapper is worth.
