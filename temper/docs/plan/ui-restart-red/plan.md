# ui-restart-red: the plan

`ui-repo-link` left after-landing red on five process stories. The request asks all five green, with a restarted box signing its person in through the source and no token file left, in one or two tasks. It got two. `ui-repo-link-2` landed while this was cut, and this branch took it from main.

## Parts and tasks

- **template:** `spend-template`. `first_repository::run_spend` hands the unfilled init template to `startup::engine`, and `$provider` refuses.
- **handover:** `restart-handover`, needing `spend-template`. Serve reads the handover again after its first reconcile; the started cut moves to a box with no source step; the fifth story is run, read and repaired; then all five run by name.

## What was read, and what's still to prove

A call builds nothing, so this is from the code, and `restart-handover` proves it first.

1. **The template.** `run_spend` alone gives the unfilled template to `startup::engine`. The other readers, listed in `spend-template`, read other members or the filled document. Two stories die here at once, and the same two then meet cause 2.
2. **The handover: the foreman's reading holds, except that the forge does reconnect.** Serve reads `admin_token` only at its start (`setup/lifecycle.rs`, `start_pending`), and with sign-in that start now comes before preparation. The box does say `started N` after the source step, but the link drops it, no ask being out (`route/box_link.rs`, `line`). So the file stays and temper's older admin tokens stay valid. The forge still connects: the start's `Reconcile` from the kept token waits for Forgejo (skein's `forge-setup.md`, 4.4). That's why the red is at `forge_land.rs:767`, the token check, and not at the row's check above it.
3. **`startover`, read and not proved.** Its started cut stops serve, then waits for the group's `started` record; but the group's last step, `add-oauth`, fetches discovery from that stopped serve (`setup.md`, 6.8 says it does so at once), so the record never comes before the patience at line 142.
4. **`a_persons_change_...`: not explained by reading.** It uses no box and no `run_spend`, so neither cause reaches it, and the request gives no message. `restart-handover` runs it first; its options cover a stale walk or a product fault.

**`temper::live`'s two owned-forge stories, not run:** the same cause 2. `owned_forge_landing` calls `forge_land::assert_forgejo_setup` for its second run (`temper/crates/temper/tests/live.rs:114`), which holds that same `no forge-N/admin_token`, and neither story calls `run_spend`.

## Decisions

- **Serve keeps starting first; it reads the handover again.** The first reconcile has waited for Forgejo, and Forgejo runs only after preparation, so a read then finds what preparation made. No line of the box or the link changes, and the row reads connected only with the handover gone, which the csv story's revocation probe needs.
- **Rejected: preparation before serve.** `ui-repo-link-2`'s later preparation runs `update-oauth`, which may fetch discovery from serve, and would hang there.
- **Rejected: taking it at the box's unasked `started N`.** The forge would first read connected from the kept token, then go connecting again, racing every `--until settled` read. And a serve restarted alone, under a live group, is told nothing, so waiting for that line would hang it.
- **The started cut runs on a box with no sign-in,** init's own document without `openssl`. The cut holds the engine, the source needs the engine, and no race-free hold on the default box was found; the task may take one if it finds it. No assertion moves.
- **No stored format, protocol or public interface changes.**

## Left open, for the user to see

**A Forgejo that exits under a running serve** runs preparation again, makes a new handover, and no `started` reaches serve for it, so the file stays until serve's next start. It's the same secret on disk by a rarer road, and older than `ui-repo-link`. It isn't in the five stories or the ruling, so it's written down and not built: `setup.md`, 15, *A handover made by a Forgejo's own restart*.

## Critical path

`spend-template`, then `restart-handover`, the only task that runs the five stories.

## Overlaps

- `first_repository.rs`, with `model-catalogue-1/stock-seven`, which rewrites `run_spend` to read `bare.json`, with no placeholder. `spend-template` says what to do if that lands first; whichever merges second keeps that form and the new test.
- `temper/docs/design/protocol/setup.md`, `sign-in.md`: no draft beside this one names them now that `ui-repo-link-2` has landed.
