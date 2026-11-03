# playground-temper: temper's playground world, natively

At the user's request (11-03), carry out the playground in the browser, part two: temper's composition and world, walked natively first. This is `temper/docs/design/playground.md` ("The playground in the browser"), sections 3, 4, 5, 8 and 9's native part.

## The scope

From `temper/docs/design/playground.status.md`:
- **3, the crates:**
  - `temper-playground`, the composition moved out of `tests/one-process`, on product crates alone, its wasm32 build proving it;
  - `temper-playground-world` (`tests/playground`);
  - `temper-fake-git-hosted`, moved from `testing/` to `crates/`.
- **4, the world:** the one-process world's composition without its harness. No box, calm at every layer, one seed, the printed link to standard output, the clients' listener at the document's address.
- **5, stories as data,** the fakes' latencies as their pace; the first story is A (5.2).
- **8, the documents' bounds,** measured.
- **9's native part:** story A walked natively through the client's host, and a log replayed natively. The one-process world composes `temper-playground` with its harness, so the gate runs the world the browser runs.

**Out of scope, for a later plan once skein's kit is in:** the page, starter and frame (6), the served files, bundle and `playground` profile (7), the wasm32 builds in the gate and the browser tier's walk (9's browser part), and the bar (10).

The reset without the box (section 4 of the status) depends on the fake forge's removal without the box (`domain/forge.md`, 4.1). Take it if it now holds, after `first-run-projects` and `playground-decisions` landed; otherwise leave it named.

## Context

- **The draft** (`temper/docs/design/draft/browser-playground.md`, deleted on 11-02 because the design carries it whole) is readable in git at `13e7c2a649^`.
- **v0.1.0's place:** `v0.1.0.md`, section 5, "the playground in the browser: built beside the ladder; its first story is rung A's".
- **Beside it:** plan `playground-skein` builds skein's kit. The reference case's M10 and configuring P2's build run too; leave their files alone.

The design exists; its designers write only where a cited section is incomplete (`DESIGN.md`). A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

One cut is at most 90 minutes. Release early. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

Story A walked natively through `temper ctl` is the slice to mark `walk = true`: the playground's story, the same as rung A's.
