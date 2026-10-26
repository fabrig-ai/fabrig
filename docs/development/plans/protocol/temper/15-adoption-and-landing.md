# Session 15: the adoption form, a change landed on the throwaway Forgejo, the live tier

temper's client gets its one form (client.md, section 4; protocol.md,
section 12, steps 7, 8 and 12). The forge's side of the client root
gains its vocabulary (the adoption, its outcome and its reasons, field
for field with `temper-protocol-client-wire`), and the native client's
routing its records: the request encoded as the family `FORGE` at
`CLIENT_VERSION`, the outcome decoded, jig's refusal at admission and the
family's hello routed to the forge's side. Then the form itself: its
draft, its rules, one keyed request through the table, its progress and
outcome, the out-of-date page, and its page, bindings and navigation
link in the view. The joint world `tests/client` runs the adoption's
stories against temper's engine root. The process tier then lands a
change on the throwaway Forgejo with real git, kills the binary and
finds the work after a restart (domain/testing.md, 2.6); and the live tier runs
the chat and a small change with a real model, once, within the
README's budget (domain/testing.md, 2.7).

## Read first

- `client.md`, sections 3.1, 3.3, 4 (all), 5.2, 5.3, 8 and 10;
  `domain/testing.md`, 2.6 and 2.7; `llm.md`, section 6; `credentials.md`,
  sections 3 and 7; `domain/forge.md`, section 4.
- jig's `docs/design/protocol/client.md`, sections 5, 9, 10 and 11, and
  `client/domain/root.md`, sections 4 and 6, at the pin.
- `decisions.md`, 11, 16, 17 and 23; README, "Live services";
  `code-organization.md`, sections 1, 4, 5 and 6.
- `findings/jig-client.md`, sections 1, 2 and 4, Discrepancies 10, 12
  and 13; `findings/jig-service.md`, section 3 ("Live test") and
  Discrepancy 9.
- jig at the pin: `examples/ops/domain-client/src/{infrastructure.rs,
  route.rs:240-318, domain.rs:121-143}` (a side's keyed request, its
  progress and answer, `EntranceRefused`), `examples/ops/view/src/{root.rs,
  bindings.rs, infrastructure.rs:200-240}`; `tests/ops/src/client/transport.rs:385-410`
  (jig's refusal at admission made the side's answer);
  `crates/jig-protocol-client-link/src/translate/answers.rs:422-497`
  (`refusal`, crate-private; `answer`, public); `testing/jig-test/tests/live.rs`
  and `scripts/proof-live.sh`.
