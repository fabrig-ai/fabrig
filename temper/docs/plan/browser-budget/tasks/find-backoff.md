+++
id = "find-backoff"
part = "finds"
kind = "feature"
needs = ["window-story"]
files = [
  "skein/testing/skein-fake-person-browser/src/face.rs",
  "skein/testing/skein-fake-person-browser/src/tests.rs",
  "skein/testing/skein-fake-person-browser/src/press_tests.rs",
  "skein/docs/design/browser.md",
  "temper/docs/plan/browser-budget/figures/find-backoff.md",
]
effort = "high"
minutes = 60
+++

# A find that keeps missing looks less often, if that pays

## Goal

The request's third lever. An earlier reading had 519 accessibility queries in 23 s on one story. The request's patch backs off repeated misses of one find, and it's unmeasured. Measure it on the stories as the first two levers left them, and **land it only if it pays**. It must not make a press or a real change slower to notice. A result of "it doesn't pay" is a good outcome of this task: only the figures land then.

Acceptance, whichever way it goes:

- `temper/docs/plan/browser-budget/figures/find-backoff.md` holds three alternated pairs of each of the four stories, A the trunk and B the patch, as `notes/measuring.md` says, the long story's accessibility query count in each arm, and the decision by the rule below.

And only if it lands:

- `BrowserFace` carries the request's patch and the rule below, with the request's two tests and the two below, and each bites.
- `skein/docs/design/browser.md`, section 6, says so, replacing "Repeated semantic finds also wait for that interval, whether the target was present or absent." with what's true: see *Owns*.

**The decision.** The patch lands if both hold:

1. **It pays:** on the long story, B's median CPU is at most 90% of A's, and B is the lower in at least two of the three pairs.
2. **It isn't slower to notice:** on each of the four stories, B's median wall is at most 105% of A's.

Otherwise no code lands. Either way, your report gives the numbers. Don't tune the thresholds or the patch's figures to pass.

## Inputs

- `temper/docs/plan/browser-budget/inputs/request.md`, the third lever and *The patch*.
- `skein/docs/design/browser.md`, 6.
- `skein/docs/design/ui-web.md`, 14.2.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.

## Owns and may change

Owns the files in the header. A departure from the implementer's text: if the patch lands, you replace that one sentence of `skein/docs/design/browser.md`, section 6, with this one, and change no other design text:

> Repeated semantic finds also wait for that interval, whether the target was present or absent, and one that keeps missing waits twice as long each time, up to four intervals. A press's own re-find, a find's success and any act of the person's bring it back to the one interval.

Readers of the face: `jig-client-fake-person-browser`, `skein-browser-world`, `skein-ui-web-tests`, `jig-client-browser-world` and `temper-client-browser-world`. Nothing in them should need to change. Their slow-tier stories are compiled, not run, but for the four: a change in how often a find looks is a change of meaning, and after-landing runs them.

**The rule the patch needs beside its own.** The patch keeps a press at its poll, resets on a find's success and treats a different find afresh. It leaves a same find's back-off standing across a person's act: `gone(latest())`, some PageUps, then `see(latest())` would start backed off. So any act of the person's (a press, a key, typed words, a navigation) clears `missing`, and brings a `look_again` that's further off than one poll back to one poll. Find the acts' choke point in `face.rs` yourself; `self.acting` and `Request::Key` are where to start.

Options, each decided:

- **The patch's context** was written at `767e88ce19`. Apply it by hand where the file has moved, keeping its names (`missing`, `delay`) and its figures: 50, 100, 200, then 200 ms.
- **Switching arms:** a throwaway way to choose an arm without rebuilding (an environment variable read in a local, uncommitted change) is fine, and is gone before the push. The arms then differ by a branch only, which is what you're measuring.
- **A find's cost has another cause:** one missing `Find` asks the kit once per accessible role of its target before it counts as absent (`search.role += 1`). If your query count shows that dominating, say so with numbers in your report. Don't change it here.

