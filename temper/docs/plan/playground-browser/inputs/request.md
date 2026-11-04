# playground-browser: the playground in the browser, part three

At the user's request (11-03), finish the playground in the browser: `temper/docs/design/playground.md`, the parts left after `playground-skein` (skein's kit) and `playground-temper` (temper's composition and native world) landed.

## The scope

From `temper/docs/design/playground.status.md`, and playground-temper's cut answer (11-03, its option (a)):
- **Story A walked natively through the web client's host,** with the recorded log replayed natively under the one-process world's referee (section 9's native part). This uses skein's visitor, in-memory bridge `Platform` and session log, now on main. The cut sketched it: a driver in `tests/playground` over `temper_client_shell_web::host::Host`, the visitor joining the composition's processes, then a story-A log recorded and replayed. It builds on `temper_playground::Process<H>` and `stories::A`.
- **6, the page:** `temper-client-shell-web`'s feature `playground`, with skein's worker platform for the page, the starter, the frame and the failure page.
- **7, served:** the served files and their headers, `scripts/bundle.sh --playground`, and the `playground` profile.
- **9's browser part:** `temper-playground`'s wasm32 build in the gate, and the browser tier's walk of story A.
- **10, its bar for v0.1.0:**
  - story A walks in the browser, from the welcome to the CSV change landed and the other two requests answered, in the browser tier and by hand on Firefox and Safari on a desktop;
  - within its bars once the user sets them (section 8);
  - a failure offers its log, which replays natively.
  
  Publishing at the site's address goes with the release, not here.
- **`temper-client-shell-web` passing wasm32 clippy,** so that it joins fab's gate list `.config/gate/wasm32.txt`, along with `temper-playground`. Today it fails on `clippy::too_many_lines` at `src/lib.rs:131` (`turn`, 102 of 100 lines). Split `turn` rather than allowing it (`AGENTS.md` 2).

## Context

- **The design is complete**, except what playground-temper's or playground-skein's cut recorded. Its designers write only where a cited section is incomplete (`DESIGN.md`).
- **The bar's figures are the user's** (section 8): propose them, measured, as Open.
- **Budgets and timeouts are the user's** (skein's `docs/foundation/testing-strategy.md`, 8.1). Desktop browsers only; phones are Later.

## Size

One cut is at most 90 minutes. Release early: the native walk through the host first, then the page and the served files, then the browser walk.

## Walk

Story A in the browser through the browser tier, and by hand on Firefox and Safari (section 10), is the slice to mark `walk = true`.
