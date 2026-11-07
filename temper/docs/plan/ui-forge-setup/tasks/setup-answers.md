+++
id = "setup-answers"
part = "answers"
kind = "feature"
needs = ["made-record"]
files = [
  "skein/crates/skein-forge/src/github/document.rs",
  "skein/crates/skein-forge/src/github/decode/**",
  "skein/crates/skein-forge/src/github/server/app.rs",
  "skein/crates/skein-forge/src/github/tests/**",
  "skein/crates/skein-fake-forge-github/src/**",
  "skein/crates/skein-forge-setup/src/**",
  "skein/testing/skein-forge-kit/src/**",
  "skein/tests/forge/github/src/**",
  "skein/tests/forge/setup/**",
  "skein/docs/design/forge-github.md",
  "skein/docs/design/forge-github.status.md",
  "skein/docs/design/forge-setup.status.md",
  "skein/docs/design/fake-forge.status.md",
]
effort = "high"
minutes = 150
+++

# `Reconcile` answers the person's id, the App's plans its account, and `Convert` marks the App it made

## Goal

Three things in skein's forge setup. A Forgejo `Reconcile` given a person's login reads that user and answers its id in `Done`. `GET /app`'s `owner` is decoded, and `Convert`, `Reconcile` and `Paste` answer it. `Convert` writes a new item, `AppMade`, first. temper's callers keep compiling, keep passing no login and ignore the new answers: its link, record and card are the next plan.

This task departs from the implementer's text in two ways. Section 13 of `skein/docs/design/forge-github.md` gains one fact here, because `github_conformance_names_every_fact_of_forge_github_13` reads it and must go green with the case. And the status entries of acceptance 8 are yours to remove, because the design already says what this task builds.

Acceptance:

1. **The owner decodes.** `App::owner` follows the contract's table, every row a row of the document table in `skein/crates/skein-forge/src/github/tests/app_documents.rs`. The server half writes it, and a round trip through `server::app` and the decoder holds it.
2. **The fake answers it.** `GET /app` on the fake names the App's account from the scenario's accounts; an owner no account holds answers none (`skein/docs/design/fake-forge.md`, 10.1).
3. **The conformance adds it.** `app-owner-id-login-type` is in `cases::NAMES` and in `forge-github.md`, section 13. It passes on the fake, and `github_conformance_names_every_fact_of_forge_github_13` passes. The live side is compiled, not run.
4. **The person.** The table test sketched below passes: an id, none for a `404`, and a stop naming the login at `Stage::Person` for an id of 0, another status and a `Down`. A `Reconcile` with no login still sends nothing more and ends `Done(Nothing)`, which every existing run holds. `Place::Person` refuses a malformed login, a row of the `tests/given.rs` table.
5. **The account in each answer.** `Convert` ends `Made { owner }`; GitHub's `Reconcile` and `Paste` end `Lacking { lacking, owner }`; a read that fails stops the run as it does today, with no answer.
6. **The mark.** `Convert`'s writes are `AppMade`, `AppId`, `AppSlug`, `AppKey`, `User`, in that order, each answered before the next; `AppMade`'s bytes are the App's id. `Paste` writes none. `item::app_made` and `item::fits` take it, a row of the `tests/item.rs` table.
7. **The stories.** `skein-forge-setup-world` holds a person given and one gone on the fake Forgejo (sketched below), and its Convert and Reconcile stories assert the account answered, through the real HTTP and codecs.
8. **temper still works.** It passes no login. Its focused suites below pass, and a Convert keeps `AppMade`: the `Item::AppMade` row is there after a Convert in `temper-service`'s `written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep`. These status entries are gone: `skein/docs/design/forge-setup.status.md`, **Not yet**, *4.1, 4.2, 4.3 (step 7), 4.4* and *4.2, 5.2, 5.6, 5.8, 6.4, 7, 10*; `skein/docs/design/forge-github.status.md`, **Not yet**, *11*; `skein/docs/design/fake-forge.status.md`, **Not yet**, *10.1*. In `forge-github.status.md`, narrow **Not yet**, *13* to: no live run has observed `app-owner-id-login-type`.
9. **The focused tests pass** in `skein-forge`, `skein-forge-setup`, `skein-fake-forge-github`, `skein-forge-kit`, `skein-fake-forge-tests`, `skein-fake-peers-tests`, `skein-forge-connection`, `github-conformance`, `skein-forge-dialect-tests`, `skein-forge-machine-world`, `skein-forge-protocol-world`, `skein-forge-setup-world`, `temper-forge-setup`, `temper-protocol-records`, `temper-codecs-world`, `temper-fake`, `temper-engine-domain-forge`, `temper-init` and `temper-service`. This changes what `Convert` and `GET /app` answer, so these are its readers' tests. The fuzzy suite of `skein-forge-setup-world` passes, in its `fuzzy` profile. `heavy cargo check --tests` passes for `temper-github-app`, `temper-real-loop-world`, `temper-one-process-world`, `temper-client-browser-world` and `temper-engine-protocol-forge-world`: compiled, never run.

