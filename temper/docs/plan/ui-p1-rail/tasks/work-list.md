+++
id = "work-list"
part = "work"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/work.rs", "temper/crates/temper-client-domain/src/tests/pages/work.rs"]
effort = "high"
minutes = 110
+++

# Work's one list: what goes where, one pure function

## Goal

Place a project's goals, changes, chats and drafts on Work's one list by their own
facts, in one pure function of the client's domain, `pages::work::place(&Facts,
wall) -> Work`, as `temper/docs/design/client/README.md`, 3.5, *Work*, *What goes
where*, has it and `contracts.md`'s C4 makes exact. It reads plain facts alone, no
domain, no client and no settings, so that a fuzz test (`fuzz`) and the page
(`work-page`) build on it.

Acceptance:

1. `temper_client_domain::pages::work` holds C4's types and `place`, exactly, the
   module's doc comment citing `temper/docs/design/client/README.md, 3.5`.
2. `place` keeps every rule of C4: In flight's three kinds of card and which change
   goes on which card; its order and ties; Drafts; Landed's seven days, its bounds
   included; the four filters' counts; a repeated task number dropped.
3. A table test holds a row for each rule, and one row of a whole small project
   after `temper/docs/design/client/web/whole-story/work.html` (two live goals, one
   with a change checking and one landed, a change of its own, a chat with its
   change, two drafts, landed rows inside and outside the week), its whole `Work`
   compared.
4. `temper-client-domain`'s focused tests pass, clippy clean.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`, and `contracts.md`, C4.
- `temper/docs/design/client/README.md`, 3.5, *Work*: *All*, *What goes where*,
  *A goal's card*, *A chat's card*; *Changes*, *Where a change came from*.
- The look, for the whole row: `temper/docs/design/client/web/whole-story/work.html`.

## Owns and may change

Owns: the new `temper-client-domain/src/pages/work.rs` and
`src/tests/pages/work.rs`.

Neighbour files, one line each: `pages.rs`, `pub mod work;` in its module list,
in order; `src/tests/pages.rs`, `mod work;` after `mod welcome;`.

Nothing else: `work::facts`, which gathers `Facts` from the domain's pages, is
`work-page`'s. No match on `Address` here.

Pre-approved options:

- **If `skein_lib::List` lacks a sort that keeps the strict subset's rules:** sort
  by insertion, comparing `(activity, task)` as C4 orders them; the lists are
  bounded by the client's row limits. Say which you used.
- **If seven days isn't a `skein_lib::Duration` constant:** make one, named for
  what it bounds (`LANDED_THIS_WEEK`), beside `place`.
- **If C4's `Default` derives meet clippy's lints on `List`:** write the `Default`
  by hand; the shape stays.

## Steps

1. **The types,** as C4, each with a one-line doc comment in the design's words.
2. **`place`:** drop repeats per kind, first kept; the live goals' cards; for each
   queued change, the live goal it belongs to, else the listed chat it belongs to
   (that chat gets its card), else a card of its own; each goal's and chat's card
   then takes its landed changes, newest first, after its changes on their way by
   place; In flight sorted newest activity first, ties by the lower task; the
   drafts' cards; Landed, `wall - 7 days <= at <= wall`, newest first; the filters'
   counts. All arithmetic checked (`checked_sub`, `saturating_*`), no indexing.
3. **Tests,** `tests/pages/work.rs`: one table of rows, each its `Facts`, its `wall`
   and the expected `Work` or the part of it the row is about: a goal live and one
   ended; a queued change of a live goal, of an ended goal, of a listed chat, of an
   unlisted chat, of nothing; a chat with only landed changes (not in In flight, but
   counted by `chats`); a draft whose chat has a change on its way (in both
   sections); order and a tie; Landed at `wall`, at `wall - 7 days`, a second
   earlier, and after `wall`; a repeated task; and the whole project of acceptance
   3. Break two rules by hand, one at a time (a change of a live goal also given a
   card of its own; the week's lower bound off by one), see their rows fail, and
   restore.
4. **Check** as `notes/common.md` says. Commit and push: the message says that
   Work places each item by its own facts, never by the project's settings.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw, the sort you used, and the test's CPU time.
