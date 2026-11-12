# testing: status

2026-11-10. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2 and 3: the tier's bundle is built with the shell's `loopback-navigation` and `small-window` features (`tools/tiers/bundles.toml`), and its addresses carry no internal setting.

- **Not yet:** 2, 3, 6: the tier holds twelve test functions. Its merged
  tests 2 (with 11 and the call line), 4 (with 5 and the model picker),
  8 and 9 (welcome then ConnectModel, with Accounts history and key
  re-entry), 14 (short, Latest, unanchored and long), and 16 (sheet scroll
  and phone geometry) retain story labels for failed steps. Tests 2 and 14
  have passing runs on record in Chromium, each run alone (2's in 6, above;
  14's with its failures, below); none proves the other current merged
  stories. The picker's 420 px, one-press
  model and effort, outside and Escape checks, a step changing and restoring anchoring on a
  live page (3), and both sign-ins in one browser (2) await after-landing
  proof (`README.md`, section 3). The picker's separate render and fake-box proof at
  `1d82a937df` establishes no passing run of the merged story. Whether
  reading's four stories fit one run's forty seconds is unmeasured: apart, their wall medians add to
  25.761 s (`temper/docs/development/browser-budget.md`), each with a setup they now
  share. Those figures read a 131,072-byte window; reading's histories are now
  sized to the tier's 32,768 (3), their code lines scaled with it and no reply,
  paragraph or step fewer, and their wall at that size is unmeasured.
- **Not yet:** 3: no whole-tier run on record establishes that the repaired
  tier fits its 127.5 CPU-second budget, 85 raised by half by the user's word
  of 11-08, for now, to be revisited. The flagged run, of 17 tests, was
  129.4 CPU s, 70.8 s wall. With ctl history and a 131,072-byte window,
  all twelve final comparison runs passed, with CPU medians of
  20.858273 s for long, 9.178371 s for Latest and 8.399148 s for unanchored,
  above the request `browser-budget`'s targets of about 12, 6 and 5, and
  6.414108 s for short, from the baseline's 4.193832 s: 44.849900 s together.
  Loads, all samples and the limits of the comparison are in
  `temper/docs/development/browser-budget.md`. The request `browser-reading`
  of 2026-11-08 cut the long story's fixture and decoding: the tier's window,
  `long_text` and `turn_text` at 32,768 bytes, reading's code lines scaled
  with them (3), and skein's decoder indexing each CDP message once (skein's
  `testing/browser.md`, 4). It reads no CPU figure, so what the four stories cost now
  is unmeasured; the final cost awaits after-landing (`README.md`, section 3). Backoff
  failed its CPU and wall criteria and is absent, and the 131,072-byte window
  alone showed no CPU saving in those figures.
  The listener-full ctl failure is repaired by ending an idle connection while
  preserving calls, partial requests and bounds. Whether that also repairs
  `browser_public`'s earlier visibility-command deadline failure is unverified:
  that story was outside the four selected for this repair.
- **Not yet:** 3: no supplied manual proof checks a long chat at the person's
  1,048,576-byte bound. The passing long browser story spans three small
  windows, while the unanchored position history stays within one; neither
  substitutes for that whole-bound check (`README.md`, section 3).
- **Not yet:** 3, 6: story 14 has failed after landing in two ways
  since `red-82-chat-opening` stopped a chat's opening reads at a screen
  and a half (`../pages/chat/conversation.md`, 2, *Opening history*), and their cause is
  unknown. Once at `0599201e87`, `long`'s words moved across a read, from
  y=519 to y=558 (`held`); in a run and its rerun at `23261749d4`,
  `unanchored`'s kept box "Position paragraph 02." was missing at step 76.
  The repair `red-87-reading` ran the story by name, alone, 30 times, and
  every run passed: ten unloaded and ten under 16 CPU busy loops at
  `2a399129fe`, ten under that load at `23261749d4`. `unanchored` held its
  anchor at its opening measurement, y=151 across its earlier read, and
  `long` held "Reading paragraph 07." and reply 8's end, its earlier pair
  at y=144 and its later from 557 to 558. Neither the product nor the
  story changed; whether they recur is for the tier's runs after landing
  to show, and a red comes back as a repair.
- **Not yet:** 6: the request `browser-trim` has a person's page slower to settle as
  the answer most wanted. Settled chat and task pages stood still and no product cause
  was found, but a chat closed on the box took 8.539 s to reach the Closed filter, a
  wait that plan located and did not split, and test 2 has since lost the step (`README.md`, section 3
  ).
