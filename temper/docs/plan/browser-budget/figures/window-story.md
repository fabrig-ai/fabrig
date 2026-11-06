# window-story: partial window measurements

2026-11-06, before commit `2db059a9c0`; 16 CPUs, AMD EPYC-Genoa Processor.

The before arm precedes every source edit, as the task requires; the arms were not alternated. Runs selected exactly one story through `heavy python3 -I -B tools/tiers/run --cpu DIR temper browser -- -E FILTER`. Loads below are the one-, five- and fifteen-minute averages.

The after arm is blocked before the long story opens: its uniquely scripted reply text totals 408,112 bytes, while `temper-fake::SCRIPT_BYTES` is 262,144 bytes (`temper/crates/temper-fake/src/lib.rs`). The joint accounting also includes script wrappers. `application.demo.llm.otherwise: scripts past their joint byte bound` is the observed startup refusal. The task owns neither this bound nor its checks.

The baseline Latest story also hits the known incoming ctl deadline while the page is open. It is not rerun unchanged. Its repair belongs to the pending `history-ctl` dependency.

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
| long | after | 1 | — | — | — | — | unobserved: joint script bound blocks after arm |
| long | after | 2 | — | — | — | — | unobserved: joint script bound blocks after arm |
| long | after | 3 | — | — | — | — | unobserved: joint script bound blocks after arm |
| latest | after | 1 | — | — | — | — | unobserved: joint script bound blocks after arm |
| latest | after | 2 | — | — | — | — | unobserved: joint script bound blocks after arm |
| latest | after | 3 | — | — | — | — | unobserved: joint script bound blocks after arm |
| earlier | after | 1 | — | — | — | — | unobserved: joint script bound blocks after arm |
| earlier | after | 2 | — | — | — | — | unobserved: joint script bound blocks after arm |
| earlier | after | 3 | — | — | — | — | unobserved: joint script bound blocks after arm |
| short | after | 1 | — | — | — | — | unobserved: joint script bound blocks after arm |
| short | after | 2 | — | — | — | — | unobserved: joint script bound blocks after arm |
| short | after | 3 | — | — | — | — | unobserved: joint script bound blocks after arm |

**Before medians (three passing runs):**

- long: 16.582325 CPU s, 19.442755 wall s.
- earlier: 7.066986 CPU s, 4.782042 wall s.
- short: 4.220289 CPU s, 2.480827 wall s.

Latest has one failed baseline and no passing median. No after median or CPU split is available. The targets cannot be assessed from this partial run.

## Development run

| Trial | CPU s | Wall s | Load before | Load after | Result |
|---|---:|---:|---|---|---|
| long, eight unique replies | 0.882255 | 0.486894 | 12.16 / 19.67 / 22.80 | 12.01 / 18.89 / 22.44 | failed before story steps: joint script bound; `development-long-1.log` |

The proposed split uses six 50,694-byte replies and two 51,974-byte replies: 408,112 bytes, above three 131,072-byte windows (393,216). Every reply is below the fake’s 65,536-byte answer bound. Thirty position paragraphs and ten middle paragraphs remain. The unanchored fixture uses replies of 35,694, 36,974 and 36,974 bytes: 109,642 bytes, inside one window.

The baseline ctl failure reports 10.000069098 seconds elapsed for a ten-second allowance, 0.065891473 seconds thread runnable wait, 0.119036546 seconds thread CPU, and 0.007407645 seconds tree runnable wait; command stderr is empty. It is not reported as starvation.

Failure logs and the machine-readable measurements are retained beside the task report under the basenames above. `target/browser-budget/` is deleted before push. The proposed later-read anchor, eviction assertion, and shorter key bursts have not been reached in Chromium. The design status entry stays until the implementation passes.
