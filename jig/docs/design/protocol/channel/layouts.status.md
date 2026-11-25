# layouts: status

2026-11-25. Where jig's code lags or departs from [layouts.md](layouts.md).

- **Not yet:** 2.1 and 4.1: the engine converts a kept part's `Turn` body
  but writes its `KeptPart` envelope at the engine's layout. A worker
  whose envelope adds a field with a default or narrows the body bound
  can pass the opening and receive an assignment it cannot decode.
- **Not yet:** 3: the live opening refuses the worker's kinds' table as
  *limits* when a responder kind's largest frame exceeds the channel's
  output capacity; judging a kept opening does not check that the table
  can be installed and can approve the same opening.
- **Not yet:** 4.1 and 5.1: the engine fits a start to the room at its own
  layout before converting it, and fits nothing again after: a start
  that would fit at the worker's layout once cut is refused as invalid,
  and one that grows past the room is refused without the cuts that
  would fit it.
- **Not yet:** 5.3: a nested Read refusal closes the channel as an
  undecodable body. For application parts, a failure of their own decoder
  is instead counted and the event forwarded without its parts, leaving
  the channel open.
- **Not yet:** 8: the opening tests do not check the *too deep* refusal
  or its text, whether deriving the plan or fitting its workspace.
- **Not yet:** 8: the wire tests do not check the `absent` refusal and
  its text when Keep cannot carry an absent option.
- **Not yet:** 5.4 and 8: `Site`, `Gap` and `gaps`, and their tests. [in flight: change 17717]

