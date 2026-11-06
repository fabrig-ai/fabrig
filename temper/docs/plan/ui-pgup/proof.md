Holds: every driven step at 1440 × 900 and 420 × 900; no input lost in 240 fast trials.

# PgUp on the fixed code

The measured product is `4bb2e3f25c0d5b109d8a3a79d0f9f70cae9ee216`, on `ui-pgup/proof`, descended from the task base `3740fc2f8b4ef3eaafa7671481178a0ceace5089`. Chromium reports `Chrome/147.0.7727.101`. The before column is [reproduced.md](reproduced.md), measured at `301458e0f3517dfbbaa64e0e4c562a68f6e79a1c`. The final whole run took 467.50 s, from 2026-11-06T19:44:37.857144+00:00 to 2026-11-06T19:52:25.354248+00:00.

## Method and notation

The native binary, loopback Wasm client and bundler were built separately through `heavy`, with `TEMPER_BUNDLE=none`, then bundled by the non-Cargo lines of `temper/scripts/bundle.sh`. Each width and pace used a fresh fake box, six seed exchanges and the same supported standalone scripted model peer as the baseline: 300 ms between slow pieces, 20 ms in the races. CDP device metrics set the exact viewport, and all typing, caret movement, keys, wheel events and pointer presses used browser input. The paragraph sampler and its movement, missing-frame and race criteria are the baseline's. The driver's change adds snapshots immediately before and after each screenshot to its saved observations.

Each cell is **baseline ⇒ fixed**. Within a cell, `a→b` is a step's start→end. Heights, offsets and moved are CSS pixels. `TA` is `TEXTAREA`; `D` is the exact draft **“Keep these draft words intact.”**, with caret **25**. Sending consumes the draft and leaves caret 0. Intentional scrolling and following move the paragraph; a reading hold requires final movement 0, maximum sampled movement 0 and no sampled disappearance.

## 1440 × 900: slow steps and focus

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1. PgUp from following | 2880→2174 ⇒ 2880→2174 | 3610→3610 ⇒ 3610→3610 | 3610 ⇒ 3610 | 730 ⇒ 730 | 706 ⇒ 706 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Hold across call + next line | 2174→3824 ⇒ 2174→2174 | 3610→4554 ⇒ 3610→4554 | 1057 ⇒ 3610 | 730 ⇒ 730 | -1650 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 1. PgUp again | 3824→3118 ⇒ 2174→1468 | 4554→4554 ⇒ 4554→4554 | 4554 ⇒ 4554 | 730 ⇒ 730 | 706 ⇒ 706 | N→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. PgDn until foot | 3118→3992 ⇒ 1468→3992 | 4554→4722 ⇒ 4554→4722 | 4554 ⇒ 4554 | 730 ⇒ 730 | -874 ⇒ -2524 | Y→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Follow incoming words | 3992→4496 ⇒ 3992→4496 | 4722→5226 ⇒ 4722→5226 | 4722 ⇒ 4722 | 730 ⇒ 730 | -504 ⇒ -504 | N→N ⇒ N→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Pointer Latest after PgUp ×2 | 3084→4664 ⇒ 3084→4664 | 5394→5394 ⇒ 5394→5394 | 5394 ⇒ 5394 | 730 ⇒ 730 | -1580 ⇒ -1580 | Y→N ⇒ Y→N | TA→BODY ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails (focus) ⇒ **holds** |
| 1/wheel. Wheel up (−600) | 8495→7895 ⇒ 8495→7895 | 9225→9225 ⇒ 9225→9225 | 9225 ⇒ 9225 | 730 ⇒ 730 | 600 ⇒ 600 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Hold across call + next line | 7895→9439 ⇒ 7895→7895 | 9225→10169 ⇒ 9225→10169 | 6672 ⇒ 9225 | 730 ⇒ 730 | -1544 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 1/wheel. Wheel up again | 9439→8839 ⇒ 7895→7295 | 10169→10169 ⇒ 10169→10169 | 10169 ⇒ 10169 | 730 ⇒ 730 | 600 ⇒ 600 | N→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. PgDn until foot | 8839→9607 ⇒ 7295→9607 | 10169→10337 ⇒ 10169→10337 | 10169 ⇒ 10169 | 730 ⇒ 730 | -768 ⇒ -2312 | Y→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Follow incoming words | 9607→10111 ⇒ 9607→10111 | 10337→10841 ⇒ 10337→10841 | 10337 ⇒ 10337 | 730 ⇒ 730 | -504 ⇒ -504 | N→N ⇒ N→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Pointer Latest after wheel ×2 | 8911→10279 ⇒ 8911→10279 | 11009→11009 ⇒ 11009→11009 | 11009 ⇒ 11009 | 730 ⇒ 730 | -1368 ⇒ -1368 | Y→N ⇒ Y→N | TA→BODY ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails (focus) ⇒ **holds** |
| 2. PgUp while writing | 13942→13236 ⇒ 13942→13236 | 14672→14672 ⇒ 14672→14672 | 14672 ⇒ 14672 | 730 ⇒ 730 | 706 ⇒ 706 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 2. Hold across ending | 13236→14278 ⇒ 13236→13236 | 14672→15008 ⇒ 14672→15008 | 12422 ⇒ 14672 | 730 ⇒ 730 | -1042 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 2. Post-end state | 14278→14278 ⇒ 13236→13236 | 15008→15008 ⇒ 15008→15008 | 15008 ⇒ 15008 | 730 ⇒ 730 | 0 ⇒ 0 | N→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 2. Send a message | 14278→14387 ⇒ 13236→14387 | 15008→15117 ⇒ 15008→15117 | 15008 ⇒ 15008 | 730 ⇒ 730 | -109 ⇒ -1151 | N→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→empty ⇒ D→empty | 25→0 ⇒ 25→0 | holds ⇒ **holds** |
| 2. PgUp at mark removal | 17210→16504 ⇒ 17210→16504 | 17940→17940 ⇒ 17940→17940 | 17940 ⇒ 17940 | 730 ⇒ 730 | 706 ⇒ 706 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 2. PgUp about 200 ms later | 16504→15798 ⇒ 16504→15798 | 17940→17940 ⇒ 17940→17940 | 17940 ⇒ 17940 | 730 ⇒ 730 | 706 ⇒ 706 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |

