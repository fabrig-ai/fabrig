# progress-value: status

2026-11-04. Where the design lags or departs from [progress-value.md](progress-value.md).

- **Design lags:**
  - 2, 3: `../ui.md`, 3.2: progress has no property; its text alone says how far.
  - 4: `../ui-web.md`, section 6: a progress bar carries `aria-valuetext` alone, no `aria-value*` numbers and no `data-progress`; section 12: `skein-ui.css` sets no `--skein-ui-progress`.
  - 6: `../ui.md`, 16.2 and 16.3: a progress maps to its platform's view with no value.
  - 9: `temper/docs/design/draft/fetch-progress.md`, section 6: temper's meter rule reads its own `--temper-flow-meter-fill`, which no node sets.
- **Open:**
  - 10: the width's channel, `data-progress` with `skein-ui.css`'s rules, or a custom property set through the CSSOM.
