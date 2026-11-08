+++
id = "how-fuzz"
part = "checks"
kind = "feature"
needs = ["how-parts"]
files = ["temper/tests/client/tests/suite/fuzzy/how.rs"]
effort = "high"
minutes = 75
+++

# A fuzz test of How work runs' parts and lanes against section 12's rules

## Goal

Check How work runs' one pure function of its parts and its lanes
(`temper-client-domain`'s `pages::how`, `how-parts`) against
`temper/docs/design/draft/surfaces.md`, section 12, and
`temper/docs/design/client/README.md`, 5.13, *Its parts*, over random settings and
their moves, in the client world's fuzzy suite.

Acceptance:

1. `temper/tests/client/tests/suite/fuzzy/how.rs` holds one seeded test, in the
   fuzzy module beside `public_address`, each draw replayable from its seed, as
   `public_address.rs` draws them.
2. Over random `pages::how::Facts` (each `Read`, Ask temper listed or not, Goals
   shown or hidden, each `TemplateBudget` and none), checked by an oracle written
   from 5.13 in the test, not by calling `parts` twice: a part shows exactly when
   its rule holds (Budgets while one is set, "Ask temper to change this" while
   Ask temper is listed, the heading's way while the read names a current way);
   no part is drawn empty (Budgets never without a figure).
3. Over random sequences of facts, a project's settings and their moves (a way
   used, Goals turned on or off, a template's budget changed, the read ending):
   each step's parts are its own facts' alone, whatever came before.
4. Over random paths, within the wire's bound of 24 stops, of random actors and
   bars, and over the three stock ways' paths as the engine derives them where the
   world's fixtures give them: `lanes` places every stop exactly once, in the
   path's order, a person's in You, an agent's in Agents, code's in Code, a bar in
   none; a path's move to another (a way used) places every stop of the new path,
   none of the old.
5. It runs under the fuzzy profile within its share of temper's 120 CPU seconds
   (its CPU time in the report), clippy clean.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C1.
- `temper/docs/design/draft/surfaces.md`, section 12 (the rules table, *A move
  along the spectrum*); `temper/docs/design/client/README.md`, 5.13, *The page*
  and *Its parts*; `temper/docs/design/domain/forge.md`, 8.13 (actors, bars, the
  wire's bound of a path).
- `skein/docs/foundation/testing-strategy.md`, section 1 and the fuzzy suite's
  rules it gives.
- The code: `temper-client-domain`'s `pages/how.rs` (`how-parts`'); the client
  world's `tests/suite/fuzzy/public_address.rs` (seeds, `skein_lib::Rng`, a
  replayable draw), `tests/suite/main.rs`'s `mod fuzzy`; the wire's
  `BranchPath`, `Stop`, `Actor`, `PathStep`, `Card` constructors
  (`temper-client-protocol-wire`'s generated `v1`) and the world's
  `fixtures::domain_limits()`.

## Owns and may change

Owns: `temper/tests/client/tests/suite/fuzzy/how.rs`.

Reader and neighbour files, and what may change there: `tests/suite/main.rs`, the
line `mod how;` inside `mod fuzzy`, nothing else (`ui-p1-rail` and `ui-p2-home`
add their own lines there).

Pre-approved options:

- **If the world gives no engine-derived paths** without building a world: draw
  random paths alone, keeping acceptance 4's rules, and say so.
- **If a rule of `parts` the oracle finds wrong** is `how-parts`' code: report it
  as a `Question:` with the seed and the facts, after pushing the test with that
  row marked by the seed in a comment; never change `pages/how.rs`.

## Steps

1. The oracle from 5.13 and section 12, in the test.
2. The draws and the three checks of acceptance 2 to 4, a seed per draw.
3. Break two rules by hand in a scratch copy of `how-parts`' code (Budgets shown
   for `Given`; a bar placed in Code), see the test fail with its seed, restore.
4. Check as `notes/common.md` says, running the test by name under `--profile
   fuzzy`. Commit and push: the message says How work runs' parts and lanes are
   fuzzed against section 12's rules.

## Report

Follow the implementer's text. Also give the number of draws, the test's CPU
time, and the rules you broke with the failures seen.