- `1/key/hold-call-next-line`: maximum sampled movement 1650 ⇒ 0 px; paragraph absent 3/87 ⇒ 0/85 frames; fixed minimum height 3610, from initial height 3610.
- `1/wheel/hold-call-next-line`: maximum sampled movement 1544 ⇒ 0 px; paragraph absent 2/84 ⇒ 0/86 frames; fixed minimum height 9225, from initial height 9225.
- `2/hold-across-end`: maximum sampled movement 1042 ⇒ 0 px; paragraph absent 2/46 ⇒ 0/45 frames; fixed minimum height 14672, from initial height 14672.

PgUp moves 730 − 24 = 706 px. The boundary key entered 2.6 ms after observed writing-mark removal; the later key began 203.4 ms after it. The fixed ending hold retains its paragraph and pill throughout; sending still goes to the foot.

## 420 × 900: slow steps and focus

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1. PgUp from following | 5119→4420 ⇒ 5119→4420 | 5842→5842 ⇒ 5842→5842 | 5842 ⇒ 5842 | 723 ⇒ 723 | 699 ⇒ 699 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Hold across call + next line | 4420→6700 ⇒ 4420→4420 | 5842→7423 ⇒ 5842→7423 | 1376 ⇒ 5842 | 723 ⇒ 723 | -2280 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 1. PgUp again | 6700→6001 ⇒ 4420→3721 | 7423→7423 ⇒ 7423→7423 | 7423 ⇒ 7423 | 723 ⇒ 723 | 699 ⇒ 699 | N→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. PgDn until foot | 6001→6996 ⇒ 3721→6996 | 7423→7719 ⇒ 7423→7719 | 7423 ⇒ 7423 | 723 ⇒ 723 | -995 ⇒ -3275 | Y→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Follow incoming words | 6996→7882 ⇒ 6996→7882 | 7719→8605 ⇒ 7719→8605 | 7719 ⇒ 7719 | 723 ⇒ 723 | -886 ⇒ -886 | N→N ⇒ N→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1. Pointer Latest after PgUp ×2 | 6484→8178 ⇒ 6484→8178 | 8605→8901 ⇒ 8901→8901 | 8605 ⇒ 8901 | 723 ⇒ 723 | -1694 ⇒ -1694 | Y→N ⇒ Y→N | TA→BODY ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails (focus) ⇒ **holds** |
| 1/wheel. Wheel up (−600) | 14814→14214 ⇒ 14814→14214 | 15537→15537 ⇒ 15537→15537 | 15537 ⇒ 15537 | 723 ⇒ 723 | 600 ⇒ 600 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Hold across call + next line | 14214→16395 ⇒ 14214→14214 | 15537→17118 ⇒ 15537→17118 | 11071 ⇒ 15537 | 723 ⇒ 723 | -2181 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 1/wheel. Wheel up again | 16395→15795 ⇒ 14214→13614 | 17118→17118 ⇒ 17118→17118 | 17118 ⇒ 17118 | 723 ⇒ 723 | 600 ⇒ 600 | N→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. PgDn until foot | 15795→16690 ⇒ 13614→16690 | 17118→17413 ⇒ 17118→17413 | 17118 ⇒ 17118 | 723 ⇒ 723 | -895 ⇒ -3076 | Y→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Follow incoming words | 16690→17577 ⇒ 16690→17577 | 17413→18300 ⇒ 17413→18300 | 17413 ⇒ 17413 | 723 ⇒ 723 | -887 ⇒ -887 | N→N ⇒ N→N | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1/wheel. Pointer Latest after wheel ×2 | 16377→17872 ⇒ 16672→18168 | 18595→18595 ⇒ 18595→18891 | 18595 ⇒ 18595 | 723 ⇒ 723 | -1495 ⇒ -1496 | Y→N ⇒ Y→N | TA→BODY ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails (focus) ⇒ **holds** |
| 2. PgUp while writing | 24213→23514 ⇒ 24213→23514 | 24936→24936 ⇒ 24936→24936 | 24936 ⇒ 24936 | 723 ⇒ 723 | 699 ⇒ 699 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 2. Hold across ending | 23514→24804 ⇒ 23514→23514 | 24936→25527 ⇒ 24936→25527 | 21028 ⇒ 24936 | 723 ⇒ 723 | -1290 ⇒ 0 | Y→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 2. Post-end state | 24804→24804 ⇒ 23514→23514 | 25527→25527 ⇒ 25527→25527 | 25527 ⇒ 25527 | 723 ⇒ 723 | 0 ⇒ 0 | N→N ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | fails ⇒ **holds** |
| 2. Send a message | 24804→24913 ⇒ 23514→24913 | 25527→25636 ⇒ 25527→25636 | 25527 ⇒ 25527 | 723 ⇒ 723 | -109 ⇒ -1399 | N→N ⇒ Y→N | TA→TA ⇒ TA→TA | D→empty ⇒ D→empty | 25→0 ⇒ 25→0 | holds ⇒ **holds** |
| 2. PgUp at mark removal | 29776→29077 ⇒ 29776→29077 | 30499→30499 ⇒ 30499→30499 | 30499 ⇒ 30499 | 723 ⇒ 723 | 699 ⇒ 699 | N→Y ⇒ N→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 2. PgUp about 200 ms later | 29077→28378 ⇒ 29077→28378 | 30499→30499 ⇒ 30499→30499 | 30499 ⇒ 30499 | 723 ⇒ 723 | 699 ⇒ 699 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |

