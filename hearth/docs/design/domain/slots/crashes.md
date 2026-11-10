# Crashes and power cuts

The activator writes its journal before each step with effects outside
the state, and the trial marker before each one-shot
(hearth/docs/design/domain/activator/README.md, section 6). Every start reads them, reads the facts, and
applies the start events of `README.md`, section 4. What each cut leaves, with `a`
running and default and `b` the slot in question:

| Step | Activator crash | Power cut |
|---|---|---|
| prepare, any step from `prepare.md`, 1 to 7 | `b` `preparing`: deleted at start, nested subvolumes first; the steward asks again | the same after the reboot into `a`; ESP files of `b` may be torn, behind entries outside `BootOrder` that nothing boots but a person's choice (a fault cell, `README.md`, section 4) |
| prepare, record written, journal not cleared | `b` `prepared`; the journal is cleared | the same |
| trial, marker `arming` | `b` `trying`, same boot: `prepared`, one-shot cleared | new boot into `a`: `prepared`, one-shot cleared |
| trial, one-shot set, marker `armed`, before the reboot | same boot: `prepared`, one-shot cleared, `/run/nextroot` unmounted | power-on consumes `BootNext` or `next_entry`: `b` boots, `start here`: `on trial`. Soft kind: `a` boots, `fell back` (never reached hearth) |
| trial, the reboot under way | none: the activator is stopped by it | as above |
| on trial, before bless | restart in `b`: still `on trial`, the dead-man still running | `a` boots: `fell back` |
| bless, record written, default not moved | restart in `b`: `blessed`, not the default: the re-bless moves it (`bless.md`, 1.5) | `a` boots, the default: `b` is a blessed slot not the default, the roll back's target; converge tries it again |
| bless, default moved, dead-man not stopped | restart in `b`: stopped at start | `b` boots as the default, blessed, unarmed: nothing to do |
| fall back recorded, marker not removed | finished at start | finished at start |
| BIOS environment, a file torn | the other holds the old state; made whole at start | GRUB reads the other: the old state (hearth/docs/design/domain/machine/boot.md, 2.5) |
| live, L1 or L2 | the journal: the delta deleted; `b` stays `prepared` | the same after the boot into `a` |
| live, marker `arming` | `prepared`: the dead-man stopped, `/run/hearth-trial/` emptied, the delta deleted | `a` boots: `prepared`, the delta deleted |
| live, `armed`, `swapped` or `running` | `reboot -ff` at start, before anything else; then `a` boots: `fell back` | `a` boots, the overlays gone: `fell back`, the delta deleted |
| live bless, record written, one-shot not set | finished at start: the one-shot set, the dead-man stopped | `a` boots: `b` `prepared` again, the delta deleted |
| live bless, one-shot set, dead-man not stopped | finished at start | the power-on boots `b`'s trial: `start here`, `on trial` |
| `live-blessed`, before the native boot | a restart: ignored while the live fact agrees with the record | the power-on boots `b`'s trial (`trial.md`, 8, L9) |
| the live fall back, any step | `reboot -ff` at start, unless the journal shows it finished | `a` boots: `fell back` |

- **The default is written once per operation,** at bless, after the
  slot's record says `blessed`, and only to a slot that is running and
  passed the criterion in that boot.
- **Firmware fall-through reaches only blessed slots:** a slot leaves
  `BootOrder` before it is prepared, and enters it only at bless.
- **A one-shot is consumed before the kernel loads,** by the firmware or
  by GRUB, so no cut makes a trial sticky.
- **Recovery is idempotent:** every start event's handling may itself be
  cut and done again.
- **A live state never outlives its boot:** overlays are kernel state
  and `/run` is a tmpfs, so any reboot comes back as the default, or as
  the trial a live bless armed; the start then deletes every delta no
  record names. Mid-trial, `reboot -ff` is the one recovery: simple, and
  always correct before the live bless.
- **A person's soft reboot while live** is neither a crash nor a cut:
  the start finds the overlays disagreeing with the live record and runs
  `reboot -ff` (`trial.md`, 8).
- **A container's slot [M5]** follows the same table for prepare. A cut
  in its trial or bless leaves a link in `/run` and a marker: a power
  cut drops the link, and the host's start links the recorded default
  (`containers.md`, 3); an activator crash finds the link and the marker, and goes on
  as `containers.md`, 3 says.
