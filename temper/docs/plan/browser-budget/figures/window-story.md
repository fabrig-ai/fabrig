# window-story: a three-window history read at both ends

2026-11-06. Before commit `2db059a9c0`; resumed baseline at `c95e92fe74`; after `c95e92fe74` plus this task’s patch. 16 CPUs, AMD EPYC-Genoa Processor.

Each measurement selects exactly one named story with `tools/tiers/run --cpu DIR temper browser -- -E FILTER`. Loads are the one-, five- and fifteen-minute averages. The original before figures were committed by the first run before its source edits; they are preserved below. The resumed run finishes Latest’s baseline after the history-ctl repair, before changing the script allowance.

The arms were not alternated: the spec requires the before arm before editing, and this run resumes an already edited branch. The after machine was much less loaded. These figures do not isolate the window’s CPU effect from load, the merged ctl repair, or the stronger traversal at both ends.

- **long:** `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`.
- **latest:** `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`.
- **earlier:** `earlier_words_stay_put_without_browser_scroll_anchoring`.
- **short:** `a_short_chat_opens_whole_without_a_reading_status`.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before | 1 | 27.033063 | 19.446622 | 10.32 / 17.50 / 25.04 | 9.72 / 16.43 / 23.25 | passed |
| long | before | 2 | 15.117861 | 18.489539 | 18.71 / 17.78 / 22.93 | 19.71 / 17.99 / 22.77 | passed |
| long | before | 3 | 16.582325 | 19.442755 | 34.10 / 22.05 / 23.84 | 29.70 / 22.79 / 24.02 | passed |
| latest | before | 1 | 10.067301 | 22.517626 | 9.72 / 16.43 / 23.25 | 10.98 / 15.73 / 22.72 | failed: incoming ctl exceeded ten-second exit deadline; `before-latest-1.log` |
| latest | before | 2 | — | — | — | — | not run: no unchanged rerun after failure |
| latest | before | 3 | — | — | — | — | not run: no unchanged rerun after failure |
| earlier | before | 1 | 8.479710 | 4.782042 | 14.63 / 16.29 / 22.79 | 24.69 / 18.69 / 23.35 | passed |
| earlier | before | 2 | 7.066986 | 5.776701 | 19.71 / 17.99 / 22.77 | 23.54 / 19.10 / 22.99 | passed |
| earlier | before | 3 | 6.500664 | 4.708744 | 29.70 / 22.79 / 24.02 | 28.58 / 23.14 / 24.10 | passed |
| short | before | 1 | 5.720408 | 2.416851 | 24.69 / 18.69 / 23.35 | 18.71 / 17.78 / 22.93 | passed |
| short | before | 2 | 4.220289 | 2.999640 | 23.54 / 19.10 / 22.99 | 34.10 / 22.05 / 23.84 | passed |
| short | before | 3 | 4.020143 | 2.480827 | 28.58 / 23.14 / 24.10 | 29.47 / 23.78 / 24.28 | passed |
| latest | before, resumed | 1 | 9.205289 | 13.384946 | 10.67 / 11.43 / 13.70 | 48.18 / 33.68 / 22.48 | passed |
| latest | before, resumed | 2 | 9.625863 | 5.020786 | 37.51 / 32.03 / 22.12 | 27.88 / 30.15 / 21.76 | passed |
| latest | before, resumed | 3 | 8.762424 | 4.748509 | 27.88 / 30.15 / 21.76 | 20.65 / 28.29 / 21.37 | passed |
| long | after | 1 | 19.389239 | 12.064276 | 0.32 / 2.02 / 5.91 | 0.99 / 1.99 / 5.75 | passed |
| long | after | 2 | 18.384365 | 11.606071 | 2.65 / 2.26 / 5.35 | 3.00 / 2.42 / 5.29 | passed |
| long | after | 3 | 19.642271 | 12.567477 | 3.02 / 2.50 / 5.10 | 3.07 / 2.57 / 5.03 | passed |
| latest | after | 1 | 9.382707 | 5.120804 | 0.83 / 1.75 / 5.43 | 2.14 / 2.00 / 5.42 | passed |
| latest | after | 2 | 9.380930 | 5.109279 | 2.59 / 2.37 / 5.19 | 3.10 / 2.48 / 5.17 | passed |
| latest | after | 3 | 8.976088 | 4.899997 | 2.21 / 2.41 / 4.91 | 1.86 / 2.31 / 4.82 | passed |
| earlier | after | 1 | 8.124029 | 3.873855 | 0.77 / 1.89 / 5.66 | 0.85 / 1.82 / 5.53 | passed |
| earlier | after | 2 | 8.646379 | 4.124818 | 3.00 / 2.42 / 5.29 | 2.73 / 2.40 / 5.22 | passed |
| earlier | after | 3 | 8.213927 | 4.086366 | 3.07 / 2.57 / 5.03 | 2.40 / 2.45 / 4.94 | passed |
| short | after | 1 | 5.622880 | 2.354153 | 2.14 / 2.00 / 5.42 | 3.70 / 2.41 / 5.46 | passed |
| short | after | 2 | 5.761806 | 2.402641 | 3.10 / 2.48 / 5.17 | 3.02 / 2.50 / 5.10 | passed |
| short | after | 3 | 5.798653 | 2.364767 | 1.71 / 2.28 / 4.80 | 1.83 / 2.26 / 4.74 | passed |