- `1/key/hold-call-next-line`: maximum sampled movement 2280 ⇒ 0 px; paragraph absent 2/88 ⇒ 0/87 frames; fixed minimum height 5842, from initial height 5842.
- `1/wheel/hold-call-next-line`: maximum sampled movement 2181 ⇒ 0 px; paragraph absent 2/88 ⇒ 0/87 frames; fixed minimum height 15537, from initial height 15537.
- `2/hold-across-end`: maximum sampled movement 1290 ⇒ 0 px; paragraph absent 2/46 ⇒ 0/45 frames; fixed minimum height 24936, from initial height 24936.

PgUp moves 723 − 24 = 699 px. The boundary key entered 3.0 ms after observed writing-mark removal; the later key began 203.2 ms after it. The fixed ending hold retains its paragraph and pill throughout; sending still goes to the foot.

The baseline post-end failures follow its preceding snap to the foot; they do not establish a separate pill fault. Its successful fresh-box controls repeat steps already covered above. The table below compares each retained control observation with that same step in the final whole run, whose history is larger in the ending cases.

## Baseline controls beside the final whole run

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1440, slow: `1/key/hold-call-next-line` | 2174→2174 ⇒ 2174→2174 | 3610→4554 ⇒ 3610→4554 | 3610 ⇒ 3610 | 730 ⇒ 730 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1440, slow: `1/wheel/hold-call-next-line` | 7895→7895 ⇒ 7895→7895 | 9225→10169 ⇒ 9225→10169 | 9225 ⇒ 9225 | 730 ⇒ 730 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1440, ending: `2/hold-across-end` | 2006→2006 ⇒ 13236→13236 | 3442→3778 ⇒ 14672→15008 | 3442 ⇒ 14672 | 730 ⇒ 730 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 1440, ending: `2/ended-stays-up` | 2006→2006 ⇒ 13236→13236 | 3778→3778 ⇒ 15008→15008 | 3778 ⇒ 15008 | 730 ⇒ 730 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 420, slow: `1/key/hold-call-next-line` | 4420→4420 ⇒ 4420→4420 | 5842→7423 ⇒ 5842→7423 | 5842 ⇒ 5842 | 723 ⇒ 723 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 420, ending: `2/hold-across-end` | 4124→4124 ⇒ 23514→23514 | 5546→6137 ⇒ 24936→25527 | 5546 ⇒ 24936 | 723 ⇒ 723 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |
| 420, ending: `2/ended-stays-up` | 4124→4124 ⇒ 23514→23514 | 6137→6137 ⇒ 25527→25527 | 6137 ⇒ 25527 | 723 ⇒ 723 | 0 ⇒ 0 | Y→Y ⇒ Y→Y | TA→TA ⇒ TA→TA | D→D ⇒ D→D | 25→25 ⇒ 25→25 | holds ⇒ **holds** |

