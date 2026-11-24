# The activator: its state

## 1. Its state

### 1.1 Layout

```
/var/lib/hearth/state                       @hearth.state: root, 0700
├── machine                                 target, firmware, fs uuid, boot partition's volume id,
│                                           the ESP's GPT GUID, entry numbers, layout version; mutable:
│                                           accept or refuse; [M3b] a retired target (../development.md, 3.2)
├── boot                                    the booted kernel: boot id, its digests (../slots/trial.md, 1)
├── journal                                 the operation in flight, and its step (7.3)
├── facts                                   quirks seen, drift's last results, last costs
├── snapshots/ [M5]
│   └── x1-prod.data                        the class's snapshots: each sequence, its time, the buckets that keep it,
│                                           and [M6] sent, confirmed by the peer; the next deadline
├── slots/
│   ├── x1-prod.a                           slot record (../slots/README.md, 2.2)
│   ├── x1-prod.a.manifest                  one entry per path of the sealed root
│   ├── x1-prod.b
│   └── x1-prod.b.manifest
├── trial/
│   └── x1-prod.b                           the trial marker, only while a trial runs (../slots/trial.md, 4);
│                                           the dead-man is armed from /run and the command line instead
├── live/
│   ├── x1-prod                             the live record, while a live state exists (../slots/README.md, 2.4)
│   └── x1-prod.manifest                    the merged view's expected entries
├── identity/
│   └── x1-prod/                            generation, machine-id, SSH host keys, the backup key [M6], public halves
├── secrets/
│   └── x1-prod/
│       ├── index                           the store's generation; each name's value file, digest, size
│       └── <name>@<generation>             one value per file, 0600
└── containers/ [M5]
    └── chat-prod                           on east-prod: the container's default slot, its id base,
                                            and a live-blessed slot's "next start is its trial" (../slots/containers.md, 8)
```

- **A host keeps its containers' records beside its own [M5],** each by
  the container's target: `slots/chat-prod.a`, `trial/chat-prod.b`,
  `live/chat-prod`, `identity/chat-prod/`, `secrets/chat-prod/`. The
  default has a record of its own, since no firmware holds it
  (hearth/docs/design/domain/slots/containers.md, 6).

### 1.2 Records

- **The filesystem is the only state** (README, section 1): subvolumes,
  and these records beside them, read back at every start. No kv store.
- **Each record is sized,** a header with its kind and format version
  then its fields, encoded by `hearth-protocol-state` (`tools.md`, 1.1). The domain
  never sees their bytes.
- **Each is written with skein's `Store`:** a temporary file, synced,
  renamed over the old one, the directory synced; conditional on the old
  one's digest where two writers can meet (the secret index).
- **Formats are read back across releases.** An activator reads every
  format version it ever wrote and refuses a newer one, reporting it. A
  newer activator writes no record in a format the older one cannot read
  while the older one's slot is still blessed, so a roll back always
  finds its state readable.
- **Bootstrap writes as the release it places.** It refuses to run when
  its own binary's digest differs from the one the output pins in its
  `hearth` task (hearth/docs/design/domain/machine/bootstrap.md, 1), since the records it writes are read
  next by that release.

### 1.3 The journal

One record: the operation in flight, its key or attempt, its slot, its
step and when it started. It is written before each step whose effects
leave the state (a subvolume, the boot partition, NVRAM, a mount, a
reboot), and cleared when the operation ends.

| Operation in the journal at start | Recovery |
|---|---|
| place | the slot is `preparing`: delete it, nested subvolumes first (hearth/docs/design/domain/slots/crashes.md) |
| trial, roll back | by the marker's phase and the facts (hearth/docs/design/domain/slots/README.md, section 4) |
| bless | the record says `blessed`: remove the marker, move the default (re-bless), stop the dead-man; otherwise the slot stays on trial |
| fall back | by the marker and the facts |
| try live | at L1 or L2: the delta deleted, the slot `prepared`; from L3, by the marker's phase: `arming` disarmed, later `reboot -ff` (hearth/docs/design/domain/slots/crashes.md) |
| bless live | the record says `live-blessed`: set the one-shot, set the marker's phase, stop the base's dead-man; otherwise the slot stays on live trial |
| the live fall back | `reboot -ff`, unless its last step was done |
| generate identity | remove `.new` |
| hold a bless [M4] | the marker `held` and its transient timer: nothing to redo |
| run a check [M5] | the transient unit, collected by systemd: run it again |
| restart a unit [M5] | read the invocation id: changed means done |
| a container's trial, bless or fall back [M5] | by the marker and the facts, with the link and settings in `/run` (hearth/docs/design/domain/slots/containers.md, 3) |
| partition and format [M6] | look for the identifiers: found means done |
| snapshot, prune [M5] | look for the snapshot by its sequence: found means done; prune again |
| send, receive [M6] | look for the received snapshot: found means done; a send not confirmed is sent again |
| restore [M5] | before the swap is recorded, delete the copy or the writable snapshot (`.restoring`); after, the recorded swap: finish it, or undo it, then reboot |
| a fault [M6] | look for it by the experiment's key; its end still holds |

### 1.4 Start

At every start, after a boot, a soft reboot or a crash:

1. **mount the top level** at `/run/hearth-activator/top`, unless it is mounted;
   find the disk and the boot partition from
   `/sys/fs/btrfs/<fs uuid>/devices/`; then the live layers' source,
   `/run/hearth-activator/layers`: a clone of the top level, private,
   read-only, without `noexec` and `nosuid` (hearth/docs/design/domain/slots/trial.md, 8);
2. **read** the machine record, the slot records, the marker, the
   journal and the boot record;
3. **read the facts:** running, default, the one-shot, `BootCurrent`,
   the boot id, soft reboots, `SecureBoot`, and the live fact from the
   mount table against the live record (hearth/docs/design/domain/slots/README.md, section 3);
4. **on a new boot id,** record the booted kernel: the running slot's
   kernel digests (hearth/docs/design/domain/slots/trial.md, 1);
5. **recover the journal's operation** (1.3);
6. **apply the start events** to both slots (hearth/docs/design/domain/slots/README.md, section 4): a
   live trial past arming in this boot, or a live record whose base
   still runs but whose overlays disagree with it (a person's soft
   reboot), ends here in `reboot -ff`, before anything else; a live
   record whose base no longer runs, or whose overlays a reboot
   dropped, is removed;
7. **repair what a firmware or a cut left:** an environment pair that
   disagrees (hearth/docs/design/domain/machine/boot.md, 2.5); hearth's entries out of order in
   `BootOrder`; a normal entry of a blessed slot that the firmware
   pruned, made again; the dead-man of a running slot that is blessed,
   stopped; on BIOS, `tries` reset when the running default meets the
   criterion (hearth/docs/design/domain/slots/bless.md, 2.4); each delta subvolume that neither the live
   record nor the marker names, deleted; after an adoption [M3b], the
   retired target's slots, entries, ESP files and records, deleted
   (hearth/docs/design/domain/development.md, 3.2);
8. **start the containers** [M5]: each container's settings and link
   that `/run` lacks, from its recorded default, then `systemctl start`
   on its unit (hearth/docs/design/domain/slots/containers.md, 2); and set each class's snapshot deadline
   from its record and the running slot's policies, a snapshot missed
   while the machine was down taken at once (hearth/docs/design/domain/operating.md, 7.1) [M5];
9. **spawn the steward;** its `Welcome` carries the facts.

Every step is idempotent, so a crash during the start repeats it.
