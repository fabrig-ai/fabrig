+++
id = "person-words"
part = "person-words"
kind = "feature"
needs = ["note-room"]
files = [
  "jig/crates/jig-client-domain/src/limits.rs",
  "jig/crates/jig-client-domain/src/tests/limits.rs",
  "jig/crates/jig-client-protocol-link/src/limits.rs",
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client/src/shapes.rs",
  "jig/crates/jig-client/src/tests/shapes.rs",
  "jig/crates/jig-domain-people/src/batch.rs",
  "jig/crates/jig-domain-people/src/limits.rs",
  "jig/crates/jig-protocol-store/schema/people-v14.schema",
  "jig/crates/jig-protocol-store/src/generated/people_v14.rs",
  "jig/crates/jig-protocol-config/src/profile/standard.rs",
  "jig/crates/jig-protocol-config/src/profile/small.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-shell-terminal/src/limits.rs",
  "temper/crates/temper-client-tools/src/limits.rs",
  "jig/docs/design/protocol/configuration.status.md",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 85
+++

# What a person reads or sends whole: a note's body, a task's words

## Goal

A client reads a note's body whole and sends a goal's words whole, past a field's 512 bytes and `people.words`' 8,192, so that a chat's document reaches the page and a goal can take it in its words.

Acceptance, as `notes/contracts.md`, *A client's long text* and *A person's words on the engine's side*:

- `Limits.long_text` in `jig-client-domain`, `valid`'s two relations and `ask_bytes`' relation, in the link's `wire`; temper's three clients set `LONG_TEXT`, 131,072. A request table's bytes (`requests.bytes`) hold an ask of it in each client: raise them, at their definitions, by what it takes.
- `client-v19.schema`, `Specification.words` to 131,072, in place; generated and goldens by their own tool (`skein-codegen`; the wire's drift test with `SKEIN_CODEGEN_WRITE=1`, `skein/docs/design/codec.md`).
- `people::Limits.spec_bytes`, `batch::valid`'s `words` by it, set where the core's limits are composed; people-v14's ceiling and `people.amendment_bytes` as the note says, in place, its goldens by the store's drift test. `jig-protocol-config`'s tests hold: `amendment_bytes_max` and the envelope's check, `store.value`'s pinned number by its new one, each bound that follows at its own definition.
- `shapes::set_agreed_goal_from`, `Specification { asks: true, .. }`, its `fill` taking the flag. `set_goal_from` unchanged in signature.
- A story in `jig/tests/client` (its `src/joint/engine_stories.rs`, or `tests/suite` by its neighbours): `a_large_document_and_a_large_goal_cross_the_wire`, at `standard`'s figures: a run writes a note of 60,000 bytes, a party's client reads it whole (`cut` false), and that party's client creates a goal whose words hold it. `a_message_still_stops_at_people_words`: a message of `people.words + 1` is refused as it is today.
- In `jig/docs/design/protocol/configuration.status.md`, the line on `person-words` goes. In `client/domain/client.status.md`, the entries on `long_text` and `set_agreed_goal_from` go.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `jig/docs/design/client/domain/client.md`, section 14 and 19; `jig/docs/design/protocol/configuration.md`, section 6's envelope, its effective codec; `jig/docs/design/domain/people.md` on `words` and `amendment_bytes`.
- `temper/docs/design/client/README.md`, 5.4.

## Owns and may change

Owns, as `files` lists them.

May also change:

- Every `Limits` literal the new field forces: `jig/crates/jig-client/src/tests/fixtures.rs`, `jig-client-domain/src/tests/fixtures.rs`, `jig/testing/**`, `jig/tests/**`, `temper/tests/**`, `temper/crates/*/src/tests/**`, `temper/crates/temper-client-*/src/limits.rs` and `temper/tools/**`: `long_text` their `text`, unless a test needs more.
- `jig-domain-core`, where it composes `people::Limits` (`rg -n 'people:' jig/crates/jig-domain-core/src/limits.rs`), `jig-protocol-store`'s codec and goldens for people-v14; `jig/crates/jig-protocol-config/src/**` for what follows.
- `temper/docs/help/reference/*`, by its renderer, if a figure there moves.
- `temper/crates/temper-client-view/src/limits.rs` and `temper/crates/temper-client-domain/src/**`, only if a check on their limits demands it.

Options, each pre-approved:

- **A worst case priced on `document_text_bytes`** (a document's held bytes, in the link's `receiving`) that grows past the client's `document_bytes`: raise `document_bytes` in that client by what it takes, name it. Each `DocumentText` field keeps its own bound by its row's bytes.
- **The envelope:** if people-v14's envelope can't hold 135,168 for a reason besides its member words' ceiling (a field that isn't this one's), raise what it names, in place. No deployment is kept.
- **A `people.amendment_bytes`-priced row count** that blows a memory check: reduce nothing; raise that memory figure at its definition, and name it.
- **`notes-list`** follows you in the client's wire: it's chained behind you, so you hold the schema alone.

## Steps

1. The client's limit, the wire, the link, the three clients.
2. The people side: the limit, the ceiling, the amendment, and what follows.
3. `shapes`, with `set_agreed_goal_from_asks` in `tests/shapes.rs`.
4. The stories, each having bitten: drop `long_text` from `document_text_bytes` and see the read refused; drop `spec_bytes` from `batch::valid` and see the create refused.
5. `heavy cargo nextest run -p jig-client-domain -p jig-client -p jig-client-protocol-link -p jig-client-protocol-wire -p jig-domain-people -p jig-protocol-store -p jig-protocol-config`; `-p temper-client-domain -p temper-client-view -p temper-client-tools --lib`; the story's world; `heavy cargo check --tests` of each other reader you changed, by package.
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: every bound that moved, with its arithmetic.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/person-words/foreman-1)

rebase person-words, preserve both sets of shape tests, and regenerate combined store layouts.
The merge failed on three paths; resolve each without dropping either feature.
- `jig/crates/jig-client/src/tests/shapes.rs`: retain `set_agreed_goal_from_asks` alongside `me_with_note_scopes`, `a_goal_keeps_its_note_scopes`, and `a_chat_keeps_its_note_scopes` from integration commits `b9af910908` and `9fe211b11b`.
- Preserve integration’s final note-scope behavior in `shapes.rs`, including `9fe211b11b`’s removal of note-scope clearing, together with this task’s `asks` plumbing.
- `jig/crates/jig-protocol-store/src/generated/layouts.rs`: combine main commit `9ef746a800`’s tasks fingerprint with this branch’s people fingerprint. Regenerate from the combined schemas using `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`; retain and report generated changes.
- `jig/docs/design/client/domain/client.status.md`: remove both the `long_text` entry and the entire section-19 gap line. Integration supplies note scopes; this branch supplies `set_agreed_goal_from`. Preserve unrelated gaps.
Run the focused `jig-client` shape tests and store drift verification, then the spec’s required checks against the rebased result. Regenerate and report any fixtures or goldens made stale by the combined code using their owning tools.
