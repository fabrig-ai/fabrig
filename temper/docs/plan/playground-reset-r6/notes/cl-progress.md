# cl's progress (temper's client)

Specs under `temper/docs/plan/playground-reset-r6/tasks/`; notes `notes/cl-wire.md`.

| Spec | State | Needs |
|---|---|---|
| `cl-wire` | final | (none) |
| `cl-ctl` | final once request 4 is answered | `cl-wire` |
| `cl-choices` | final | `cl-wire` |
| `cl-forges` | final | `cl-choices` |
| `cl-welcome` | final | `cl-forges` |

## Contract requests

1. **Accepted (lead).** **T8's version move lands with the journeys, not in `cl-wire`.** Every forge ask (watch,
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
2. **Accepted (lead).** **The wire's reading of the engine's new answers** (for sv's family translation):
   `notes/cl-wire.md`, 3. T8 names the new outcomes but not each engine answer's; the one
   T8 leaves open is `RemoveRefusal::NoSuchForge`, read as the existing
   `connect_refused { why: no_such_forge }`. The `PlaygroundAnswer`'s `project` and
   `forge` are 0 while no `Playground` row exists.
3. **A Forgejo elsewhere made by `make_forge` lists with `admin: true`** (fe's projection,
   sv's settings). T4 makes it "mode Managed, no setup entry"; today's `admin` (the row's
   offer of the admin connect, `Configured.admin_connect`) is true only for a managed setup
   entry with no admin token (`temper-service/src/startup/application.rs`, `configured`).
   `cl-choices` opens the made forge's Details, which offer "Connect as its admin" only for
   a row with `admin`; without it, a Forgejo elsewhere made from the client cannot be
   connected. **Request:** the record of `ForgeChoice::Forgejo` carries a managed setup
   entry with no admin token (the machine's default bot), so it lists `admin: true` and the
   admin connect's setup host has its settings. Designed around as it stands: the client
   opens the row's Details whatever they offer.
4. **`cl-ctl` regenerates two help pages.** `temper ctl`'s reference pages are generated
   from the tools' table and checked by the `temper` package's
   `every_ctl_group_page_is_what_the_table_renders`, so new ctl tools cannot land without
   `docs/help/reference/ctl-forge.md` and `ctl-playground.md` rewritten by
   `SKEIN_CODEGEN_WRITE=1`. **Request:** allow `cl-ctl` that generated rewrite (no hand
   edit); the tools' words are in its spec, step 1. `cl-ctl` is final once this is
   answered.
5. **Reprinting a journey's pages.** A client task that changes a page a journey prints
   (words, a banner) leaves the gate's journeys test red unless it reprints
   (`temper/docs/design/client/journeys.md`, 1: "a landing that changes a recorded page
   carries its new print"). The client specs allow `heavy cargo run -p
   temper-client-journeys` and committing the reprinted `NN.html` pages under
   `temper/docs/design/client/journeys/`, never a recording; a stale journey stops the
   task. **Request:** confirm this edit of `docs/design/` (generated output) for cl's
   tasks; `cl-pg` needs it for certain (the playground's chats page, `welcome-playground`'s
   `02.html`, carries the banner).
6. **`welcome-set-up` will need a person at the end.** On a fresh box with no forge
   record, the welcome's "Run on this machine" is `MakeAndSetUpAtOnce`, not today's
   `SetUpAtOnce { forge: 3 }`: the walk's act changes, so `again` cannot record it
   (journeys.md, 8: "walked anew in a browser ... then added"). The client tasks keep the
   committed recordings valid until then (a card targets a kept record nothing has begun
   on, as the recording's forge 3), and `cl-journeys` records `welcome-playground` again
   with `again`. **Question for the user, through the lead:** at `cl-journeys`, (a) the
   user walks `welcome-set-up` anew on a fresh `temper box --fake-backends` and adds it
   (`add welcome-set-up FILE`), or (b) `cl-journeys` deletes `welcome-set-up` and it is
   walked after the run. Recommendation: (a), the user's walk being three clicks; `cl-journeys`
   then waits for the file.
7. **The Forges document says nothing of the machine's offers.** With zero records the
   client cannot tell whether Forgejo on this machine is offered (today a row's `offer`).
   Designed around: the card is drawn offered and a `not_offered` answer says so on it.
   **Later** (no change now): a member of `Forges` with the deployment's offers for each
   kind.

## Help changes needed (the lead's)

- `docs/help/forge.md`, `forgejo.md`, `github.md`, `playground.md`, `ctl.md`: `forge make`,
  `forge remove`, `playground reset`; the playground's banner "set up temper"; Remove on
  Forges and what it leaves on the forge; Start over and Go on on the welcome; a Forgejo
  elsewhere added from Connect a forge, no longer from the configuration.
- `docs/help/reference/ctl-forge.md`, `ctl-playground.md`: generated by `cl-ctl`
  (request 4).
