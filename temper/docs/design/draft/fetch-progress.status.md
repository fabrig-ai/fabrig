# fetch-progress: status

2026-11-04. Where the design lags or departs from [fetch-progress.md](fetch-progress.md).

- **Design lags:**
  - 2: `../protocol/setup.md`, 6.6: the fetch says nothing as it streams, and keeps no `Content-Length` bound.
  - 3: `../protocol/setup.md`, 7.1: the pipe has no `fetched` line.
  - 4: `../protocol/forge.md`, 2.8: the box's link keeps no progress, and the forge's row's setting up carries its step alone.
  - 5: `../client/forge.md`, 5.4: "Setting up: fetching" and the step list's first step name no percentage. The look request `welcome-fetch-percentage` waits on the fetch-progress work for the status line naming the binary, the percentage in words beside the moving bar, the progress node's name, text and `aria-valuetext`, "Each step"'s first step, and the bytes with no size. `temper-flow-meter`'s token and rules are landed; no node carries the token yet.
  - 6: skein's draft of a progress value is not written. The bar's fill waits on skein's progress-value work; the rule's `--temper-flow-meter-fill` defaults to 0% until then, and the moving bar stays.
- **Open:** none.
