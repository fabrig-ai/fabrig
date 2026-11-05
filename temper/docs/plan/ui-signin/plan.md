# ui-signin: the plan

The fake ChatGPT sign-in fails in a browser, so no recording reaches past Ready. The request's diagnosis, read from the code and not yet run: the fake issuer's connection cap of 2. This plan proves that in a walk, fixes it in the lowest tier, makes a rejected connection visible, and proves the whole in Chromium.

## Parts and tasks

| Part, task | Tree | Needs | Effort, minutes | What it does |
| --- | --- | --- | --- | --- |
| `peer-rejected` | skein | | medium, 20 | `oauth::Peer::rejected()` counts connections turned away at the cap |
| `issuer-connections` | temper | `peer-rejected` | high, 90 | Red walk, `CONNECTIONS = 8`, the stderr line, the Chromium walk to Home, one status line |

## Critical path

`peer-rejected`, then `issuer-connections`: about 110 implementer-minutes. The Chromium walk is the long end of the second task.

## Decisions

- **Two tasks, because a branch stays in its tree.** The peer's count is skein's and its reader is temper's. The fix's only wait is that small first task.
- **The red walk opens its sockets at the authorize hop, not "just before the visit" as the request says.** The native visitor closes each hop, and at cap 2 two sockets held before the visit would reject the visit itself, so it would panic rather than end `Failed Later`. Chromium's sockets exist after the 302. The walk makes the hops on plain sockets, with the authorize hop's own socket as one of the two, which leaves the order to the peer's accept queue, with no race. The request's stop stands: if the walk passes at cap 2, the diagnosis is wrong and the task ends in a `Question:`.
- **A public interface grows:** `skein_fake_peers::oauth::Peer::rejected()`. A count, because the issuer runs `without_records` and no record reaches temper. The peer stays free of io. Only the OAuth peer has it: nothing reads the others.
- **`PLAN_SLOTS` stays 2.** Its old comment tied it to the two connections; it now says it bounds token plans. With no plan queued the issuer answers `Refused`.
- **One design line:** `journeys.status.md`, 9 loses its unknown cause, in `issuer-connections`, on a passing Chromium walk. The Not yet entry stays, since Home is not recorded. `journeys/project-landing/README.md` still describes the blocked walk: that and the recording are `ui-welcome`'s.
- **No stored format or protocol changes.**

## Overlaps

- `temper/docs/design/client/journeys.status.md`: `look-round-4/account-onboarding` and `look-round-4/record-landing` (merged on that plan's trunk) also touch it. It is one paragraph, edited late in the second task.
- `temper/crates/temper-fake/src/backends.rs` belongs to `claude-always/always-offered`; this plan only reads it.
