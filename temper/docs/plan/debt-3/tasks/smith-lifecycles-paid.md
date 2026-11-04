+++
id = "smith-lifecycles-paid"
part = "smith-lifecycles"
kind = "docs"
needs = ["smith-domain-chat-states", "smith-stages-machines", "smith-hostio-inspection"]
files = [
  "smith/docs/development/debt/D002-lifecycles-in-flags.md",
  "smith/docs/development/debt/D003-host-io-route-table.md",
  "smith/docs/development/debt/D012-large-domain-files.md",
  "smith/docs/development/debt/README.md",
]
effort = "medium"
minutes = 15
+++

# Pay smith D002: the item and its row

## Goal

`Domain`, `Stages` and `HostIo` now hold their lifecycles as state enums. Check the item's
**Paid when** on the trunk, delete its file and row, and bring up to date the two items that name
it.

Acceptance, each read on the trunk:

- `Domain` (`smith-local-domain/src/domain.rs`) has 22 fields, 3 of them `Option` (`effort`,
  `choice`, `answer_wall`), and `interactive` its one `bool`.
- `Stages` (`smith-agent-service/src/service.rs`) reads 44, 1 and 6 by the commands of
  `temper/docs/plan/debt-3/tasks/smith-stages-machines.md`'s Acceptance.
- `HostIo` (`smith-local-service/src/host_io.rs`) has 30 fields, 3 of them `bool` or `Option`
  (`programs`, `shell_output`, `failed`), and `DeliveryIo` 6 fields.
- Every lifecycle has its state enum: `Delivering` (with `Input` and `Load` in the chat's states);
  `Line`, `Admission`, `Opening`, `Roots` and `Teardown`; `Close`, `TerminalInput`, `WindowRead`,
  `Signals`, `Child` and `Inspection`. Each has a `follow` where a state of it implies an effect.
  `Line`, `Signals` and `Inspection` have none, since no state of theirs implies one
  (`skein/docs/foundation/programming-model.md`, 5.4, last bullet), and `Child`'s is `drain_process`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md` and its README row are gone;
  `tools/citations/citations.py --all` reads 0; D003 and D012 name it no more.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md`, whole.
- The Acceptance of the five `smith-*` specs in `temper/docs/plan/debt-3/tasks/`.

## Owns and may change

- `smith/docs/development/debt/D002-lifecycles-in-flags.md` (delete) and its row in
  `smith/docs/development/debt/README.md`.
- `smith/docs/development/debt/D003-host-io-route-table.md`: delete the sentence "Goes well with
  D002's `HostIo` conversion."; each `host_io.rs:NN` in its **What** reads the line where
  the named item stands now (`operations`, `native_io_valid`, the third by its sentence).
- `smith/docs/development/debt/D012-large-domain-files.md`: in **What**, `host_io.rs`'s count reads
  what `wc -l` says. If it is 1,500 or more, the sentence reads "The last three belong to D001 and
  D003." Under 1,500, the file leaves the list: "Four remain", and "The last two belong to D001."

Nothing else changes. `smith/docs/development/debt/reviews/` stays as it is.

Pre-approved: if a count reads otherwise than the Acceptance, or a field stands that is meaningful
in some states only, delete nothing and report `Question:` with what you read. A count that differs
by a field an implementer's report explains is no question: report it and go on.

## Steps

1. Read each count and each enum on the trunk.
2. Delete the file and its row; edit D003 and D012 as above.
3. `python3 -I -B tools/citations/citations.py --base origin/main` and `--all` from the repository
   root.
4. Commit ("pay smith D002: the item and its row"), pushed.

## Report

As the implementer's text says, and also: each count as read.