## Medians

| Story | Before CPU s | Before wall s | After CPU s | After wall s | Target CPU s |
|---|---:|---:|---:|---:|---:|
| long | 16.582325 | 19.442755 | 19.389239 | 12.064276 | about 12 |
| latest | 9.205289 | 5.020786 | 9.380930 | 5.109279 | about 6 |
| earlier | 7.066986 | 4.782042 | 8.213927 | 4.086366 | about 5 |
| short | 4.220289 | 2.480827 | 5.761806 | 2.364767 | — |

The three named CPU targets are unmet. Every final run passed without starvation, leaks or incomplete accounting. Latest’s original failed baseline remains in the table; the resumed baseline has three passes. The original deadline failure was 10.000069098 s against ten seconds, with 0.065891473 s thread runnable wait, 0.119036546 s thread CPU and 0.007407645 s tree runnable wait; stderr was empty.

The four CPU medians sum to 42.745902 s; this is not a whole-tier measurement. It leaves 42.254098 s for the other thirteen against 85, or 27.254098 s against the target of 70. Their earlier 45–70 s range would put that estimate at 87.745902–112.745902 s. The remaining levers and the merged account belong to the other tasks; after-landing runs the whole tier.

## Fixture and proof

The 64 KiB alternative hits the existing minimum: `temper/crates/temper-client-domain/src/limits.rs` refuses a window below `long_text`, and `temper/crates/temper-client-shell-web/src/limits.rs` sets that to 131,072. The answer therefore authorizes the fallback used here: keep the 131,072-byte window and raise `temper-fake::SCRIPT_BYTES` from 262,144 to 524,288. The one-answer allowance stays 65,536.

Eight distinct replies use 10,000 repetitions of “data ” each. Seven have forty paragraphs and are 52,614 bytes; the last has thirty and is 51,974: **420,272 bytes**, above three windows (393,216), with every reply below the answer allowance. No other message split was tried. Each cue names its numbered message, and each reply has its own “End of reply N.”.

Thirty paragraphs in the final reply make opening read its predecessor. Forty in the other long replies leave room at both read margins while a position is held. The trials below show why the shorter middle replies and equal final height failed. The unanchored story keeps its original thirty/thirty/ten paragraph heights and 7,000 code words: **109,642 bytes**, inside one window.

The original earlier-position and held later-position measurements remain. Four keys per traversal read run while the deployment is stopped, then the next numbered marker is awaited after resuming: an answering read cannot consume the rest of a key burst and skip that marker. The first reply and the first message arrive on separate activity pages, so reaching the start takes the additional read. The return from the end starts at the retained penultimate message.

The long story sees the newest reply at opening; reads the first message and sees the newest gone; reads forward to the last reply and sees the first message gone; reads the first message back, sees the newest gone again, and restores it with Latest. Focus, loading text, both position checks and the final end-in-log check remain. Latest/new incoming words/sending and the short story keep their existing assertions.

In the sampled final long run, the log is y=59, height=629; opening end y=627, height=19; initial head y=-2536. Earlier stays y=105, height=24; later goes y=563 to y=564, height=19 (within the one-pixel allowance); Latest returns the end to y=627, height=19.

The unanchored run has log y=59, height=629, initial head y=-2131, and its earlier measurement remains y=105, height=24 before and after. Its thirty-paragraph replies still span those measurements.

