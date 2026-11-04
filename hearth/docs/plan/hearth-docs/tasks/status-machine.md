+++
id = "status-machine"
part = "status"
kind = "docs"
needs = ["names-docs"]
files = ["hearth/docs/design/domain/machine.md", "hearth/docs/design/domain/machine.status.md", "hearth/docs/design/domain/slots.md", "hearth/docs/design/domain/slots.status.md", "hearth/docs/design/domain/activator.md", "hearth/docs/design/domain/activator.status.md", "hearth/docs/design/domain/steward.md", "hearth/docs/design/domain/steward.status.md"]
effort = "medium"
minutes = 35
+++

# Status for the machine's side

## Goal

M3's documents, `hearth/docs/design/domain/machine.md`, `slots.md`, `activator.md` and `steward.md`, describe code that mostly doesn't
exist. Each gets a status file that says so briefly, closes with `Open questions`, and loses its build state.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task".

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `hearth/docs/design/README.md`, 3

## Owns and may change

- **Owns:** the eight files under `files`.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code, and section 1.3's names in `notes/contracts.md`.
- **Options:** nothing built means a one-line **Not yet** (`notes/status.md`, 3), with no section list.

## Steps

1. **Seeds:** `machine.md`: nothing built; 6.1: tokens; 7: initramfs and `/boot`; EFI, BIOS, containers and pool hosts. `slots.md`: nothing
   built; EFI slots, bless, live changes, GRUB; 12: containers. `activator.md`: nothing built; slot and live operations, restarts, snapshots,
   containers, backups, disks. `steward.md`: the builder's domain runs in the client (`hearth/docs/design/client/README.md`, 3.5); the steward
   isn't built, and the seed says "other detail unknown": find it.
2. Check by `rg` over `hearth/crates` and `hearth/testing/hearth-fake-machine` for each document's main types. M2's boot test depends on part of
   `machine.md` (its first paragraph says which): read that part closely. The fake machine isn't these documents' code.
3. Follow `notes/status.md`, 1 to 4, for each document.
4. Commit (`hearth: the machine side's status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
