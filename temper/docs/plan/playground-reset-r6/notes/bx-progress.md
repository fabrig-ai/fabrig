# bx: the box, the binary, init and the process tier (progress)

Designer bx of playground-reset-r6. Crates: temper-box, temper-forge-setup,
temper-forge-setup-shell, temper-fake, temper-init, temper (the binary, its src and tests).

## Specs

| id | status | needs | crates |
|---|---|---|---|
| bx-pipe | final | - | temper-box |
| bx-steps | final | - | temper-forge-setup (setup.rs, step.rs), temper-fake (steps.rs) |
| bx-compose | final | bx-pipe, bx-steps | temper (box_command, forge.rs, fake.rs, suite tests) |

- `bx-pipe`: the box's load seam (`Service::take_load`/`loaded`, `take_removal`/`removed`,
  `Loaded::{Group, Absent, Refused}`), group slots (`box.groups`, 4), `remove N`/`removed N`,
  `failed N no settings`. The box reads no file of a group: its shell (the binary, in
  bx-compose) reads `box/groups/N`, builds the steps and removes the files. Recorded groups
  at a box start are loaded by the shell before construction and given in `Steps.asked`, so
  they join the start chain before serve as today.
- `bx-steps`: `temper_forge_setup::setup::{Entry::group, Entry::body, Entry::body_text,
  group_settings, directory, BODY_BYTES, GROUP_BYTES}`; `temper_forge_setup::step::owned_group`;
  `temper_fake::steps::{playground_group, group}`. For sv: `Entry::body`/`body_text` check
  `application.machine.owned`, `application.demo.forge` and a record's setup text;
  `group_settings(body, N)` is the text serve writes to `box/groups/N`. Name `bx-steps` in
  `needs` to call them.

- `bx-compose`: the binary answers the seam (`box_command/groups.rs`, `Boxed` host), loads
  recorded groups from files at the box's start, and `temper forge ... N`/`temper fake forge N`
  read `box/groups/N` first, the document's setup entry as a fallback. It does not touch
  `temper/crates/temper/src/host.rs` or `lib.rs` (sv's request 2 may edit them in parallel).
- sv's request 3 (one entry parsed alone) is answered by `bx-steps`:
  `Entry::body_text(text, forge, limits, directories)` (and `Entry::body` at a document
  position, `Entry::group` for the file).

## Contract requests

1. **A failed removal on the pipe (T3).** T3 gives `remove N` only its success,
   `removed N`. bx-pipe answers a removal the shell could not finish `failed N not removed`
   (the group stays, dormant; a later `remove N` tries again), and a `remove N` while another
   group starts or stops `failed N busy`, as `stop N` is answered. **Request:** sv's box link
   takes a `failed N ...` while its `remove N` is in flight as "not removed": the kept act
   stays and is sent again (at the next start at the latest). No change to T3's lines.
