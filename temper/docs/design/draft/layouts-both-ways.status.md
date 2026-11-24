# layouts-both-ways: status

2026-11-24. Where the design lags or departs from [layouts-both-ways.md](layouts-both-ways.md).

- **Design lags:**
  - 3: `skein/docs/design/data/codec/README.md`, 5, and `temper/docs/design/draft/redeploys.md`, 3.1, move a wire's version at each layout change; `temper/docs/design/draft/redeploys.md`, 3.2, agrees a highest common version.
  - 5 and 9: `skein/docs/design/data/codec/patches.md`, `skein/docs/design/data/codec/bridges.md` and `skein/docs/design/data/codec/command.md`, 2 and 6, keep texts and patches in the repository and bridge through them.
  - 6: `temper/docs/design/draft/redeploys.md`, 4.1, has the newer side translate, through the bridges.
  - 8: `jig/docs/design/protocol/store/opening-rehearsal.md`, 1, goes back to the previous generation only; `jig/docs/design/protocol/store/layouts.md`, 1, keeps the transcripts' frames outside the registry.
  - 9: `AGENTS.md`, 2, says a wire's change brings its patch.
- **Design departs:**
  - 4.2: in Keep mode, an absent `option` that the target takes away is refused, where the table has Keep do as Read and take the target's default (`skein/docs/design/data/codec/layouts.md`, 2). Writing the default would come back present by the Read plan, so the value would not mean the same at the target, against 4.2's Keep, and 4.5's round trip would not hold. Decided by goal L1's designer, 2026-11-24, where two of the draft's lines contradict.
  - 3: a client's exchanges that wait for no hello, signing in and out, approvals and the link's asks, are judged by no layout: at another layout, one whose own records changed may be refused unnamed, where *version not served* named it before (jig's `protocol/client/versions.md`, 1 and 2). Decided by goal L4's designer, 2026-11-24: the draft compares layouts at the hello, which those exchanges don't wait for, and a record whose declarations are unchanged decodes alike at either layout; reads, the one other exchange that left before the hello, now wait for it.
  - 4.4: the store keeps its derivation, the Read mode with every part a value could be refused at refused as the plan's, so its plans and refusals are unchanged and a value type whose Read plan would refuse a value still needs its owner's written job (`skein/docs/design/data/codec/layouts.md`, 2 and 7). Whether a store opens with plans that refuse per value is L2's to decide, beside 8.1. Decided by goal L1's designer, 2026-11-24, since L1 is in `skein-codec` alone and its acceptance asks the store's plans to derive unchanged.
- **Open:** none.