## Fast race counts

Every path has 30 independent inputs, phased across the 20 ms delivery cycle. All 240 baseline and all 240 final inputs were observed while writing, had a subsequent sampled streaming delivery, and kept TA/D/25. The baseline definitions are retained: lost up means the later view is at the foot or never away; lost down means reaching the former foot without resuming following, confirmed by both the next delivery and the later sample. Reached former foot and reached current foot remain separate counts; a trial that never reaches the former foot is not counted as a following success.

| Width | Path | Presses | Lost: reproduced ⇒ task base ⇒ fixed | Reached former foot: before ⇒ fixed | Current foot: before ⇒ fixed | Never reached former foot: before ⇒ fixed | Focus failures: before ⇒ fixed | Result |
|---:|---|---:|---:|---:|---:|---:|---:|---|
| 1440 | key-up | 30 | 0 ⇒ 0 ⇒ 0 | — | — | — | 0 ⇒ 0 | **holds** |
| 1440 | key-down | 30 | 0 ⇒ 0 ⇒ 0 | 30 ⇒ 30 | 30 ⇒ 30 | 0 ⇒ 0 | 0 ⇒ 0 | **holds** |
| 1440 | wheel-up | 30 | 0 ⇒ 0 ⇒ 0 | — | — | — | 0 ⇒ 0 | **holds** |
| 1440 | wheel-down | 30 | 6 ⇒ 4 ⇒ 0 | 29 ⇒ 30 | 23 ⇒ 30 | 1 ⇒ 0 | 0 ⇒ 0 | **holds** |
| 420 | key-up | 30 | 0 ⇒ 0 ⇒ 0 | — | — | — | 0 ⇒ 0 | **holds** |
| 420 | key-down | 30 | 0 ⇒ 0 ⇒ 0 | 30 ⇒ 30 | 30 ⇒ 30 | 0 ⇒ 0 | 0 ⇒ 0 | **holds** |
| 420 | wheel-up | 30 | 1 ⇒ 0 ⇒ 0 | — | — | — | 0 ⇒ 0 | **holds** |
| 420 | wheel-down | 30 | 10 ⇒ 4 ⇒ 0 | 27 ⇒ 30 | 17 ⇒ 30 | 3 ⇒ 0 | 0 ⇒ 0 | **holds** |

Every final downward key and wheel trial reached the **current** foot: 30/30 on each path and width. There are no unobserved deliveries, unreached downward trials or lost presses hidden by the count.

The following ranges retain the geometry behind the counts. Every cell is baseline ⇒ fixed; moved includes the intentional input and following. Pill is the count of final Y/N observations. Focus and words/caret are TA and D/25 throughout both runs.

