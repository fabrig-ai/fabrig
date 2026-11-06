+++
id = "add-guard"
part = "guard"
kind = "feature"
needs = ["add-writes-last", "replay-drawn", "redact", "scan"]
files = ["temper/tools/journeys/src/lib.rs", "temper/tools/journeys/src/main.rs", "temper/tools/journeys/src/tests.rs", "temper/tools/journeys/src/again.rs", "temper/tools/journeys/tests/suite/arguments.rs", "temper/tools/journeys/tests/suite/refusals.rs", "temper/tools/journeys/tests/suite/main.rs"]
effort = "high"
minutes = 60
+++

# `add` redacts, proves, scans, then writes

## Goal

The guard whole, at the program. `add [--redact TEXT]... NAME FILE` runs the steps of `notes/contracts.md`, in that order, and no refusal puts an original's bytes on stdout, stderr or a file on any path. `--help` and `add`'s counts say what's left to the person.

Acceptance:

- `add` and `Added`, and `Refusal`'s new variants, as `notes/contracts.md` gives them. No `fs` call comes before the last scan has passed.
- A drawn or printed hit takes precedence over a proof's refusal: a page that showed a secret is named even when the proof then fails.
- **What is printed:** a passing `add`, on stdout, the count of each kind and the places, then the sentence below. A refusal, on stderr, with exit 1: the line, the record's word and where in it, or the turn or page, and the kind in plain words, and that nothing was written. Never bytes of the recording, nor a `--redact` TEXT.
- `--help`, on stdout, exit 0: the usage, what `add` takes out, what it refuses, and these, the sentence the plan asked for: `These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.` and `FILE keeps every original: keep it out of the repository and remove it once added.`
- `again` adds with no given text, and is otherwise unchanged.
- **The tests below,** each seen failing first.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.6 *What `add` prints*, 3.7 *A refusal*, 3.8, 4.3, 5.1 and 6.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns the files in the header. `again.rs`: its one `crate::add` call only, with `&[]`, and an arm for any `Refusal` a `match` there needs.

Reader files, these call sites only: `src/tests.rs`, `add`'s calls, about lines 63, 66, 73 and 236, with `&[]`.

`tests/suite/main.rs`: `mod refusals;`.

Options, each pre-approved:

- **The arguments:** `add` takes `--redact TEXT` anywhere after it, as often as wanted. An empty TEXT, a `--redact` with no TEXT, any other argument starting `--`, or a wrong count is the usage line, exit 2, echoing nothing given. TEXT and FILE are bytes, by `OsStrExt`. Usage: `usage: temper-client-journeys [add [--redact TEXT]... NAME FILE | again NAME | --help]`; `arguments.rs`'s expected line changes with it.
- A NAME that fails the pattern keeps its refusal, which names it: a NAME is the person's word for the journey. If a mistaken order could put a TEXT there, the parse refuses first.
- `Added`'s words are yours, each kind's count named.

## Steps

In `src/tests.rs`, by `add` on a `Scratch` root, the fixture as `a_reprint_keeps_a_journeys_readme` builds one, with `secret: Some(fake)` so that the start asks `/jig/v1/sign-in/link`:

```rust
#[test] fn add_takes_the_link_out_and_the_replay_holds() // Added.link == 1; `recording` and each page hold no fake bytes; `secret "xxx..."`
#[test] fn add_takes_a_given_text_out()                  // a text in a heard answer's body, given: gone
#[test] fn a_shape_in_a_heard_answer_refuses()           // Refusal::Found; line, `heard`; the root as it was
#[test] fn a_given_text_the_page_draws_refuses()         // a word of the view: Refusal::FoundDrawn, ahead of the proof's own refusal
#[test] fn a_given_text_only_the_print_holds_refuses()   // `skein-ui.css`: Refusal::FoundPrinted, `01.html`; the root as it was
#[test] fn a_refusal_keeps_the_journey_of_its_name()
#[test] fn the_committed_journey_redacted_prints_the_same_pages() // welcome-playground: redact, print in memory, each page equals the committed one; the time reported
```

In `tests/suite/refusals.rs`, as a process, each recording in a `Scratch` directory; each test holds a list of every fake byte string in play and checks them all absent from stdout and stderr, by position, and checks that `temper/docs/design/client/journeys/<its name>` doesn't exist and that the listing of the journeys directory and of `temper/tools/journeys` is what it was before the run:

```rust
#[test] fn a_shape_prints_no_bytes()                // each shape
#[test] fn a_split_token_prints_no_bytes()          // a run's text across two pieces and two deliveries
#[test] fn a_failed_proof_prints_no_bytes()         // the exchange edited: stale, with a `--redact` given too
#[test] fn an_undecodable_line_prints_no_bytes()
#[test] fn a_differing_mark_prints_no_bytes()       // the end mark's html edited to hold a marker
#[test] fn usage_echoes_nothing_given()             // `add --redact SECRET`; `add --bogus SECRET a b`
```

In `tests/suite/arguments.rs`:

```rust
#[test] fn help_says_what_is_left_to_the_person()   // both sentences, exit 0, on stdout
```

A process that passes would write under the checkout, so these all refuse: pick NAMEs no committed journey has, and build each so that it refuses.

Then fmt, clippy on `temper-client-journeys`, `temper-client-shell-web`; both packages' tests, commit and push.

## Report

As the implementer's text says, with each test's first failure, and any path you found where an original could reach a stream or a file and how it's closed.
