# testing: status

2026-11-24. Where the tree's code lags or departs from [testing.md](testing.md).


- **Not yet:** 2, 3, 6: the tier holds twelve test functions. Its merged
  tests 2 (with 11 and the call line), 4 (with 5, the model picker and 15),
  8 and 9 (welcome then ConnectModel, with Accounts history and key
  re-entry), 14 (short, Latest, unanchored and long), and 16 (sheet scroll
  and phone geometry) retain story labels for failed steps. Tests 2 and 14
  have passing runs on record in Chromium, each run alone (2's in 6, above;
  14's with its failures, below), from before the slow-tests-down draft
  reshaped them (`temper/docs/design/draft/slow-tests-down.md`, 5.1); none
  proves a merged story as it now stands. The picker's 420 px, one-press
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
- **Not yet:** 3: a story's settings are still committed by sittings of
  processes, each starting a box, its serve and three or four ctl
  processes, not hosted in the test's own process. [in flight: change 18126]
- **Not yet:** 3: no supplied manual proof checks a long chat at the person's
  1,048,576-byte bound. The passing long browser story spans three small
  windows, while the unanchored position history stays within one; neither
  substitutes for that whole-bound check (`README.md`, section 3).
- **Not yet:** 6: the request `browser-trim` has a person's page slower to settle as
  the answer most wanted. Settled chat and task pages stood still and no product cause
  was found, but a chat closed on the box took 8.539 s to reach the Closed filter, a
  wait that plan located and did not split, and test 2 has since lost the step (`README.md`, section 3
  ).