| Width/path | scrollTop before range | scrollTop after range | scrollHeight before range | minimum height range | clientHeight | moved range | pill after Y/N |
|---|---:|---:|---:|---:|---:|---:|---|
| 1440/key-up | 462–35306 ⇒ 462–35306 | 0–34600 ⇒ 0–34600 | 1192–36036 ⇒ 1192–36036 | 1192–36036 ⇒ 1192–36036 | 730–730 ⇒ 730–730 | 462–706 ⇒ 462–706 | 30/0 ⇒ 30/0 |
| 1440/key-down | 262–35106 ⇒ 262–35106 | 1427–36270 ⇒ 792–35966 | 1192–36036 ⇒ 1192–36036 | 1192–36036 ⇒ 1192–36036 | 730–730 ⇒ 730–730 | -1165–-270 ⇒ -1164–-530 | 0/30 ⇒ 0/30 |
| 1440/wheel-up | 462–35306 ⇒ 462–35306 | 0–34706 ⇒ 0–34706 | 1192–36036 ⇒ 1192–36036 | 1192–36036 ⇒ 1192–36036 | 730–730 ⇒ 730–730 | 0–600 ⇒ 462–600 | 30/0 ⇒ 30/0 |
| 1440/wheel-down | 262–35106 ⇒ 262–35106 | 792–35306 ⇒ 792–35702 | 1192–36036 ⇒ 1192–36036 | 1192–36036 ⇒ 1192–36036 | 730–730 ⇒ 730–730 | -965–0 ⇒ -926–-530 | 7/23 ⇒ 0/30 |
| 420/key-up | 916–49810 ⇒ 916–49810 | 217–49111 ⇒ 217–49111 | 1639–50533 ⇒ 1639–50533 | 1639–50533 ⇒ 1639–50533 | 723–723 ⇒ 723–723 | 699–699 ⇒ 699–699 | 30/0 ⇒ 30/0 |
| 420/key-down | 716–49610 ⇒ 716–49610 | 1373–50267 ⇒ 1373–50725 | 1639–50533 ⇒ 1639–50533 | 1639–50533 ⇒ 1639–50533 | 723–723 ⇒ 723–723 | -1521–-245 ⇒ -1206–-657 | 0/30 ⇒ 0/30 |
| 420/wheel-up | 916–49810 ⇒ 916–49810 | 316–49210 ⇒ 316–49210 | 1639–50533 ⇒ 1639–50533 | 1639–50533 ⇒ 1639–50533 | 723–723 ⇒ 723–723 | -864–600 ⇒ 600–600 | 29/1 ⇒ 30/0 |
| 420/wheel-down | 716–49610 ⇒ 716–49610 | 1373–49810 ⇒ 1465–50725 | 1639–50533 ⇒ 1639–50533 | 1639–50533 ⇒ 1639–50533 | 723–723 ⇒ 723–723 | -1521–-70 ⇒ -1206–-657 | 13/17 ⇒ 0/30 |

## Holding while words arrive

Both PNGs are viewport captures, with “Latest ↓” visible, writing still active, and the composer retaining D/25. The interval numbers span the hold through the call and next-line paragraph 003, before the screenshot. The separately recorded capture bracket identifies the picture's geometry; the growing height over the hold proves that words arrived below the held view.

| Screenshot | Held scrollTop before→after | scrollHeight before→after | Growth | clientHeight | moved / maximum sampled moved | pill / focus / words / caret | Capture bracket: top; height |
|---|---:|---:|---:|---:|---:|---|---|
| [1440 × 900](proof/holding-1440.png) | 2174→2174 | 3610→4554 | +944 | 730 | 0 / 0 | Y→Y / TA→TA / D→D / 25→25 | 2174→2174; 4554→4554 |
| [420 × 900](proof/holding-420.png) | 4420→4420 | 5842→7423 | +1581 | 723 | 0 / 0 | Y→Y / TA→TA / D→D / 25→25 | 4420→4420; 7423→7423 |

## What this task repaired

The initial whole run on the task base still lost the desktop wheel call hold and the phone key call and ending holds. Fast downward wheel inputs lost following in 4/30 trials at each width. Those observations are retained alongside the final evidence.

**Retain the thread's prefix across suffix snapshots and gaps.** Numeric diagnosis at `jig_client::thread::Thread::run` showed the same attempt/turn/retry replacing thousands of retained bytes with an empty or final suffix. A later diagnostic caught the gapped case: the thread held bytes 0–8155, then received only bytes 8184–8696, a 512-byte cut tail. Its prefix disappeared before the committed page arrived. The first contiguous-only repair was insufficient; the retained intermediate phone ending still moved −1290 px.

