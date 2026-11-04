# fetch-progress: status

2026-11-04. Where the design lags or departs from [fetch-progress.md](fetch-progress.md).

- **Design lags:**
  - 2: `../protocol/setup.md`, 6.6: the fetch says nothing as it streams, and keeps no `Content-Length` bound.
  - 3: `../protocol/setup.md`, 7.1: the pipe has no `fetched` line.
  - 4: `../protocol/forge.md`, 2.8: the box's link keeps no progress, and the forge's row's setting up carries its step alone.
  - 5: `../client/forge.md`, 5.4: "Setting up: fetching" and the step list's first step name no percentage. `../client/README.md`, 3.5: the Model step's progress node is textless, named by the connection, and the terminal has no bar. `../client/browser.md`, 2.5: the moving bar has no figure beside it. The look request `welcome-fetch-percentage` left to this work the status line naming the binary, the percentage in words beside the moving bar, the progress node's name ("Fetching Forgejo"), text and `aria-valuetext`, "Each step"'s first step, and the bytes with no size, over the moving bar. `temper-flow-meter`'s token and rules stand, and no node carries the token.
  - 6: skein's draft of a progress value is not written. The bar's fill waits on it; the rule's `--temper-flow-meter-fill` is 0% unless given, and the moving bar stays.
- **Open:** none.
