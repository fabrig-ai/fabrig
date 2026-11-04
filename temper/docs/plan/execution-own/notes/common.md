# execution-own: common rules

Every task of this plan names this note. From `inputs/request.md`, which the user approved.

## What this plan is

Wording and deletion of documents. fabrig keeps each milestone's content and acceptance, and the
design choices with their reasons; whatever says who ran the work, and where, leaves.

## Departures from the implementer's text

- **Drafts and design files are yours here.** The text's "Not yours: any `docs/design/`, drafts"
  does not hold for the files your spec owns: changing them is the task. It still holds for every
  other file under `docs/design/`, and for `docs/help/`.
- **No Rust changes, so no `cargo fmt` and no clippy.** The finishing checks are your spec's.

## Never touched

- `temper/docs/development/`: three of its files stay in `tests/boundary/outward.txt`, and that is
  right for this plan.
- Any `docs/plan/` directory, code, tests, and `tests/boundary/tests/outward.rs`.
- A section number: surviving sections are never renumbered, a removed section leaves its number
  unused (`DESIGN.md`, 4). So `notes.md` goes 1, 4, 5, 6, with no note saying why.
- The N numbers, including the gaps: there is no N18 and no N24, and "N1 to N24" stays as written.

## The words that go, and those that stay

**Go:** `fab`, its loop and pipeline; trunks; a plan as a unit of running ("a plan of its own",
`temper/docs/plan/...`, "the plan's retire"); calls, leads and sessions as who does the work;
hosts and paths on a machine; the lab's role names (`fabrig/qa`, `fabrig/dogfood`,
`agents/loop`); "in place of ..." comparisons; the lab's implementers and their run numbers.

**Stay, they are the product's or the reference's own:** goal, G, g, L, coordinator, designer,
producer, foreman, the cut and the close-out (`temper/docs/design/draft/reference-code-production.md`,
sections 5 and 12), a plan of changes as a coordinator makes it, batch, change, tier, QA, walk,
rung and ladder (`temper/docs/design/draft/v0.1.0.md`), main's queue, writer slots, host slots, a
program's session, both plans (Claude's and ChatGPT's), the user's dated decisions
("the user, 2026-11-02").

Where a line holds both, cut the lab's words and keep the rest. Where nothing is left, delete it.

## The draft's voice

Short sentences that say what is true now: no history ("originally", "used to"), no emphasis
words, no "formerly" or "removed" notes, no em dashes. Keep each file's line width and its
bullet form (`- **Lead:** text`). Your spec's replacement text binds in content. Smooth its wording
to fit the sentence around it if you need to, and add no claim of your own.

## The counted names

The check's names are in `tests/boundary/tests/outward.rs`. To read them in a file:

```sh
grep -nEi "\bfab\b|fabrig-ai/agents|agents repo|fabrig/(qa|dogfood|debt)|agents/(loop|status|creds|ops)|hel8|hetzner[0-9]|debian-16gb|status\.fabrig\.ai|~/src/(fabrig|rust/tmp)/|/home/free/|heavy (cargo|env|--)|FAB_HOME" FILE
```

## The checks

`citations.py` reads committed trees: commit first.

```sh
python3 -I -B tools/citations/citations.py --all          # 0 broken
python3 -I -B tools/citations/citations.py --base origin/main
python3 -I -B tools/citations/test_citations.py
python3 -I -B temper/scripts/old_forms.py
heavy cargo nextest run -p fabrig-boundary --test outward
```

Check the count: the `outward` run selects that binary's tests, above zero.