- The session files `temper/02-adoption.md` (02.2's
  `PartyInput::Application`, `Delivery::Application`; 02.3's
  `ApplicationOutcome`, `AdoptionRefusal`), `temper/03-wire-codecs.md`
  (03.2's schema), `temper/05-forgejo-tools.md` (05.1's
  `temper_throwaway_forgejo`), `temper/10-startup-and-binary.md` (10.4's
  support), `temper/11-client.md` (11.1's root, 11.2's view, 11.3's
  `App` and `Front`), `temper/12-native-client.md` (12.1's `route`,
  `NativeClient`; 12.2's person).
- temper: `tests/system/src/forge/stories.rs:9-60` (a chat delegating the
  coding procedure, an agent's edit and finish).

## What the design asks

- **The form** (client.md, 4.4): title "Adopt a repository into
  PROJECT"; fields forge ("Forge number", default `1`), owner, name, role
  (a choice group of three), home ("Make it the project's home",
  checked), prefix (`temper/`), landing, ci ("It runs CI", checked),
  checks; then "Adopt". "Every field is bounded"; "Checked before
  sending: each rule of the table that the draft alone decides ... A
  field that fails is marked invalid, with one status line under it
  saying the rule in words ... and nothing is sent." "Adopt makes one
  keyed request through the table (4.6): saved in the small store before
  it is sent, sent again with its key after a drop or a restart, parked
  while offline, made once". "After the outcome, the outcome stays shown
  until the person edits a field or leaves the page. Adopted, the fields
  go back to their defaults". "Out of date: ... the page says 'This
  temper is out of date for adopting repositories: run the temper the
  engine runs.', the button is disabled, and a pending adoption stays in
  the small store."
- **Its outcome** (client.md, 4.6 and 4.7): adopted, refused by the
  core ("jig's refusal facts, carried whole ... and decoded into jig's
  `Refusal` by the client's routing"), refused by the forge (eight
  reasons); "A refusal at admission ... is not an outcome: it is jig's
  answer ..., and the form shows it in jig's words"; the sentences of
  4.7's table, "unknown past the retention" among them.
- **Routing** (client.md, 3.3): "Parts by connector"; "Addresses: jig's
  to `jig-domain-client`; the forge's adoption page to the forge's side;
  every other child is told it left its page"; "A kept adoption has no
  form to show it: its answer goes to no child".
- **The view** (client.md, 5.2 and 5.3): "Every node a person acts on has
  a role and a name"; "Bindings: writing a field, choosing a role,
  toggling home or ci, pressing Adopt, following the outcome's link, and
  going to the page from the navigation entry. Typed input is made here";
  the navigation slot "holds one link, 'Adopt a repository', to `Adopt {
  project }` for the project the person is in, as jig's frame says it;
  with no project chosen, the slot holds nothing."
- **The joint world** (client.md, 10): "a repository adopted from the
  navigation's link, its outcome read and the engine holding it, and
  listed on the project's resources once jig's core lists adopted ones
  (4.3); refused: not found, a second home, a field the form refuses;
  ... a double press of Adopt, a reload with an adoption pending ...,
  each request made once." client.md, 4.3: jig's core at temper's pin
  "lists there only the roles a live task's resources narrowed, and
  never an owned one", so no story looks for the repository on the
  project's page.
- **The process tier's story** (domain/testing.md, 2.6, steps 3 to 6): "a
  repository is adopted through temper's client" with "It runs CI"
  unchecked and one check; "a goal is given whose change, its check
  passed, lands on the throwaway Forgejo, pushed by real git"; "the
  binary is killed, and started again on the same directory"; "the
  person finds the work, and the change's branch on the forge is at the
  run's commit". Its referee: "the forge's state, read through the
  throwaway Forgejo's API, shows the change; nothing the person saw
  before the kill is lost after it; no process the binary started
  outlives it."
- **The live tier** (domain/testing.md, 2.7): the borrowed login "read-only",
  written "to a private file in the run's log directory, removed at
  exit"; "the process tier's chat, answered by the real model; and one
  small change ... pushed by real git"; "at most 25 completions and 15
  minutes in all"; "the test fails when the transcripts count more than
  25".

## Choices this plan makes

- **15.1 is two increments, its routing first.** The forge's vocabulary
  and the native client's records of it (15.1a) land before the form
  (15.1b), so that no root request of the forge's ever reaches a routing
  that cannot send it; the skeleton's "forge family's records in
  `route`" move from 15.2 into 15.1a for that reason, and 15.2 is the
  joint world's stories. "15.1's form" (16.4's and 17.3's Needs) and the
  README's "15.2: 15.1" mean 15.1b landed.
- **The forge's side has a vocabulary of its own** (client.md, 2:
  neither root links a wire): `forge::{Adoption, Ask, Answer, Adopted,
  Kinds, Protection, Rule, Reason, Role}` mirror 03.2's `Adopt` and
  `AdoptAnswer` field for field; `Role` keeps `Adopted`, which the form
  never offers (later.md).
- **jig's refusal at admission reaches the form as `Answer::EntranceRefused`,**
  as `ops`'s infrastructure answer carries it (`documents.rs:455`); the
  routing knows a forge attempt by the bounded table `family::Attempts`
  (room for 11.1's table's requests), filled on `Send` and emptied on a
  final envelope, the one token table this plan's routing keeps.
- **The core's refusal is decoded with jig's public `answer`**
  (`translate/answers.rs:471`): the routing decodes `CoreRefused.refusal`
  as jig's wire `Refusal`, builds jig's `Final::Refused` and `Owned` around
  it, and takes `Answer::Refused(refusal)` back; jig's `refusal`
  translation (`:422`) is crate-private at the pin.
- **"Unknown past the retention" is an outcome:** 08.5b answers
  `Delivery::Application { outcome: None }` as `AdoptAnswer { outcome:
  unknown }` (03.2's variant; jig's clients' component has no unknown
  keyed answer), the routing decodes it as `forge::Answer::Unknown`, and
  the form shows client.md, 4.7's "unknown past the retention" sentence
  from that outcome.
- **The version is given at construction** beside the connector
  (client.md, 3.1): 11.1's `new(limits, seed, forge: u16)` becomes
  `new(limits, seed, forge: forge::Config)`, `forge::Config { connector:
  FORGE, version: CLIENT_VERSION }`; the forge's side compares the
  family's hello with it.
- **Typed input in the view** (client.md, 5.2): the forge number's digits
  as `Option<u16>`, the checks' words as `Option<Box<[u32]>>` (numbers from
  1, separated by spaces or commas, each once, at most the client's
  `checks`); the domain keeps each field's text and its own write count.
- **Checks and choices take `Press` and `Toggle` alike** (Discrepancy 13:
  jig's fake person only presses).
- **The navigation link reads the frame's project** (Discrepancy 10):
  `root.jig().frame_summary().project` (jig-domain-client `domain.rs:189`),
  where `ops`'s fixed its target at construction.
- **The joint world's forge:** 11.3's `Joint` (its `Front`) over temper's
  engine root on the engine's domain world, whose fake forge domain
  (02.1's ids) answers locate and the adoption's reads; 15.2 adds the
  forge's request and outcome to 11.3's `translate.rs` (the translation
  written once; client.md, 10), exhaustive both ways: the client's
  `forge::Ask` to the root's `PartyInput::Application` (the host filled
  from the world's forge base, the digest as `Joint` digests jig's
  requests), and
  `Delivery::Application` back to `forge::Answer`.
- **The process tier's change** (domain/testing.md, 2.6): the throwaway Forgejo
  (05.1's `Settings::small()` with `actions: false`), users `acme` and
  `temper-bot`, a repository `acme/widget` made with `Init::Readme`,
  `temper-bot` its collaborator with `write`; the forge's API token and
  the identity's token both `temper-bot`'s; the landing rule of
  domain/testing.md, 2.6 (`"ci": false, "gates": [], "approvals": []`) on
  `branch main` of `acme/widget`; the person adopts `acme/widget` with "It
  runs CI" unchecked and checks `1`; the person's chat cues the fake
  LLM's chat script delegating the coding procedure on the adopted
  repository (`tests/system/src/forge/stories.rs:9-26`, its connector 1
  and the adopted repository's resource), the coding agent's script edits
  `README.md` and finishes (`:28-60`), and the gate's agent passes the
  check (Questions).
- **The live tier's mode by its account file:** `live.rs` runs against the
  real model when `$TEMPER_RUN_LOGS/live-account.json` exists (only while
  `scripts/proof-live.sh` runs), and against skein's fake LLM peer
  otherwise, the same stories and checks; so "first passing against the
  fake peer" is `cargo nextest run -p temper --profile live`
  with no account file. As the README says, 15.4's live run
  (`proof-live.sh`) runs once; its fake-peer run does not count.
- **`tools/live-login` takes its paths as arguments** (code-organization.md,
  1: no variable but section 1's): `live-login AUTH_JSON OUT_DIR [--models
  MODELS_TOML]`, the script passing `"$HOME/.codex/auth.json"`,
  `"$TEMPER_RUN_LOGS"` and, unless `TEMPER_LIVE_MODEL` is set, `--models
  "$HOME/src/rust/smith/benchmarks/agents/models.toml"`; the model is
  `TEMPER_LIVE_MODEL`, else `[small.codex]`'s `model` in that file, read by
  a line scan (decision 10: no TOML crate); no directory is walked up.

## Rules that bind it

- "No increment of this plan: ... adds a field or predicate that its
  counterpart across a boundary lacks: a record mirrors its domain type
  field for field" (code-organization.md, section 6): `forge::Adoption`
  and `forge::Answer` against 03.2's records; the routing's translation
  matches both without a wildcard.
- "in `temper-shell-terminal`'s step modules (`route`, ...), uses `Vec`,
  `String`, `format!`, `std::`, a closure or a `loop`/`while`"; the
  check `grep -nE 'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'`
  over `route.rs` and `route/*.rs` prints nothing (code-organization.md,
  section 6); it also matches `||`.
- **Live services** (README): "at most 25 completions and 15 minutes"
  per run; "at most 3 runs and 50 completions" for the plan; "15.4's
  Proof runs once"; "Nobody signs in, nothing refreshes the token,
  nothing writes `~/.codex`"; "It refuses a token that expires within 20
  minutes."

## Increments

### 15.1a The forge's vocabulary and its records in the routing

Scope: `temper-client-domain`: `forge/vocabulary.rs` (the sketch's
`Address` to `Answer`, `Config`; `Served` and `Event::Served`, kept by the
forge's side); the root's `Address::Forge`, `Ask::Forge`,
`Answered::Forge`, `Event::Forge(forge::Event)` (`Served` alone here),
the table's `Owner::Forge(Token)`; `new(limits, seed, forge:
forge::Config)` in place of 11.1's `forge: u16`; the forge's
`Event::Served` from the shell; addresses routed as client.md, 3.3 says.
`temper-shell-terminal`: `route/family.rs` (the sketch's `family`
module): `request` (an `Ask::Adopt` as 03.2's `Adopt`, field for field,
in `link::Family { family: FORGE, version: CLIENT_VERSION, bytes }`),
`answer` (03.2's `AdoptAnswer` to `forge::Answer`, `core_refusal` for
`CoreRefused`, `unknown` as `Answer::Unknown`), `restored` (a table
entry's request at `CLIENT_VERSION`;
another version `Notice::Dropped { keeper: Family(FORGE) }`), `Attempts`,
`served`; 12.1's `route::up` and `route::down` take `&mut family::Family`
(the sketch's `up` and `down`): `Send` of `Ask::Forge` to the link with
its attempt kept; `Answered::Family` decoded; `Answered::Jig(Refused)`
for a kept attempt as `EntranceRefused`; `Hello { family: Some(FORGE) }`
as `Event::Forge(Served)`; `saved` and `kept` with the forge's table
entries. `client.rs` makes the `Family` (room for the root's table's
requests) and passes it.
Needs: 03.2's records, 12.1's routing (the session's start).
Files: `crates/temper-client-domain/src/{lib.rs, boundary.rs, owner.rs,
address.rs, domain.rs, route.rs, store.rs, forge.rs, forge/vocabulary.rs,
tests.rs, tests/fixtures.rs}`; `crates/temper-shell-terminal/{Cargo.toml
(it gains `jig-protocol-client-wire`),
src/{route.rs, route/family.rs, client.rs, limits.rs, tests.rs,
tests/fixtures.rs, tests/family.rs}}`. Users of the root's `Address`,
`Ask`, `Answered`, `Event` and `Owner` (a variant each, an arm each per
the README's rule) and of `new` (its third argument):
`crates/temper-client-domain/src/*`,
`crates/temper-client-view/src/{root.rs, bindings.rs, tests/fixtures.rs}`,
`tests/client/src/{app.rs, front.rs, translate.rs}` (11.3's names; `git grep -n
'Answered::\|Ask::Jig\|Address::Jig' -- crates/temper-client-view
tests/client crates/temper-shell-terminal` lists them when the increment
starts).
Shape: the sketch's `forge` (`Address` to `Answer`, `Config`, `Served`)
and `family` modules.
Test shape: `temper-client-domain`'s `src/tests.rs` (11.1's, fixtures
gain `ADOPTION`, `ADOPTED`): `the_forge_address_goes_to_the_forge_side`,
`a_family_hello_outside_the_version_makes_the_side_out_of_date`.
`temper-shell-terminal`'s `src/tests/family.rs` (new; fixtures gain
`wire_limits()`, `adoption()` with owner `acme` and name `shop`):
`an_adoption_round_trips_through_the_family_record`,
`each_outcome_decodes_to_the_forge_sides_answer` (rows: adopted with each
protection, core refused (jig's `Refusal::Role`), each forge reason,
`unknown`),
`jigs_refusal_at_admission_of_a_forge_attempt_is_entrance_refused`,
`a_restored_adoption_of_another_version_is_dropped`. Under 20 ms each.
Reference test, in full:

```rust
// crates/temper-shell-terminal/src/tests/family.rs
#[test]
fn jigs_refusal_at_admission_of_a_forge_attempt_is_entrance_refused() {
    let mut family = fixtures::family();
    let sent = family::down(fixtures::send_adoption(fixtures::ATTEMPT), &mut family);
    let Down::Link(link::Event::Send { ask: link::Ask::Family(record), .. }) = sent else {
        panic!("the adoption leaves as the family's record");
    };
    assert_eq!((record.family, record.version), (FORGE, CLIENT_VERSION));
    let refused = link::Answered::Jig(contract::Answer::Refused(fixtures::role_refusal()));
    let up = family::up(fixtures::final_answer(fixtures::ATTEMPT, refused), &mut family);
    let Up::Root(client::Event::Answer { answer: Some(client::Answered::Forge(answer)), .. }) = up else {
        panic!("jig's refusal reaches the forge's side");
    };
    assert_eq!(answer, forge::Answer::EntranceRefused(fixtures::entrance(fixtures::role_refusal())));
}
```

Proof: unit, `cargo nextest run -p temper-client-domain -p
temper-shell-terminal`, run once: an adoption round-trips as the family's
record, every outcome decodes to the forge side's answer, jig's refusal
at admission reaches the side as `EntranceRefused`, a restored record of
another version is dropped, and the forge's address and hello reach the
forge's side (the six tests above); under 1 s. The joint layer runs them
at 15.2.
Don't: link a wire from `temper-client-domain`; decode the forge's
records in the root; keep any token in `route` but the forge's attempts.
Size: at most 450 lines.
Removes: nothing.

### 15.1b The adoption form and its page

Scope: `temper-client-domain`: `forge/form.rs` (the sketch's `Text` to
the end): the form made at `Went { address: Adopt { project } }` with the
defaults (decision 23), dropped at `Left` (client.md, 3.4); each `Act`
applied (a value past the client's field bound not taken, its field
`Long`; any write clears a shown outcome); `Adopt` checks `broken` and
sends nothing when a rule fails, else one `Request::Send` through the
table with the confirmed party as its scope, at most one pending, and
nothing while no party is confirmed (`Confirmed { party }`, which the
root passes the side beside the table; decisions.md, ruling 124);
`Progress` and
`Answered` kept for the page; adopted, the fields back to their
defaults; out of date, `Adopt` disabled and a pending request left in
the table, never sent: the root's `take` (`domain.rs`) pushes no
`Request::Send` whose ask is `Ask::Forge` while the side is out of date,
and the side shows its pending request as `Parked` (decisions.md,
ruling 125). `temper-client-view`: `forge/page.rs`, `forge/bindings.rs`,
`forge/words.rs` (the sketch's `view` module): the page after jig's
frame (a level-1 heading, the fields in 4.4's order with their names, the
three role choices, the two checks, the button disabled while pending or
out of date, a status per broken field with 4.4's sentence, a status for
progress in jig's words, the outcome as 4.7's sentences, a refusal as an
alert, the "See the project's resources" link to jig's project page,
its resources); the navigation slot's link; `decode` with typed input;
4.7's KINDS words.
Needs: nothing beyond 15.1a.
Files: `crates/temper-client-domain/src/{forge.rs, forge/form.rs,
route.rs, domain.rs (ruling 125), limits.rs, tests.rs, tests/form.rs, tests/fixtures.rs}`;
`crates/temper-client-view/src/{root.rs, forge.rs, forge/page.rs,
forge/bindings.rs, forge/words.rs, tests.rs, tests/forge_page.rs,
tests/fixtures.rs}`; the root's `Action::Forge(forge::Act)`. Users of
the forge side's `Event` and `Request` and of the root's `Action`
(variants added): `crates/temper-client-domain/src/route.rs`,
`crates/temper-client-view/src/root.rs` (`decode`),
`tests/client/src/app.rs` (11.3's `Temper`: `act`, `duty`, `unsent`).
Shape: the sketch's `forge` (from `Text`) and `view` modules; the forge
side's `Limits` gains `field_bytes` (at most the forge's `name_bytes`,
256 in temper's small profile) and `checks`.
Test shape: `temper-client-domain`'s `src/tests/form.rs`:
`a_new_form_holds_the_designs_defaults`,
`each_rule_the_draft_breaks_marks_its_field_and_sends_nothing` (rows:
owner empty `Required`; forge `0` `ForgeNumber`; checks `1, 1` `Checks`;
context with home `ContextHome`; no CI and no check `NoCheck`),
`a_value_past_its_bound_is_not_taken`,
`adopt_sends_one_keyed_request_and_a_second_press_none`,
`an_adopted_outcome_resets_the_fields_and_stays_shown`,
`out_of_date_disables_adopt_and_keeps_the_pending_request`.
`temper-client-view`'s `src/tests/forge_page.rs`:
`the_page_names_every_field_a_person_acts_on`,
`the_navigation_slot_links_the_frames_project_or_holds_nothing`,
`a_check_takes_a_press_or_a_toggle`,
`each_outcome_reads_as_the_designs_sentence` (rows: 4.7's table, "unknown
past the retention" from `Answer::Unknown` among them). Under
20 ms each. Reference test, in full:

```rust
// crates/temper-client-domain/src/tests/form.rs
#[test]
fn each_rule_the_draft_breaks_marks_its_field_and_sends_nothing() {
    // `None` is the home check; `Some(text)` a text field.
    let rows: [(Box<[forge::Act]>, Option<forge::Text>, forge::Broken); 5] = [
        (fixtures::owner_empty(), Some(forge::Text::Owner), forge::Broken::Required),
        (fixtures::forge_zero(), Some(forge::Text::Forge), forge::Broken::ForgeNumber),
        (fixtures::checks_twice(), Some(forge::Text::Checks), forge::Broken::Checks),
        (fixtures::context_home(), None, forge::Broken::ContextHome),
        (fixtures::no_ci_no_check(), Some(forge::Text::Checks), forge::Broken::NoCheck),
    ];
    for (acts, place, broken) in rows {
        let mut root = fixtures::on_adoption_page();
        let sent = fixtures::act_then_adopt(&mut root, &acts);
        assert_eq!(sent, 0, "nothing is sent: {broken:?}");
        assert_eq!(fixtures::marked(&root, place), Some(broken), "{broken:?} marks {place:?}");
    }
}
```

(Each row's acts fill the other fields validly; `fixtures::marked` reads
the form's mark at the place, `Check::broken` for the home check.)

```rust
// crates/temper-client-view/src/tests/forge_page.rs
#[test]
fn the_navigation_slot_links_the_frames_project_or_holds_nothing() {
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &fixtures::root_in_project(Some(1)));
    let link = fixtures::find(tree, Role::Link, b"Adopt a repository").expect("the link for project 1");
    assert_eq!(
        crate::decode(&mut view, link, jig_view::Act::Press),
        Some(Action::Forge(forge::Act::Go { address: forge::Address::Adopt { project: 1 } }))
    );
    let tree = fixtures::build(&mut view, &fixtures::root_in_project(None));
    assert_eq!(fixtures::find(tree, Role::Link, b"Adopt a repository"), None, "no project, no link");
}
```

(`fixtures::root_in_project` is a client root whose frame names that
project; `build` returns the view's last tree; `find` a node by role and
name; `crate::decode` is 11.2's.)
Proof: unit, `cargo nextest run -p temper-client-domain -p
temper-client-view`, run once: the defaults, each rule's refusal, the
field bound, one request per press, the reset after adoption, the
out-of-date page, the page's names, the navigation link and each
outcome's sentence (the ten tests above); under 1 s. The joint layer
runs the form at 15.2.
Don't: compute what the person may do (the engine judges, client.md,
5.3); keep the form across a restart (client.md, 3.4); send while a
request is pending.
Size: at most 550 lines.
Removes: nothing.

### 15.2 The adoption's stories in the joint world

Scope: `tests/client` (11.3's world): `translate.rs` gains the forge's
request and outcome (Choices), exhaustive both ways, and `Joint` routes
them; `Temper`'s (the `App`'s) `duty`, `unsent` and `card` gain the
forge's side; the engine world's fake forge holds the scenario's
repositories: 02.2's `ForgeStand::new()` is fixed to `org/repo` and
`org/private`, so `forge_stand.rs` gains `ForgeStand::with(repositories:
&[(&[u8], &[u8], u32)])` (owner, name, id; `new` calls it with 02.2's
two) and `direct::Driver` a constructor that takes them, and the joint
world's scenario holds `acme/shop` (id 7) and no `acme/none`; the stories
of client.md, 10: adopted, its outcome read, its link opening the
project's page, and the engine holding it (in place of the listing,
client.md, 4.3);
refused: not found, a second home, a field the form refuses; a double
press of Adopt made once; a reload with an adoption pending sent once.
`Delivery::Application`'s `None` reaches the client as 08.5b answers it,
`AdoptAnswer { outcome: unknown }`, so the form shows client.md, 4.7's
"unknown past the retention" sentence from `Answer::Unknown`: 15.1b's
`each_outcome_reads_as_the_designs_sentence` holds that row.
Needs: 15.1b's form; 11.3's joint world and 02.3's adoption (the
session's start and 02's, landed before).
Files: `tests/client/{src/{app.rs, front.rs, translate.rs,
translate/forge.rs, fixtures.rs}, tests/adoption.rs}`;
`tests/engine/domain/src/{forge_stand.rs, direct.rs}` (a constructor
each; their users unchanged). No production type changes Shape.
Shape: world code: `translate/forge.rs` (`adoption(&forge::Adoption, host:
&[u8]) -> temper_engine_domain_forge::Adoption`, `outcome(&ApplicationOutcome)
-> forge::Answer`, each a `match` with no wildcard).
Test shape: `tests/client/tests/adoption.rs`:
`a_repository_adopted_from_the_navigation_link_is_held_by_the_engine`
(the world gains `Joint::engine_adopted(forge: u16, repository: u32) ->
bool`: the engine store's forge row at
`Key::Repository(Repository { forge, repository })` names the
scenario's project and the owned role),
`adopting_a_repository_the_forge_lacks_says_not_found`,
`a_second_home_is_refused_by_the_forge_connector`,
`a_field_the_form_refuses_sends_nothing_to_the_engine`,
`a_double_press_of_adopt_adopts_once`,
`a_reload_with_an_adoption_pending_sends_it_once`. Each under 0.5 s.
Reference test, in full (names as 11.3 lands its world):

```rust
// tests/client/tests/adoption.rs
#[test]
fn a_repository_adopted_from_the_navigation_link_is_held_by_the_engine() {
    let mut world = fixtures::joint(fixtures::SEED);
    let outcome = world.run(fixtures::person(fixtures::adopt_then_open_resources(b"acme", b"shop")));
    assert_eq!(outcome.verdict, Verdict::Passed, "{:?}", outcome.why);
    assert_eq!(world.front().applications_admitted(), 1);
    assert!(world.saw_text(b"Adopted acme/shop on forge 1 as owned."), "the outcome's sentence");
    assert!(world.engine_adopted(1, 7), "the engine holds acme/shop, id 7, in the project, owned");
}
```

Proof: joint, `cargo nextest run -p temper-client-world -E
'binary(adoption)'`, run once: through temper's client and engine roots,
a repository adopted from the navigation's link is held by the engine
and its outcome read, three refusals read as the design's sentences, and
a double press and a reload each adopt once (the six tests above); their
sum within what is left of `tests/client`'s 1.0 s focused share after
11.4 (about 1.0 s already): `cargo nextest run -p
temper-client-world --profile measure -j 1`, then `awk -F'[][]' '/PASS \[/
{s += $2} END {printf "%.1f\n", s}' <its log>` (the README's, Done when
11), prints at most 1.0.
Don't: exceed the world's share (code-organization.md, 5: `tests/client`
1.0 s focused): past it, a story moves to the fuzzy suite
(`tests/client/tests/fuzzy_adoption.rs`), run by `cargo nextest
run -p temper-client-world --profile fuzzy -E 'binary(fuzzy_adoption)'`.
Size: world code; production 0 lines.
Removes: nothing.

### 15.3 A change landed on the throwaway Forgejo, killed and found

Scope: the process tier's story (domain/testing.md, 2.6, steps 1 to 6):
`tests/support/forgejo.rs` (the throwaway Forgejo started with
`temper_throwaway_forgejo::Forgejo::from_env()` (refused without
`TEMPER_FORGEJO`, naming it), provisioned as Choices say, stopped at the
test's end); `support::Forge::Throwaway` writing the forge (`api` its
loopback socket over plaintext, `git` its `http://127.0.0.1:PORT`, `user`
`temper-bot`'s id), the identity and the landing rule; the person's
scripts `adopt_and_change` and `found`; the referee polling the forge's
API every 2 s for the merged pull request, killing the binary once it is
merged, starting it again on the same directory, then checking the
forge's state and the second person's script. 12.2's test becomes this
story (its steps 1 and 2 kept as the first). At temper's pin of jig the
native client shows no page of a task that holds or is granted the
forge's resources (client.md, 5.4; domain/testing.md, 2.6), so the person's
scripts read no such page: `adopt_and_change` gives the goal in a chat,
then waits in the inbox for that chat's entry to count a reply (its
`replies_unread`, or its `last_reply` moving), not for the reply's words
on the chat's page; `found` reads the same inbox entries after the
restart. An inbox answered too large while such a chat is waiting or
held (its chip then carries its holdings) is waited past, as any page
not yet showing what the script waits for. The first chat (steps 1 and
2) holds and is granted nothing of the forge's, and its answer is still
read on its page. The process
configuration (`support::configuration`) adds the override
`documents.lines` 2 (configuration.md, 7), so such a page is the engine's
`TooLarge` and never an undecodable document at the client; 12.2's
`the_process_configurations_documents_fit_the_native_client` also
asserts `core.documents.lines` at most the native client's `rows`.
Needs: 14.1's forge component wired, 09.2's git component, 10.4's
process support, 12.2's story.
Files: `crates/temper/{Cargo.toml (dev-dependency
`temper-throwaway-forgejo`), tests/{process.rs, configuration.rs,
support/mod.rs, support/person.rs, support/forgejo.rs}}`.
Shape: the session's ordinary-Rust sketch of the support's growth below.
Test shape: `tests/process.rs`:
`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`,
replacing `the_binary_answers_a_chat_from_a_person_signed_in_by_its_link`,
aiming at 2 minutes, at most 4. Reference test, in full:

```rust
// crates/temper/tests/process.rs
#[test]
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    let scratch = support::Scratch::new("landed");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::fake_llm(support::landing_scripts(support::ANSWER));
    let first = support::run_until_landed(scratch.path(), support::person::adopt_and_change(&forgejo), peer, &forgejo);
    assert_eq!((first.signed_in, first.finished), (1, true), "{}", String::from_utf8_lossy(&first.stderr));
    let merged = forgejo.merged_pull().expect("the change's pull request is merged");
    assert!(forgejo.main_readme().ends_with(support::EDIT), "main holds the agent's edit");
    let second = support::run(scratch.path(), support::person::found(support::ANSWER), None, Duration::from_secs(60));
    assert!(second.finished, "the work is found after the restart");
    assert_eq!(forgejo.merged_pull(), Some(merged), "the forge's state is unchanged by the restart");
    forgejo.stop();
    scratch.remove();
}
```

Proof: process, `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
scripts/proof-one-process.sh`, run once: the real binary
with the throwaway Forgejo and the machine's git: the person signs in by
the printed link, a chat is answered, `acme/widget` is adopted with "It
runs CI" unchecked and one check, the change lands (its pull request
merged, `main` holding the edit), the binary is killed and started again
on the same directory, and the person finds the work (the test above,
beside 10.4's); under 4 minutes for the profile.
Don't: start any Forgejo but `TEMPER_FORGEJO`'s, or one under `/tmp`;
write a token to the log; skip the restart; wait for a page of a task
that holds or is granted the forge's resources, or of the project
(its policy names the forge's patterns): the native client takes none
at temper's pin; shorten temper's resource names or patterns to fit.
Size: test code; production 0 lines.
Removes: 12.2's test (its steps are this story's first).

### 15.4 The live tier

Scope: `tools/live-login` (ordinary Rust binary, `[lints]` as
`tools/forgejo-conformance`'s, `tests/clippy.toml`'s rules; dependencies
`serde_json`, `skein-lib` (base64), `skein-shell` (the clock)):
`live-login AUTH_JSON OUT_DIR [--models MODELS_TOML]` reads `AUTH_JSON`
once, read only; takes
`tokens.access_token`, its `exp` (the JWT's payload, base64url), and
`tokens.account_id`; refuses a token expiring within 1,200 s, or a
missing field, with one line on stderr and exit 1; writes
`OUT_DIR/live-account.json` (`0o600`: `access_token`, `expires_at`,
`account_id`, nothing else) and `OUT_DIR/live-config.json` (`account_file`,
`model`: `TEMPER_LIVE_MODEL`, else `[small.codex]`'s `model` in
`MODELS_TOML`, a line scan; `address`: `chatgpt.com:443` resolved to an
IPv4 socket address); prints `live-login: model M, token valid for N s`.
`scripts/proof-live.sh` (the sketch below). `crates/temper/tests/live.rs`
(`[[test]] name = "live"`): one test, two stories (the chat answered;
one small change in `acme/widget` adopted as 15.3's and pushed; the
story ends once the change's branch on the forge is at the run's
commit; the person's scripts are 15.3's, which read no page of a forge
task), with the real model when the account file exists (its endpoint
`chatgpt.com` over TLS with the machine's roots, its headers
`skein_llm::openai::identity::headers()`, the charters' `turns` bounding
the two stories to at most 20 completions, `time_ms` 240,000), else the
fake peer; completions counted from the transcripts' frames under
`state/transcripts/` (`live.rs:67-88` at the pin, every transcript file);
fails past 25; prints `live completions=N; wall=Ss; mode=M`.
Needs: 15.3's story.
Files: `tools/live-login/{Cargo.toml, clippy.toml, src/main.rs,
src/tests.rs}`; `scripts/proof-live.sh`, mode 100755 (`git ls-files -s
scripts/proof-*.sh` shows 100755); `crates/temper/{Cargo.toml,
tests/live.rs, tests/support/{mod.rs, live.rs}}`; the workspace `Cargo.toml`
(member `tools/live-login`). Shared hotspot: the workspace `Cargo.toml`.
Shape: the session's ordinary-Rust sketch of `live-login` and the
script below.
Test shape: `tools/live-login/src/tests.rs` (no file):
`a_token_expiring_within_twenty_minutes_is_refused`,
`the_account_file_holds_three_members_and_nothing_else`,
`the_small_codex_model_is_read_from_its_table`. `crates/temper/tests/live.rs`:
`a_real_model_answers_a_chat_and_pushes_a_small_change`. Reference tests,
in full:

```rust
// tools/live-login/src/tests.rs
#[test]
fn a_token_expiring_within_twenty_minutes_is_refused() {
    let now = 1_800_000_000_u64;
    let auth = auth(&jwt(now + 1_199), "acct-1");
    assert_eq!(
        super::borrowed(auth.as_bytes(), now),
        Err("live-login: the access token expires within 20 minutes".to_owned())
    );
}
```

(`jwt(exp)` makes a token whose payload is `{"exp": exp}` in base64url,
and `auth(token, account)` the login's JSON, both helpers of
`src/tests.rs`; `Account` derives `Debug` and `PartialEq`.)

```rust
// crates/temper/tests/live.rs
mod support;
use skein_lib::Duration;

#[test]
fn a_real_model_answers_a_chat_and_pushes_a_small_change() {
    let mode = support::live::mode();
    let scratch = support::Scratch::new("live");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::live::configure(scratch.path(), &mode);
    let seen = support::live::run(scratch.path(), &forgejo, peer, Duration::from_secs(840));
    let completions = support::live::completions(scratch.path());
    eprintln!("live completions={completions}; wall={}s; mode={mode:?}", seen.wall_seconds);
    assert!(seen.finished, "the chat's answer was seen: {}", String::from_utf8_lossy(&seen.stderr));
    assert_eq!(forgejo.branch_head(&seen.branch), Some(seen.pushed), "the change's branch is at the run's commit");
    assert!((1..=25).contains(&completions), "at most 25 completions, at least one: {completions}");
    forgejo.stop();
    scratch.remove();
}
```

Proof: live, two runs, each once:
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
cargo nextest run -p temper --profile live` (no account
file: the fake peer) passes; then
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
TEMPER_RUN_LOGS=<run>/logs/temper-15 scripts/proof-live.sh`
passes: the real model answers the chat and its small change's branch
reaches the run's commit on the throwaway Forgejo, at most 25
completions and 900 s. As the README says, 15.4's live run
(`proof-live.sh`) runs once; its fake-peer run does not count. The
commit body carries the live run's `Completions: N` and `Wall: S s`
lines in a paragraph of their own, before the trailers' paragraph. A run
that fails on the model's words runs again once, on the supervisor's
ruling (README, "Live services").
Don't: write `~/.codex`, read its refresh token, or refresh or sign in
anything; leave the account file after the script; run the live script
more than once without a ruling.
Size: test code; at most 150 lines of production code (`live-login`).
Removes: nothing.

## Interface sketches

`s15.rs` holds the client's side, compiled: the forge's vocabulary and
form (15.1a, 15.1b), the view's page and bindings (15.1b) and the
routing's family (15.1a), whose `up` and `down` return 12.1's `Up` and
`Down` and take 12.1's stand-in for 11.1's `Request`. The process tier's
growth, `live-login` and the script are ordinary Rust, below it, not
compiled.

```rust
//! Sketches of session 15: the forge's vocabulary on the client's side and
//! its records in the native client's routing (15.1a), and the adoption form
//! in the client root and the view (15.1b). The joint world's stories
//! (15.2), the process tier's change landed (15.3) and the live tier (15.4,
//! `tools/live-login`) are ordinary Rust: their sketches are in the session
//! file only.
//!
//! Who owns what: the forge's side owns the form's draft, its rules, its one
//! pending request's token and its last outcome; the request table owns the
//! request's key, its record and its sending; the view owns the words and
//! the nodes; the routing owns the bytes. What ends what: one keyed adoption
//! ends with one answer: adopted, refused by the core, refused by the forge,
//! refused at admission, or unknown past the engine's retention (client.md,
//! 4.6 and 4.7).
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

/// `crates/temper-client-domain/src/forge/vocabulary.rs` (15.1a: `Address` to
/// `Answer`, field for field with 03.2's `Adopt` and `AdoptAnswer`; client.md,
/// 4.4 to 4.7) and `forge/form.rs` (15.1b: `Text` to the end, the form's
/// state, its rules, its events and requests).
pub mod forge {
    use alloc::boxed::Box;
    use jig_domain_client::contract::{EntranceRefusal, Refusal, VersionRange};
    use skein_lib::{RequestProgress, Token};

    /// The forge's one address: the adoption page for a project (client.md,
    /// 4.5); reached from the navigation slot's link.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Address {
        Adopt { project: u32 },
    }

    /// A repository's role, as the forge connector's `Role`, in its order;
    /// the form offers `Owned`, `Fork` and `Context`.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Role {
        Owned,
        Adopted,
        Fork,
        Context,
    }

    /// A person's keyed adoption (03.2's `Adopt`, field for field): sent by
    /// the form through the request table; ended by one `Answer`.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Adoption {
        pub project: u32,
        pub home: bool,
        pub forge: u16,
        pub owner: Box<[u8]>,
        pub name: Box<[u8]>,
        pub prefix: Box<[u8]>,
        pub role: Role,
        pub landing: Box<[u8]>,
        pub ci: bool,
        pub checks: Box<[u32]>,
    }

    /// The forge's keyed requests: one, the adoption.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Ask {
        Adopt(Adoption),
    }

    /// What temper may do in an adopted repository (03.2's `Kinds`).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[expect(clippy::struct_excessive_bools, reason = "each flag is one kind of the forge's, as the wire's record has it")]
    pub struct Kinds {
        pub read: bool,
        pub push: bool,
        pub open: bool,
        pub land: bool,
        pub review: bool,
        pub status: bool,
        pub comment: bool,
        pub issue: bool,
        pub branch: bool,
    }

    /// A branch protection rule as the forge connector read it (03.2's `Rule`).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Rule {
        pub branch: Box<[u8]>,
        pub contexts: Box<[Box<[u8]>]>,
        pub approvals: u32,
        pub dismiss_stale: bool,
    }

    /// Whether branch protection was read (03.2's `Protection`).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Protection {
        Unknown,
        Absent,
        Rule(Rule),
    }

    /// The repository as the forge connector recorded it (03.2's `Adopted`).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Adopted {
        pub forge: u16,
        pub repository: u32,
        pub owner: Box<[u8]>,
        pub name: Box<[u8]>,
        pub role: Role,
        pub home: bool,
        pub kinds: Kinds,
        pub protection: Protection,
    }

    /// The forge connector's reasons (03.2's `ForgeReason`; client.md, 4.7).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Reason {
        NoSuchForge,
        NotFound,
        NoAccess,
        Refused,
        InFlight,
        TooLarge,
        Unreachable,
        InvalidAnswer,
    }

    /// The adoption's terminal, as the routing decoded it.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Answer {
        /// The forge connector adopted it.
        Adopted(Adopted),
        /// The core refused the adoption the root made on the party's behalf.
        CoreRefused(Refusal),
        /// The forge connector refused it.
        ForgeRefused(Reason),
        /// jig refused the request at admission; it never reached the connector.
        EntranceRefused(EntranceRefusal),
        /// The outcome's retention ended: whether it was adopted is unknown
        /// (03.2's `unknown`, sent for 02.2's `Delivery::Application` with none).
        Unknown,
    }

    /// The form's text fields, in the order of client.md, 4.4.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Text {
        Forge,
        Owner,
        Name,
        Prefix,
        Landing,
        Checks,
    }

    /// The rule a field breaks, its status line's words in the view.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Broken {
        /// "Required."
        Required,
        /// "A forge number is from 1 to 65535."
        ForgeNumber,
        /// "Checks are numbers from 1, each once."
        Checks,
        /// "A context repository cannot be the project's home."
        ContextHome,
        /// "Without CI, name at least one check."
        NoCheck,
        /// A value past the field's bound, not taken; the bound in words.
        Long { bound: u32 },
    }

    /// A person's act on the form, typed by the view's bindings (client.md, 5.2).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Act {
        /// Go to the adoption page (the navigation's link).
        Go { address: Address },
        /// A field written whole: its text, and for `Forge` and `Checks` the
        /// value the view typed from it, none when the text is not one.
        Write { field: Text, text: Box<[u8]>, typed: Typed },
        /// A role chosen.
        Choose { role: Role },
        /// "Make it the project's home" toggled.
        Home,
        /// "It runs CI" toggled.
        Ci,
        /// "Adopt" pressed.
        Adopt,
    }

    /// What the view typed from a field's text.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Typed {
        /// Owner, name, prefix and landing are taken as written.
        Bytes,
        Forge(Option<u16>),
        Checks(Option<Box<[u32]>>),
    }

    /// One field's state: its text, the domain's count of its writes, and the
    /// rule it breaks, if any.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Field {
        pub text: Box<[u8]>,
        pub written: u64,
        pub broken: Option<Broken>,
    }

    /// The adoption page's state, for the view to build (client.md, 4.5).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Form {
        pub project: u32,
        pub texts: [Field; 6],
        pub role: Role,
        /// "Make it the project's home": its rule's status sits under it.
        pub home: Check,
        /// "It runs CI".
        pub ci: Check,
        /// The pending request's token and progress: at most one.
        pub pending: Option<(Token, RequestProgress)>,
        /// The last outcome, shown until a field is written or the page left.
        pub outcome: Option<Answer>,
        /// Whether the engine serves this client's version of the forge's records.
        pub served: Served,
    }

    /// A check of the form: pressed or not, and the rule it breaks, if any.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Check {
        pub on: bool,
        pub broken: Option<Broken>,
    }

    /// Where each rule the draft breaks is marked: the six text fields in
    /// the order of `Text`, and the home check.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Marks {
        pub texts: [Option<Broken>; 6],
        pub home: Option<Broken>,
    }

    /// Whether the engine serves the forge's records at this client's version.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Served {
        /// No hello for the family yet.
        Unknown,
        /// The engine's range holds `CLIENT_VERSION`.
        Yes,
        /// It does not: the page says the client is out of date.
        OutOfDate,
    }

    /// The forge side's construction values, given by the shell (client.md, 3.1).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Config {
        /// `temper-protocol-client-wire`'s `FORGE`.
        pub connector: u16,
        /// `temper-protocol-client-wire`'s `CLIENT_VERSION`.
        pub version: u32,
    }

    /// The events 15.1b adds to 11.1's forge side (15.1a adds `Served` alone).
    #[derive(Debug)]
    pub enum Event {
        /// The root went to the forge's address.
        Went { address: Address },
        /// The root went elsewhere: the form is dropped (client.md, 3.4).
        Left,
        /// A person's act.
        Act { act: Act },
        /// The table's progress for the pending request.
        Progress { request: Token, progress: RequestProgress },
        /// The table's one durable answer.
        Answered { request: Token, answer: Answer },
        /// The stream's hello for the family `FORGE`.
        Served { versions: VersionRange },
        /// The party confirmed, or none: the scope of its requests
        /// (ruling 124; jig's ops `infrastructure.rs:348-352`).
        Confirmed { party: Option<u64> },
    }

    /// The requests 15.1b adds to 11.1's forge side.
    #[derive(Debug)]
    pub enum Request {
        /// Show the forge's address (the root wraps it).
        Show { address: Address },
        /// One keyed request through the root's table; the confirmed party
        /// is its scope (ruling 124), the project in its `Ask`.
        Send { request: Token, scope: u64, ask: Ask },
    }

    /// The defaults of client.md, 4.4 and decision 23: forge `1`, home and
    /// CI checked, prefix `temper/`, role owned, every other field empty.
    #[must_use]
    pub fn form(project: u32) -> Form {
        unreachable!("sketch")
    }

    /// Each rule the draft alone decides (client.md, 4.4), where it is
    /// marked: `Required`, `ForgeNumber` and `Checks` at their fields,
    /// `NoCheck` at the checks, `ContextHome` at the home check; every mark
    /// none when the draft may be sent.
    #[must_use]
    pub fn broken(form: &Form, checks: u32) -> Marks {
        unreachable!("sketch")
    }

    /// The adoption the draft makes, when no rule is broken.
    #[must_use]
    pub fn adoption(form: &Form, checks: u32) -> Option<Adoption> {
        unreachable!("sketch")
    }
}

/// `crates/temper-client-view/src/forge/page.rs` and `forge/bindings.rs`
/// (15.1b): the adoption page after jig's frame, the navigation link, and
/// the acts its nodes bind (client.md, 5.2 and 5.3).
pub mod view {
    use super::forge;
    use alloc::boxed::Box;
    use skein_lib::Token;

    /// What a node of the forge's page means; one table per built tree.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Meaning {
        /// A text field: its act types the text it is given.
        Field(forge::Text),
        Role(forge::Role),
        Home,
        Ci,
        Adopt,
        /// The navigation slot's link.
        Navigate { project: u32 },
    }

    /// A binding's meaning by its token, for the last tree.
    #[derive(Debug)]
    pub struct Bindings {
        _table: (),
    }

    /// Builds the page into `builder`: a level-1 heading, the fields in the
    /// order of client.md, 4.4, each node named by its words, the button,
    /// then a status for progress and the outcome, or an alert for a refusal.
    pub fn page(builder: &mut jig_view::Builder, form: &forge::Form, project_name: &[u8], bindings: &mut Bindings) {
        unreachable!("sketch")
    }

    /// The navigation slot's fragment: one link, "Adopt a repository", for
    /// the frame's project; nothing with no project chosen.
    pub fn navigation(builder: &mut jig_view::Builder, project: Option<u32>, bindings: &mut Bindings) {
        unreachable!("sketch")
    }

    /// A shell's act on a bound node as the forge side's act; `Press` and
    /// `Toggle` alike on a check or a choice; `Input` typed by the field; none
    /// for a token of an older tree.
    #[must_use]
    pub fn decode(bindings: &Bindings, token: Token, act: jig_view::Act) -> Option<forge::Act> {
        unreachable!("sketch")
    }

    /// The forge number's digits as a number from 1 to 65535, or none.
    #[must_use]
    pub fn forge_number(text: &[u8]) -> Option<u16> {
        unreachable!("sketch")
    }

    /// The checks' words, numbers from 1 separated by spaces or commas, each
    /// once and at most `most`; none otherwise.
    #[must_use]
    pub fn checks(text: &[u8], most: u32) -> Option<Box<[u32]>> {
        unreachable!("sketch")
    }
}

/// `crates/temper-shell-terminal/src/route/family.rs` (15.1a): the forge
/// family's records between the link and the root (client.md, 4.6; jig's
/// `protocol/client.md`, 11), and the one bounded table the routing keeps.
pub mod family {
    use super::forge;
    use crate::s03::temper_protocol_client_wire as wire;
    use jig_domain_client::contract::Refusal;
    use jig_protocol_client_link as link;
    use skein_lib::Token;

    /// The forge's keyed requests in flight, by attempt, so that jig's
    /// refusal at admission of one reaches the forge's side as its
    /// `EntranceRefused`; at most the client table's requests.
    #[derive(Debug)]
    pub struct Attempts {
        _set: (),
    }

    /// Why an attempt was not kept: the table is full.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Full;

    impl Attempts {
        /// Room for `most` attempts.
        #[must_use]
        pub fn new(most: u32) -> Attempts {
            unreachable!("sketch")
        }
        /// An attempt of the forge's is sent.
        pub fn sent(&mut self, attempt: Token) -> Result<(), Full> {
            unreachable!("sketch")
        }
        /// Whether `attempt` is the forge's; a final envelope removes it.
        pub fn answered(&mut self, attempt: Token, last: bool) -> bool {
            unreachable!("sketch")
        }
    }

    /// What the routing keeps for the forge's family: its attempts in
    /// flight, and the limits its records are made and decoded within.
    #[derive(Debug)]
    pub struct Family {
        pub attempts: Attempts,
        pub wire: wire::v1::Limits,
        pub link: link::Limits,
    }

    /// 12.1's `route::up`, as 15.1a changes it: a family answer decoded for
    /// the forge's attempts, jig's refusal at admission of one as its
    /// `EntranceRefused`, the hello of `FORGE` to the forge's side, a table
    /// entry holding the forge's request restored.
    #[must_use]
    pub fn up(request: link::Request, family: &mut Family) -> crate::s12::route::Up {
        unreachable!("sketch")
    }

    /// 12.1's `route::down`, as 15.1a changes it: the forge's request sent
    /// as the family's record, its attempt kept.
    #[must_use]
    pub fn down(request: crate::s12::client::Request, family: &mut Family) -> crate::s12::route::Down {
        unreachable!("sketch")
    }

    /// The adoption as the link sends it: the family `FORGE` at `CLIENT_VERSION`.
    #[must_use]
    pub fn request(ask: &forge::Ask, limits: &wire::v1::Limits) -> Option<link::Family> {
        unreachable!("sketch")
    }

    /// A restored table entry's request, if it is the forge's at `CLIENT_VERSION`.
    #[must_use]
    pub fn restored(family: &link::Family, limits: &wire::v1::Limits) -> Option<forge::Ask> {
        unreachable!("sketch")
    }

    /// The family's final answer as the forge side's; none when it does not decode.
    #[must_use]
    pub fn answer(family: &link::Family, limits: &wire::v1::Limits, link: &link::Limits) -> Option<forge::Answer> {
        unreachable!("sketch")
    }

    /// The core's refusal carried whole in `CoreRefused`, decoded with jig's
    /// public `translate::answers::answer` on a final answer built around it
    /// (jig's own `refusal` translation is crate-private at the pin).
    #[must_use]
    pub fn core_refusal(bytes: &[u8], link: &link::Limits) -> Option<Refusal> {
        unreachable!("sketch")
    }

    /// Whether the engine's range for the family holds this client's version.
    #[must_use]
    pub fn served(versions: jig_domain_client::contract::VersionRange) -> forge::Served {
        unreachable!("sketch")
    }
}
```

The process tier's growth, ordinary Rust, not compiled:

```rust
// crates/temper/tests/support/forgejo.rs (ordinary Rust, not compiled)

/// The throwaway Forgejo of one test: `acme`, `temper-bot`, `acme/widget`.
pub struct Throwaway { instance: temper_throwaway_forgejo::Instance, acme: Account, bot: Account, widget: Made }
impl Throwaway {
    /// `Forgejo::from_env()` (refused naming `TEMPER_FORGEJO`), started in
    /// `scratch/forgejo` with `Settings { actions: false, ..Settings::small() }`.
    pub fn start(scratch: &Scratch) -> Throwaway;
    /// The forge the configuration names (10.4's `support::Forge`).
    pub fn forge(&self) -> Forge<'_>;
    /// The merged pull request's number, if one is merged.
    pub fn merged_pull(&self) -> Option<u64>;
    /// `README.md` on `main`, raw.
    pub fn main_readme(&self) -> Vec<u8>;
    /// A branch's head commit, as `/repos/acme/widget/branches/{branch}` says.
    pub fn branch_head(&self, branch: &[u8]) -> Option<Box<[u8]>>;
    pub fn stop(self);
}

// crates/temper/tests/support/mod.rs gains:
pub const EDIT: &[u8] = b"Changed by temper.\n";
/// The chat delegating the coding procedure, the coding agent's edit and
/// finish, and the gate's passing verdict.
pub fn landing_scripts(answer: &[u8]) -> Box<[skein_fake_llm_domain::api::Script]>;
/// `run` whose referee kills the binary once the forge shows the merge.
pub fn run_until_landed(directory: &Path, script: person::Script, peer: Peer, forgejo: &forgejo::Throwaway) -> Seen;

// crates/temper/tests/support/person.rs gains:
/// The chat answered, then the adoption form (navigation link, owner,
/// name, landing `main`, "It runs CI" pressed off, checks `1`, Adopt), its
/// outcome seen, then the chat that asks for the change.
pub fn adopt_and_change(forgejo: &forgejo::Throwaway) -> Script;
/// After a restart: the chat's page, its answer seen.
pub fn found(answer: &[u8]) -> Script;

// crates/temper/tests/support/live.rs (15.4)
/// The real model when `$TEMPER_RUN_LOGS/live-account.json` exists, else the fake peer.
#[derive(Debug)]
pub enum Mode { Fake, Live { account_file: PathBuf, model: String, address: SocketAddr } }
pub fn mode() -> Mode;
/// The live endpoint, model, charters' bounds and agent configuration
/// patched in (jig's `live.rs:13-65`); the fake peer for `Fake`, none for `Live`.
pub fn configure(directory: &Path, mode: &Mode) -> Option<Peer>;
/// The chat answered, then `acme/widget` adopted and a small change asked;
/// ends once the change's branch is pushed, or at `deadline`.
pub fn run(directory: &Path, forgejo: &forgejo::Throwaway, peer: Option<Peer>, deadline: Duration) -> Pushed;
/// What the live run showed: the person's script, the branch and commit pushed, its wall time.
pub struct Pushed { pub finished: bool, pub branch: Box<[u8]>, pub pushed: Box<[u8]>, pub wall_seconds: u64, pub stderr: Vec<u8> }
/// Every frame of every transcript file under `state/transcripts/` (jig's `live.rs:67-88`).
pub fn completions(directory: &Path) -> u32;
```

`tools/live-login`, ordinary Rust, not compiled:

```rust
// tools/live-login/src/main.rs (ordinary Rust, not compiled)
/// `live-login AUTH_JSON OUT_DIR [--models MODELS_TOML]`: exit 0 with the two files written, or 1
/// with one line on stderr; nothing secret printed.
fn main() -> std::process::ExitCode;
/// The account's three members from the login's bytes, refusing a token
/// that expires within 1,200 s of `now` (seconds since the epoch).
fn borrowed(auth: &[u8], now: u64) -> Result<Account, String>;
/// `TEMPER_LIVE_MODEL`, else `[small.codex]`'s `model` in `models` (the
/// script passes smith's `benchmarks/agents/models.toml`); no directory walked.
fn model(models: Option<&Path>) -> Result<String, String>;
#[derive(Debug, PartialEq)]
struct Account { access_token: String, expires_at: u64, account_id: String }
```

`scripts/proof-live.sh`, at most 20 lines, mode 100755:

```sh
#!/bin/sh
# The live tier (domain/testing.md, 2.7): a borrowed Codex login, read only;
# at most 25 completions (the test) and 15 minutes (the timeout).
set -eu
cd "$(dirname "$0")/.."
: "${TEMPER_RUN_LOGS:?the run's log directory}"
umask 077
mkdir -p "$TEMPER_RUN_LOGS"
trap 'rm -f "$TEMPER_RUN_LOGS/live-account.json"' EXIT
export NEXTEST_DOUBLE_SPAWN=0
cargo build -p temper --bin temper -p live-login --bin live-login
set --
[ -n "${TEMPER_LIVE_MODEL:-}" ] || set -- --models "$HOME/src/rust/smith/benchmarks/agents/models.toml"
target/debug/live-login "$HOME/.codex/auth.json" "$TEMPER_RUN_LOGS" "$@"
started=$(date +%s)
status=0
timeout --signal=TERM --kill-after=10s 900s \
    cargo nextest run -p temper --profile live --success-output immediate || status=$?
echo "proof-live: wall=$(( $(date +%s) - started ))s exit=$status"
exit "$status"
```

## Questions to raise when reached

- **The check's gate in the process tier** (15.3). domain/testing.md, 2.6: "a
  gate run as an agent task whose verdict the fake LLM's script gives".
  Default: the gate's agent runs under `application.charter` and its
  script finishes with a passing verdict as `temper-engine-smith` reads
  one (`crates/temper-engine-smith/src/finish.rs:92`); if no gate's run
  reaches the fake model, raise it with the forge's change state.
- **A goal given in the process tier** (15.3). The design names "a goal";
  jig's pages offer a chat. Default: the chat that delegates the coding
  procedure, as the system world's story does; a goal page is a question.
- **The real model and the delegate call** (15.4). Default: the live
  chat's charter instructions name the exact `delegate` call and the
  coding charter's the exact edit; a model that does not follow them is
  "a run that fails on the model's words" (README).
- **A second home's refusal sentence** (15.2). client.md, 4.7 has one
  sentence for `Refused`'s three causes. Default: that sentence, as the
  design's open question keeps it.
- **The form's limits against the forge's.** Default: `field_bytes` 256
  (the small profile's `name_bytes`), `checks` the forge's
  `change_policy.gates`; past the wire's bounds, a limits test fails
  first.
- **jig's `refusal` made public later.** Default: `core_refusal` as
  Choices say until a repin.

## Done when

- The client's side passes: `cargo nextest run -p
  temper-client-domain -p temper-client-view -p temper-shell-terminal -p
  temper-client-world` passes.
- The routing keeps the subset: `grep -nE
  'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'
  crates/temper-shell-terminal/src/route.rs crates/temper-shell-terminal/src/route/*.rs`
  prints nothing.
- The process tier passed with the throwaway Forgejo once in 15.3's
  Proof, its log naming
  `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`.
- The live tier ran within its budget: `git -C ~/src/rust/temper log -1
  --grep='^Step: temper-15.4$' --format=%B origin/main | grep -E
  '^(Completions|Wall): '` prints `Completions: N` with N at most 25 and
  `Wall: S s` with S at most 900.
- No borrowed secret was kept: `test ! -e <run>/logs/temper-15/live-account.json`
  succeeds after the script.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-15\.4$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