## Inputs

- `temper/docs/plan/ui-forge-setup/notes/contracts.md`, *The App's account* and *The plans' answers*: they bind.
- `temper/docs/plan/ui-forge-setup/notes/common.md`.
- `skein/docs/design/forge-setup.md`, 4.1, 4.2, 4.3 (step 7), 4.4, 5.2, 5.6, 5.8, 6.4, 7 and 10.
- `skein/docs/design/forge-github.md`, section 11 (*The App's own reads*).
- `skein/docs/design/fake-forge.md`, 10.1 (*The App*).
- `temper/docs/design/draft/forge-identity.md`, 4.1 and 4.2, for the reason only: don't build its link.

## Owns and may change

**Owned:** the `files` above.

**Readers in skein, each edited only as named:**

| What changes | Sites |
|---|---|
| a literal of `document::App` gains `owner` (`None` unless the test asserts it) | `skein-forge/src/github/tests/app_facts.rs`, `skein-fake-forge-github/src/service.rs`, `skein-forge-setup/src/tests/{github_convert,github_send,github_fixtures}.rs` |
| `Outcome::Lacking { lacking }` gains `owner` or `..` | `skein-forge-setup/src/tests/{github_hook,github_installations,github_paste,github_reconcile}.rs`, `skein/tests/forge/setup/src/fixtures.rs`, `skein/tests/forge/setup/tests/suite/github_paste.rs` |
| a Convert's `Outcome::Nothing` becomes `Made { .. }` | `skein-forge-setup/src/tests/github_convert.rs`, `skein/tests/forge/setup/src/fixtures.rs`, `skein/tests/forge/setup/tests/suite/github_convert.rs` (`done`), and `github_host.rs` where its plan is a Convert |
| an exhaustive `Item`, `Outcome`, `Stage` or `Place` gains its arm | `forgejo/reconcile/tokens.rs`, `forgejo/reconcile.rs`, `tests/{fixtures,github_manifest,given,limits}.rs` |
| a literal of Forgejo's `Given` gains `person: None` | `skein-forge-setup/src/tests/fixtures.rs`, `skein/tests/forge/setup/src/fixtures.rs` |
| the item lists a Convert writes gain `AppMade` | `skein/tests/forge/setup/tests/suite/github_convert.rs` (`ITEMS`), `skein/tests/forge/setup/src/host.rs` where it stores items |

**Readers in temper, each edited only as named.** This is the plan's required crossing: these lines name what skein's change requires, and nothing else in temper changes.

| File | Edit |
|---|---|
| `temper/crates/temper-forge-setup/src/given.rs` | `values`: `person: None`. `refusal`: `Place::OrgOwner \| Place::Person` to the person's login path; `Place::Person` beside `OrgOwner` in the `other` arm |
| `temper/crates/temper-forge-setup/src/place.rs` | `item`: `Item::AppMade` joins the `unreachable!` arm, whose words already say GitHub's items are records |
| `temper/crates/temper-service/src/setup.rs` | `item_record`: `I::AppMade => Item::AppMade` |
| `temper/crates/temper-protocol-records/src/forge.rs` | the `kit` row `made-record` checks as `item::Item::AppId` becomes `item::Item::AppMade` |
| `temper/crates/temper-service/src/setup/lifecycle.rs` | `done`: `Lacking { lacking, .. }`, both sites; `Outcome::Made { .. } \| Outcome::Person { .. }` join `Nothing`, to `SetupOutcome::Done`. The stage match: `Stage::Person` joins the `unreachable!` arm |
| `temper/crates/temper-service/src/setup/making.rs` | `Stage::Person` joins the arm of `MakeStep::Repository` |
| `temper/crates/temper-service/src/setup/public.rs` | `Stage::Person => domain::SetupStage::Admin`, the token that read it: no wire change |
| `temper/crates/temper-service/src/setup/settings.rs` | `Place::Person` joins the `unreachable!` arm of the kit's limit check |
| `temper/crates/temper-service/src/tests/setup.rs`, `setup_lifecycle.rs` | a Convert's expected items gain `Item::AppMade` first (the survey found `setup.rs`, 439, and `setup_lifecycle.rs`, 135, 970, 1015 and 1497; check each is a Convert's, as `Paste`'s stay as they are) |

**Options, each with its default:**

- **A nullable, optional field:** `walk.rs`'s `Field` has both flags, but no constructor for both. Add one beside `optional`.
- **`fixture::app()`'s `owner: 1`** names no account in `fixture::accounts()`. Make it `PERSON`, so that a world's Reconcile answers an account. If a test of the fake depends on 1, keep that test's value and say so.
- **A golden or byte-exact expectation of `GET /app`'s body** in `skein/tests/forge/dialect` or `skein/crates/skein-fake-forge-github/src/tests.rs`: update it for `owner`, and say so.
- **A heap or memory bound** in `skein/tests/forge/setup/src/memory.rs` tripped by the longer document: raise that fixture's figure by what's needed, and say so. It's a fixture's bound, not a budget.
- **A temper test that asserts a Convert's write count by number:** add one, and say so.
- **`temper/tests/real-loop`, `one-process`, `client-browser`:** the survey found no assertion of these answers. Compile them; if one needs a line, change it, compile it, don't run it.

## Steps

1. **The document,** per the contract. Give `GET /app`'s owner its own walk node, whose `id`, `login` and `type` are each `optional`, and `Node::App` an `owner` that's optional and nullable. Build `Some` only at the node's end, when all three are good; an unknown `type` is no error. The node's arms join each exhaustive `Node` list. `State::new`'s `app` literal gains `owner: None`, and it must reset nothing else: a document decodes once.
2. **The server half and the fake.** `server/app.rs`, `app_json`, as the contract says. `skein-fake-forge-github/src/service.rs`, `app_identity`:

   ```rust
   let owner = self.accounts.iter().find(|a| a.id == self.app.owner).map(|a| d::AppOwner {
       id: a.id,
       login: a.login.clone(),
       kind: if a.organization { d::Account::Organization } else { d::Account::User },
   });
   ```

3. **The case.** In `skein/tests/forge/github/src/cases.rs`, beside `jwt-iss-string`, and in `NAMES`:

   ```rust
   case(r, "app-owner-id-login-type", |r| {
       let response = r.app(Method::Get, "/app", b"")?;
       let json = response.json()?;
       let owner = json.root().get("owner");
       Ok(response.status == 200
           && /* owner.id a nonzero number */
           && /* owner.login a nonempty string */
           && /* owner.type "User" or "Organization" */)
   })?;
   ```

   In `forge-github.md`, section 13, **Tokens and identity**, name it after `jwt-iss-string`, with: "`GET /app`'s `owner` has `id`, `login` and `type`, as GitHub's documentation lists them; written, and not yet observed live."
4. **The person.** `forgejo/given.rs`: `person`, checked by `login`, refused at `Place::Person`. `forgejo/reconcile.rs`:

   ```rust
   enum State { /* ... */ Person(Box<[u8]>), Ended }
   // finish(): with a login, State::Person(login) and send(Request::UserOf { login })
   //           with none, as today: Need::Done(Outcome::Nothing)
   // advance(State::Person(login), event): match account(event) {
   //     Ok(user) => NonZeroU64::new(user.id), none is stop_named(Stage::Person, &login, Why::Unexpected)
   //     Err(Why::Refused(404)) => ended, Need::Done(Outcome::Person { id: None })
   //     Err(why) => stop_named(Stage::Person, &login, why)
   // }
   ```

   The admin's token and the shared secret go at the end, as `finish` drops them today, so they stay held through this step. `halt` and `step` list the new state, and a `halt` during the read names the login.
5. **The account in the answers.** `github/convert.rs`: `State::App` keeps `app.owner`. `github/reconcile.rs`: `Checked` and `State::Sweep` carry it through the hook and the sweep to `Done`. `github/paste.rs`, as its `Lacking` is built.
6. **The mark.** `item.rs` per the contract. `github/convert.rs`: after the bot's id, a state that writes `Item::AppMade` with the id's 8 bytes, and on its `Written` starts `Write` as today; `Convert`'s `subject` and `halt` list that state. `github/write.rs` doesn't change.
7. **The tests:**

   ```rust
   // skein-forge-setup/src/tests/reconcile.rs: a table on the fixture's Run, with a person given
   #[test]
   fn a_given_person_is_answered_by_id_and_one_gone_by_none() {
       for (answer, expected) in [
           (user_of_id(31), Need::Done(Outcome::Person { id: NonZeroU64::new(31) })),
           (status(404), Need::Done(Outcome::Person { id: None })),
           (user_of_id(0), stopped(Why::Unexpected)),   // Stage::Person, subject b"ada"
           (status(403), stopped(Why::Refused(403))),
           (down(), stopped(Why::Down)),
       ] { /* run to step 7; assert the UserOf send on the admin's token; assert expected */ }
   }

   // skein-forge-setup/src/tests/github_convert.rs: a_conversion_keeps_four_items_before_done
   // gains the AppMade row first, is renamed for five items, and ends Made { owner }.

   // skein-forge-setup/src/tests/github_reconcile.rs: a table of owner rows, User, Organization and
   // None, each walked from the App's read to Done(Lacking { lacking, owner }).

   // skein/tests/forge/setup/tests/suite/reconcile.rs: beside an_owned_forge_makes_a_missing_org_for_its_person
   #[test]
   fn a_given_person_is_answered_by_id_and_one_gone_by_none() {
       // rows: the login seeded on the fake forge (setup_mut), Person { id: Some(its id) };
       //       a login no account has, Person { id: None }, and nothing else stops.
       // A setter beside plans.rs's org_owner gives the login.
   }
   ```

   Rows of `tests/item.rs` and `tests/given.rs` as acceptance says; `suite/github_convert.rs` and `suite/github_reconcile.rs` assert the account as an existing assertion of `Done`, with no new story.
8. **Status,** per acceptance 8, and acceptance 3's line in section 13.
9. **Checks:** narrow tests as you go, then once `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge -p skein-forge-setup -p skein-fake-forge-github -p skein-forge-setup-world -p github-conformance -p temper-forge-setup -p temper-protocol-records -p temper-service --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`; acceptance 9; commit and push.

## Report

As the implementer's text says; also: that the live case was compiled and not run; the rule each new row bites on and the failure seen; each temper line you changed, as the crossing's record; and anything the next plan will find, a shape that fights its link among them.
