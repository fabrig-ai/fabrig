# cl's progress (temper's client)

Specs under `temper/docs/plan/playground-reset-r6/tasks/`; notes `notes/cl-wire.md`.

| Spec | State | Needs |
|---|---|---|
| `cl-wire` | final | (none) |

## Contract requests

1. **T8's version move lands with the journeys, not in `cl-wire`.** Every forge ask (watch,
   read, request) carries `CLIENT_VERSION`, so moving it to 12 stales both committed
   journeys (`temper/docs/design/client/journeys.md`, 8), which the gate's
   `temper-client-journeys` test replays (`.config/gate/always.txt`): `cl-wire` could not
   land green without recording both again (`again NAME`: headless Chromium from
   `SKEIN_TEST_CHROMIUM` and a `temper box --fake-backends` built in the slot), and they
   are recorded again anyway at the end of the run. So `cl-wire` builds v12's whole shape
   at version 11 (renamed variants keep their tags, new ones appended: every byte encoded
   before is unchanged, and the journeys hold), and `cl-journeys`, the last client task,
   moves the family line and `CLIENT_VERSION` to 12 with the journeys' new recordings. The
   shapes T8 names are unchanged; no other designer's code reads the number. If the lead
   wants 12 in `cl-wire` instead: one line in the schema, `CLIENT_VERSION`, regenerated
   goldens, the two hand-made goldens `golden/connect_forgejo_admin_*.bin` and
   `src/tests/forgejo_admin.rs`'s `peer` head (`0, 11` → `0, 12`), and `again` for both
   journeys in that task, which then needs Chromium on its builder.
2. **The wire's reading of the engine's new answers** (for sv's family translation):
   `notes/cl-wire.md`, 3. T8 names the new outcomes but not each engine answer's; the one
   T8 leaves open is `RemoveRefusal::NoSuchForge`, read as the existing
   `connect_refused { why: no_such_forge }`. The `PlaygroundAnswer`'s `project` and
   `forge` are 0 while no `Playground` row exists.
