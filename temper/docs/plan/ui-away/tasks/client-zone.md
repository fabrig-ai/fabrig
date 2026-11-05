+++
id = "client-zone"
part = "client-zone"
kind = "feature"
needs = ["home-base"]
files = [
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/crates/temper-client-shell-web/src/host.rs",
  "temper/crates/temper-client-shell-web/src/journey.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/tests/words.rs",
]
effort = "high"
minutes = 60
+++

# The person's zone, the shell's input

## Goal

"Away until 08:00" has to read 08:00 in the browser, and every clock is universal time today: `Words::time` and `turn_clock`
divide the wall by the minute. Bring the browser's offset in as the shell's input and read 5.12's forms at it:
`temper/docs/design/client/README.md`, 5.12; `browser.md`, 2.1. A change of meaning: see the last acceptance line.

Acceptance:

- `Domain::zone_minutes()` holds what the shell gave it: the browser's `Date` offset, minutes east of universal time,
  `skein-ui-web`'s `platform.offset`; 0 in the terminal and in every test that doesn't name one.
- A journey's facts (`temper-client-shell-web/src/journey.rs`, `Facts`) record it, so a replay says the same clocks.
- *clock*, *the day's rule*, `turn_clock`, the weekday and date forms, and *ago*'s "at HH:MM" and its days read in the zone,
  both `at` and `now`. *ago*'s and *span*'s differences don't move. `Wall`'s arithmetic stays checked or saturating.
- A section that says "in universal time" (`README.md`, 5.5, 5.8, 5.9) still does: it doesn't call these forms; if one does,
  it passes zone 0 and your report names it.
- At zone 0 no golden, print or recording moves. This task runs `temper-client-view`, `temper-client-shell-terminal`,
  `temper-client-gallery` and `temper-client-journeys` whole, since it changes what `TimeForm::Clock` means.
- `README.status.md` and `browser.status.md` lose the zone's entries.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md` (`zone_minutes`).
- `temper/docs/design/client/README.md`, 5.12; `temper/docs/design/client/browser.md`, 2.1.

## Owns and may change

Owns, as `files` lists them. May also change: call sites of `Host::new` and of the domain's constructor; a new test module's
`mod` line; the shell-web fixture that builds `Facts`.

Options, each pre-approved:

- **Where the view takes it:** `Words` holds the zone, set from `root.zone_minutes()` at the build's start, so that no call of
  `time` or `turn_clock` changes; `Words::time`'s signature stays. If `Words` can't be set there, `Times` holds it and
  `turn_clock` takes it; say which.
- **The domain's input:** a field set by a new `Domain::set_zone_minutes` before the first event, or the constructor's argument,
  whichever edits fewer call sites.
- **A recording's format changed by `Facts`:** re-record, as `notes/common.md` says.

## Steps

1. `Facts` and `Host::new`: pass the offset from `platform`; the domain's field and accessor.
2. `words.rs`: the forms. Tests in `src/tests/words.rs`: `a_clock_reads_in_the_zone`, at +120 and -300;
   `the_day_turns_at_the_zones_midnight`, `ago_at_hh_mm_reads_in_the_zone`, `a_span_and_ago_differences_do_not_move`,
   `zone_zero_says_what_it_said`.
3. The check of the implementer's text. Then `heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal -p
   temper-client-shell-web -p temper-client-gallery -p temper-client-journeys`, and push.

## Report

As `notes/common.md`, Your report: where the view takes the zone, and each caller that keeps universal time.

## Rebase (2026-11-05, step merge, foreman ui-away/client-zone/foreman-1)

preserve incoming status updates and remove only the two completed zone gaps.
The merge conflicts in `temper/docs/design/client/README.status.md` and `temper/docs/design/client/browser.status.md`.
Incoming integration commit `98d3ddc0cc` adds Goal status entries beside the lines changed by `7a41679353`; `4c8072800e` also updates Settings status.
Resolve both files using the incoming status content, retaining its Goal and Settings entries.
In `README.status.md`, remove only `5.12: the zone, which every form reads as universal time.`
In `browser.status.md`, remove only `2.1: the zone, the shell's input.`
Preserve all other gaps, including the Away entries.
The log reports automatic merges for the code files; it reports no build or test failure.
After rebasing, run the spec's required package checks through heavy.
