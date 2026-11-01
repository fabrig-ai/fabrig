# bx: the box, the binary, init and the process tier (progress)

Designer bx of playground-reset-r6. Crates: temper-box, temper-forge-setup,
temper-forge-setup-shell, temper-fake, temper-init, temper (the binary, its src and tests).

## Specs

| id | status | needs | crates |
|---|---|---|---|
| bx-pipe | final | - | temper-box |
| bx-steps | final | - | temper-forge-setup (setup.rs, step.rs), temper-fake (steps.rs) |

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

## Contract requests

(none yet)
