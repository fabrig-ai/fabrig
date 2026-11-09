# ui-away-all: the plan

A follow-up to P1 and P2 of `whole-story.execution.md` (`inputs/request.md`):
`surfaces.md`, section 2, has Away on All projects set every project the
person is in and read "Away on 2 of 4" while they differ. P1 and P2
landed without it: on All projects the switch and the chip are the rail's
open project's.

Design written by this cut:

- `temper/docs/design/client/README.md`, 3.5, *Away*, a new bullet *On
  All projects*: what the switch counts (`pages::away_all::across`), its
  words and state, its act (`away::Act::ToggleAll`, one `SetAway` a
  project that changes), and its toast. 5.3's table of the rail's parts:
  the Away row's rule and fact on All projects.
- `temper/docs/design/client/browser.md`, 7.2: the gallery states
  `all-home-away` and `all-home-away-on`.
- Its status, `README.status.md`: a **Not yet** 3.5, *Away*, *On All
  projects*, which the task removes. `surfaces.status.md` had no entry for
  section 2's Away, so none closes.

## Decided

- **Every project `me` lists, not only those whose switch shows.** The
  draft says "every project the person is in"; M counts them all, and a
  press sets them all, so a project whose own switch is hidden (nothing
  Away may decide there) is set too and then shows its switch on, which
  is true. Counting only the shown ones would read "2 of 2" on a
  deployment where the person is away from two of four projects.
- **Shown on All projects while any project's switch shows.** A
  deployment with no switch anywhere shows none on All projects either.
- **A press turns every project on unless all are on;** with all on, it
  turns every one off. Between, the switch is not pressed and reads "Away
  on N of M", so its state says what a press does. skein has no mixed
  state, and none is needed.
- **The per-project request, one each, all at once.** Each changing
  project gets the request its own switch sends, from one function the
  per-project act and the All act share; a project already as asked gets
  nothing. If the client bounds the requests out, the task sends them one
  after another (pre-approved). While any is out, every Away act waits.
- **The toast counts what changed,** "Away turned on for 3 projects: …",
  with a one form; a partial answer says "Away changed for 2 of 3
  projects. Try again."; none changed is today's "Away didn’t change. Try
  again.". No "Limits" on it: limits are a project's.
- **When all agree, the words are a project's,** with the earliest
  `until` of the rows on, which is also when the build wakes, since N
  changes then.
- **Tests:** two small table tests in the domain (the count, the act and
  its toast) and rows of existing tables (`a_second_toggle_waits`, the rail
  facts, the view's switch test); two gallery states hold the words and
  the toast. No new client-world story: the per-project `SetAway` is
  already proven end to end, and `temper/tests/client/tests/suite/away.rs`
  is `ui-reaches/reaches-story`'s.
- **No public interface or stored format changes:** `away::Act` and
  `Toasted` gain a variant each, inside the client.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `away-all` | `away-all` | `temper-client-domain` (`pages/away_all.rs`, `pages/away.rs`, `pages.rs`, `pages/rail.rs`, `pages/needs.rs`, tests); `temper-client-view` (`frame.rs`, `words.rs`, `tests/rail.rs`); `temper-client-gallery` (`samples.rs`, `worlds/whole_story.rs`, `tests.rs`, `names.txt`); `README.status.md`'s entry | | 110 |

One task, as the request asks: the domain's act and the view's words are
small, and splitting them would hold the view behind the domain for no
parallel gain. Slice 1 is the part; `walk = false`.

## Critical path

`away-all` alone, released with the first push.

## Overlaps with drafts beside this one

Unavoidable: Away lives in these files.

- **ui-reaches** (its trunk): `pages/away.rs` (drops `Kinds` and `away::ask`'s
  `kinds`), `tests/away.rs`, `domain.rs`'s `away_shown` (by classes),
  `frame.rs`, `words.rs`, `pages/needs.rs`, the gallery's `samples.rs`,
  `tests.rs`, `names.txt`, and `README.md`, 3.5, *Away*, which it rewrites
  around this cut's new bullet. Whichever lands second merges: the shared
  per-project function takes ui-reaches' `ask` without `kinds`, and the
  *On All projects* bullet stays after *Its toasts*. The status entry sits
  apart from ui-reaches' edited lines.
- **ui-p4a-team** (its trunk): `pages/rail.rs` (`facts`' `members`, a line
  apart from `away`), `frame.rs`, `tests/pages/rail.rs`, `words.rs`, the
  gallery's lists.
- The gallery's `names.txt` and `tests.rs` meet every UI plan: regenerated,
  not hand-merged (`notes/common.md`).
