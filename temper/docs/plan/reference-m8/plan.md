# reference-m8: plan

M8 of `temper/docs/design/draft/reference-case.execution.md`: main's tiers, the verdict, and
a change living to cleared (`temper/docs/design/domain/after-landing.md`, sections 2 to 6,
with what of 10 and 11 they need), skein's reads, jig's procedure funding. Out: section 7
(reds to goals, `repairs`) and a goal branch's tier (M9), sections 8 and 9, findings.

## Design written by this cut

- `temper/docs/design/domain/after-landing.md`, sections 1, 2, 3, 5.3, 6, 11, 12: a tier
  names the branch it follows (`branch`), the repository's landing branch when it names
  none; a landing on a branch no tier follows is cleared as it lands (N5).
- `temper/docs/design/protocol/configuration.md`, section 3: a tier's `branch`, and the
  tiers' project settings `tier_retries` and `verification_bound` in the `landing` member;
  `temper/docs/design/domain/landing.md`, 13.1 to match.
- Statuses: `after-landing.status.md` and `configuration.status.md` (Not yet);
  `reference-landing.status.md` loses "6: main's tiers follow main"; the code-production
  draft's g's tier entry now names only what M9 adds.

## Old code

The verdicts line (jig `verdicts/jig-zero-seed-13`, skein `verdicts/skein-reads-3`, temper
`verdicts/2a-set-8` and `verdicts/t9-bound`) is handed to implementers as patches in
`notes/old/`, re-rooted to this repository; each spec says which to read and what of it to
take. The plan's retire may delete `notes/old/` (git keeps it).

## Parts and tasks so far

| Part | Task | Needs | Tree |
|---|---|---|---|
| procedure-funding | jig-procedure-seed | | jig |
| tiers-core | tiers-policy | | temper (new pure crate) |
| skein-reads | skein-reads | | skein (forge kit, Forgejo dialect) |
| skein-reads | skein-fake-reports | skein-reads | skein (fake forge) |
| change-cleared | change-verifying | | temper (change crate, store's change records) |
| tiers-config | tiers-config | tiers-policy | temper (service, translate, forge config and limits) |

## Critical path, as it stands

tiers-policy and skein's reads, then the forge connector's tiers (following, runs, receipts,
the tier task), then reruns and bisect, the change's life to cleared and the bound, then the
world stories and ctl; the walked slice is the last. Later batches name it task by task.

## Decisions

- **Store families** 28 `Tier`, 29 `Verification`, 30 `LandingLog` (designed, not built in M8);
  findings move to 31 to 33: bytes 25 to 27 hold `Forge`, `ForgeNumber`, `Playground` today
  (`temper/docs/design/protocol/store.md`, 2). In place in v11, before the first release.
- **No landing log in M8:** after-landing.md 8 is out; a late verdict after the bound stays on
  the tier's record; its news to goals stays Not yet.
- **The change panel** gains `verifying`, `red`, `cleared` (`temper/docs/design/client/forge.md`,
  2.1) and `change show --until cleared` (`temper/docs/design/client/tools.md`, 5.4, 5.5), so the
  walk reads the verdict through `ctl`.
- **A change no tier follows ends at `Landed`, as today;** only a followed landing reaches
  `Cleared` or `Uncleared` (contracts.md, 2), so no existing story changes.
- **skein's inflater stays private to `skein-forge`;** no design change (forge.md, 2.1).
- **Configuration:** `verification_bound_ms` (the document's `_ms` rule), `tier_retries`; no
  `suspects` or `red_pauses` member in M8 (bisect; section 7 is M9's).

## Overlaps inside this plan

- `change-verifying` and `tiers-config` run side by side and both touch
  `temper-engine-domain-forge` (`boundary.rs`, `landing.rs`, `lib.rs`: arms against new types)
  and a few test fixtures' literals; their hunks differ.

## Overlaps with drafts beside this one

None so far: live-smith-reds-2 touches smith and one skein crate (`skein-oauth-accounts`),
none of this plan's files.