## Steps

1. The throwaway arm switch and the counter. Three alternated pairs of each story, with the patch as the request has it plus the rule above.
2. The decision, written in the figures.
3. **If it doesn't land:** remove everything but the figures, commit and push, and report `Done:` saying so.
4. **If it lands:** the four tests in `src/tests.rs` and `src/press_tests.rs`:
   - `repeated_missing_finds_back_off_without_delaying_a_different_find` and `finding_a_target_resets_its_missing_find_backoff`, as the request has them;
   - `a_press_looks_again_each_poll_however_long_it_misses`: a press whose re-find keeps missing asks again at each 50 ms, never later;
   - `an_act_brings_a_backed_off_find_back_to_its_poll`: a find backed off to 200 ms, then a key, is asked again within 50 ms.

   Break each rule once by hand and see its test fail. The design sentence. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-fake-person-browser --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-fake-person-browser -p jig-client-fake-person-browser`, and `heavy cargo check --tests -p skein-browser-world -p skein-ui-web-tests -p jig-client-browser-world -p temper-client-browser-world`. `python3 -I -B tools/citations/citations.py --base origin/main`. Make sure the throwaway code is gone, delete `target/browser-budget/`, commit and push.

## Report

Landed or not, by the rule, with each arm's medians and load and each query count; the rules you broke to see the tests bite, if it landed; and your implementation and check minutes apart.

## Answers

2026-11-06 (temper/ui). Q: resume after `window-story` repairs the long story's script admission, or widen this task to repair that fixture? A: resume after `window-story`; this task doesn't widen, and `browser_pages.rs`, `temper-fake` and the bundle's window stay out of it. The blocker is `window-story`'s own, its answer settles it (a 64 KiB window, about 192 KiB of replies, inside the fake's joint bound), and its second run is carrying that out; measuring any other fixture would measure stories that aren't the ones landing.

fab may start this run before `window-story` merges, on what it has pushed. So, before any build or measurement, `git fetch origin` and merge `origin/browser-budget/window-story`, then read `temper/docs/plan/browser-budget/figures/window-story.md`: the repair is in when its after arm shows passing runs of all four stories, in place of `unobserved: joint script bound blocks after arm` (or when that branch is an ancestor of `origin/browser-budget/integration`). If it isn't in, fetch again every five minutes for up to thirty, building and measuring nothing. If it's still not in, stop with `Question: window-story's repair isn't pushed; resume after it`, changing nothing. Don't run the long story to find out.

Once it's in, take all 24 comparison runs on that base, and merge again before pushing; if the merge changes `browser_pages.rs`, the tier's bundle or `face.rs`, measure again. In `figures/find-backoff.md`, keep the failed `A/1` row of `7485300d7b` as a run of the old base, counted in no median, and replace the rest of what's written there with the comparison and the decision by the rule above. The thresholds, the patch's figures and the *Steps* stay as they are.

## Rebase (2026-11-06, step merge, foreman browser-budget/find-backoff/foreman-2)

reconcile browser.status.md with integration’s public-history status and resolved window gap
The merge conflicts only in `temper/docs/design/client/browser.status.md`.
Merge the current integration into the task branch and resolve that file by preserving:
- Main’s public-history limits paragraph and section 8.3 **Design departs** entry.
- Main’s updated section 8.4 entry naming `public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` and its pending runtime check.
- The removal of section 4.3’s obsolete **Not yet: The tier’s window** entry.

Integration commits `aeba68c39b` and `0268dac74c` already reconcile these entries; the latter’s file is the intended resolution against the refused tip.
Keep `abcf3e0159`’s figures and decision that backoff does not pay; introduce no backoff code.
This status reconciliation needs no new measurements. If merging also changes `browser_pages.rs`, the tier’s bundle or `face.rs`, follow the spec’s remeasurement requirement.