`jig/crates/jig-client/src/thread.rs` and `thread/model.rs` now append contiguous same-retry suffixes, using `LiveTurn.from` for the retained prefix's origin and `offset` for the newest stream end. A gap leaves that prefix in place with its cut mark, without joining missing bytes. Full-origin recovery, a newer retry and a new attempt retain their replacement paths. The committed page replaces provisional text. This implements the prefix guarantee in `jig/docs/design/client/domain/client.md`, section 10, without changing the lower stream buffer's cut/recovery rules in section 8.3.

The two new focused tests in `jig-client` cover empty/contiguous/overlapping suffixes, UTF-8 bounds and a gap. Both were made to fail by restoring snapshot replacement. A separate mutation replaced the prefix only on a gap: the bound-16/from-6 case failed with `z` instead of `aé`. Restoring the rule passed. The 29 thread tests and 10 conversation tests passed together.

**Oracle repair outside the production-fix file list.** The request's *For every task of this plan* explicitly requires repairing a test oracle that correct behaviour trips. Two existing tests in `jig/crates/jig-client-view/src/tests/state/domain/conversations.rs` had asserted lower stream-buffer replacement through the conversation's retained prefix. They now assert the cut tail, suffix deduplication, stale-delivery rejection and reopened-origin behaviour on `client.text`, and separately assert the thread's retained prefix. Each previously failing test passed alone after the repair; the remaining retry and recovery assertions stay in place.

**Make vertical wheel movement count in the same browser turn.** The task base still allowed a native wheel to reach a stale foot while new text arrived before following was restored. `skein/crates/skein-ui-web/src/page/web.rs::Mounted::wheel` now applies and measures unmodified vertical wheel movement over a log synchronously, normalizing pixel, line and page deltas. It shares the computed line height with page keys. Nested scrolling controls/panes, components and modified or horizontal input keep their platform handling. The `WheelEvent` feature is enabled in that crate's manifest. This follows `skein/docs/design/ui-web.md`, section 7. The native scheduling failure is observable only in Chromium; under the task's explicit exception the driver is this fix's proof. No browser story was added.

The inherited pointer-focus fix holds at both widths: Latest reaches the foot with TA/D/25. PageUp and wheel now agree on holding; PageDown and downward wheel input resume following. A turn's ending leaves the reading paragraph at 0 px and never collapses the sampled height. Sending still returns to the foot.

## Checks and evidence

`cargo fmt` and the final formatting check passed. Old-form checks passed in jig, skein and temper. Native clippy passed for `jig-client`, `jig-client-view` and `skein-ui-web` with all targets and warnings denied; Wasm clippy passed for `skein-ui-web`. All 48 changed/direct-consumer packages compiled with tests, including the existing slow-tier stories. Their focused default suite passed **9,237 tests across 73 binaries**, with 158 tests skipped. No browser, process or live tier ran. The citation check against `origin/main` found **0 new broken citations, 0 in all**. The package list and mutation checks are in [checks.json](proof/checks.json).

The required `ui-pgup/ending-keeps` branch was fetched and merged again and was already present. `skein/docs/design/ui-web.status.md` and `jig/docs/design/client/domain/client.status.md` carry no **Not yet** for this plan's page-key following, pointer focus or retained live text. The stale `turn_bytes` departure in `client.status.md` was also removed: a sliding or cut lower stream window now leaves the thread's prefix in place, as the suffix and gap tests show. The numeric diagnostic instrumentation was removed before the final build. The driver, report data and both images were reviewed for machine paths, machine addresses and private values.

- Final observations: [1440 slow](proof/1440-slow.json.gz), [1440 fast](proof/1440-fast.json.gz), [420 slow](proof/420-slow.json.gz), [420 fast](proof/420-fast.json.gz).
- Initial task-base observations: [1440 slow](proof/before-repair-1440-slow.json.gz), [1440 fast](proof/before-repair-1440-fast.json.gz), [420 slow](proof/before-repair-420-slow.json.gz), [420 fast](proof/before-repair-420-fast.json.gz).
- Diagnosis: [prefix replacement](proof/prefix-diagnostic.json.gz), [512-byte tail and its offsets](proof/origin-diagnostic.json.gz), [intermediate ending failure](proof/before-gap-repair-420-slow.json.gz).
- Provenance: [browser](proof/browser.json), [whole-run timing](proof/timing.json), [manifest](proof/manifest.json), [driver](driver/pgup.py).

## Left

None.