The reply search over the other thirteen stories finds no other `with_replies` calls. The call-line story has two small read_docs calls and its short answer; the hold and chat help bodies total 13,000 bytes. None of the other thirteen histories can fill this conversation window. They were not run.

## Long story CPU split

The first final long run was sampled from its process tree’s `/proc/PID/stat`, user plus system ticks, nominally every 10 ms. Command lines and executable names identify families; every spawning thread’s child list is followed. No process environment was read. Short-lived work and final ticks between samples can be missed.

| Family | Observed CPU s |
|---|---:|
| Chromium renderer | 6.65 |
| Chromium other | 3.00 |
| temper serving | 1.91 |
| temper ctl | 0.36 |
| other children | 3.40 |
| test binary | 2.51 |

The split accounts for **17.83 of 19.389239 CPU s (92.0%)**; 1.559239 s is unassigned. Chromium’s two groups use 9.65 observed CPU s. The other children include setup, fakes and agents; their observed pkill share is 0.64 s. This family split does not separate layout, measurement and accessibility queries inside the renderer. Those are the remaining measured costs, rather than evidence that the twelve-second target was reached.

## Trials retained

These diagnostic runs precede the final fixture. Earlier sampler diagnostics are not used as the final CPU split.

| Trial | CPU s | Wall s | Load before | Load after | Result / log |
|---|---:|---:|---|---|---|
| first-run distinct replies | 0.882255 | 0.486894 | 12.16 / 19.67 / 22.80 | 12.01 / 18.89 / 22.44 | startup refused the 408,112-byte script at the old joint allowance; development-long-1.log |
| after-long-1 | 13.095110 | 7.134600 | 6.20 / 22.12 / 19.75 | 4.11 / 19.57 / 18.99 | ten middle paragraphs; earlier measurement anchor evicted; after-long-1.log |
| development-long-taller-1 | 10.986025 | 24.444500 | 0.29 / 11.28 / 15.90 | 1.99 / 10.25 / 15.33 | thirty middle paragraphs, six keys; reply 5 unreached; development-long-taller-1.log |
| development-long-keys-1 | 19.158013 | 24.476079 | 2.38 / 7.36 / 13.64 | 3.01 / 6.88 / 13.18 | ten keys; first message unreached; development-long-keys-1.log |
| development-long-keys-2 | 11.794071 | 6.738141 | 0.71 / 4.86 / 11.74 | 1.06 / 4.54 / 11.41 | sixteen keys; temporary measured reply 6 skipped; development-long-keys-2.log |
| development-long-held-1 | 19.157300 | 40.283695 | 0.40 / 3.51 / 10.48 | 1.07 / 3.16 / 9.92 | held traversal, thirty paragraphs; loading absent; four leaked descendants on failure; development-long-held-1.log |
| development-long-margins-1 | 10.791137 | 6.007341 | 0.12 / 1.80 / 8.27 | 0.49 / 1.76 / 8.08 | forty everywhere; opening did not need the preceding reply; development-long-margins-1.log |
| development-long-margins-2 | 17.896384 | 18.213601 | 1.95 / 1.89 / 7.75 | 2.76 / 2.14 / 7.59 | forty except final thirty; first reply arrived without first message; development-long-margins-2.log |
| development-long-first-page-1 | 16.880900 | 12.094112 | 2.87 / 2.26 / 7.32 | 9.98 / 3.97 / 7.70 | passed with the separate first-message read, sixteen keys and extra busy checks; development-long-first-page-1.log |
| after-long-final-1 | 20.599749 | 13.196806 | 2.34 / 3.14 / 6.89 | 2.09 / 3.00 / 6.72 | passed with eight keys and extra busy checks; after-long-final-1.log |
| mutation-long-history-1 | 0.012421 | 0.012747 | 2.82 / 3.29 / 7.12 | 3.27 / 3.36 / 7.04 | expected failure after halving code words: 220,272 < 393,216; mutation-long-history-1.log |

The baseline failure rows are carried from the first run’s committed figures; their original logs were not re-read in this invocation. Resumed baseline, trial and final logs and machine-readable CPU records are retained beside this run’s report. The new fake admission test was also checked by restoring the old joint allowance: it failed with “scripts past their joint byte bound”, then passed with 524,288 restored. The standalone and demo refusal rows still reject 544,000 or 560,000 text bytes plus wrappers. The focused check exposed the demo rows’ old smaller payloads as newly valid; those two oracle literals were increased without changing their refusal assertions.

The temporary sampler and target/browser-budget directory are removed before push.
