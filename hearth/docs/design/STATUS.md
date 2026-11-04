# Status

Updated 2026-11-04. How far hearth's code is from its design.

| Area | Built | Not yet |
|---|---|---|
| `hearth/docs/design/README.md` | 4.2: render's libraries, builder root, protocol layer and service. | 8.1: remaining crates/support. |
| `hearth/docs/design/protocol/format.md` | Format. | 11: checks. 10.2: artifacts. |
| `hearth/docs/design/domain/render.md` | Render and client render/diff. | — |
| `hearth/docs/design/domain/store.md` | 3.1, 3.2: canonical encoding and H. 2.1, 2.4, 4.4, 6.1, 6.4 to 6.6, 6.8, 8.1, 8.2: layerless store/resolutions/cache/lock/scan. 6.2, 5.3: layer record/accounts. 4.2, 7.3: stage/seal/publish/discard, fake helper/digests. | Other layers/store; 10: peers. |
| `hearth/docs/design/domain/build.md` | 3.3, 4.1, 4.3: build keys/plan/stage/seal/discard; steps refused. | Steps and remaining build. |
| `hearth/docs/design/domain/machine.md` | None. | 6.1: tokens. 7: initramfs /boot; EFI/BIOS, containers/pool hosts. |
| `hearth/docs/design/domain/slots.md` | None. | EFI slots/bless/live changes/GRUB; 12: containers. |
| `hearth/docs/design/domain/activator.md` | None. | Slot/live operations, restarts, snapshots, containers/backups/disks. |
| `hearth/docs/design/domain/steward.md` | Builder domain in client (`hearth/docs/design/client/README.md`, 3.5). | Steward; other detail unknown. |
| `hearth/docs/design/protocol/wire.md` | None. | 2, 4: faces/standalone. 5, 9: wire. 7: telemetry. 8: bytes by digest. |
| `hearth/docs/design/client/README.md` | 3.1 to 3.3, 3.5: in-process render/diff. | 3.5: proxy; remaining client/retarget/development. |
| `hearth/docs/design/domain/operating.md` | None. | All. |
| `hearth/docs/design/domain/testing.md` | 2.9: the worlds' suites. 5: conformance of git, resolve and render. 10: render/git/mmdebstrap fakes. | 6.6: boot test; other tiers. 3.11: shared fake machine. |
| `hearth/docs/design/domain/development.md` | None. | Mutable machines, capture/apply, Incus, clusters/containers and VMs. |
| `hearth/docs/design/domain/artifacts.md` | None. | All. |
| `hearth/docs/design/domain/reproducible.md` | None. | All. |

- **Not yet:** `hearth/docs/design/domain/testing.md`: 5.5: nothing runs `conformance-hearth` when a fake or a recording changes, since neither the gate nor the after-landing tier names it (`.config/gate/README.md`): `recorded_*`, which needs no prerequisite, runs by hand alone, and nothing records `conformance_*`, which need the real tools and the mirror, passing on the skein hearth now builds on.
- **Departs:**
  - `hearth/docs/design/domain/render.md`: 8.4: the builder's worst case is no longer 6.3's sum, because `skein/docs/foundation/programming-model.md`, 6.3, ended the whole-process figure and skein removed its io and file terms. `hearth_service_builder::worst_case` keeps its old name and sums hearth's domain, protocol layer and queues; only its tests call it, and the 1 GiB they hold it to is no process's figure (`hearth/docs/development/debt/D002-process-worst-case.md`).
  - `hearth/docs/design/domain/testing.md`: 5.5, 9.2: the `conformance` profile is `conformance-hearth` and also runs `recorded_*`, and there is no `measure` profile, because the root's tier profiles are `<tier>-<tree>`.
  - `hearth/docs/design/domain/testing.md`: 8.4: a world's memory tests are modules of its suite, not a binary of their own, because a suite has one counting allocator (`skein/docs/design/testing.md`, 6). The builder world's four scenarios and git's step peak are checked under ceilings, with no worst case of the whole, because the root's rules are skein's (`skein/docs/foundation/programming-model.md`, 6.3). The other worlds' peaks are still held to formulas, and the render world keeps `Memory::Checked` as a flag beside its own meter, where the user's request of 2026-11-04 asks for `Memory::Within`, because that meter still enforces its bound (`hearth/docs/development/debt/D003-memory-tests-on-formulas.md`).
  - `hearth/docs/design/domain/testing.md`: 9.1: no global timeout holds hearth's suites alone, because the root has one `default` and one `fuzzy` profile, whose 150 s and 600 s sum each tree's figures, hearth's 30 s and 120 s among them. Measured on 2026-11-04 on a shared machine: focused, 35 to 36 CPU s and 6 to 7 s of wall; fuzzy, 68 CPU s and 16 to 21 s of wall. On a full gate hearth adds 27 s to the test build and 9 s to clippy. A change to `skein-lib` reaches 28 of hearth's 30 packages and rebuilds them in 3 s of wall, with its compiler caches warm. `hearth-render-world`, in `.config/gate/always.txt`, costs every change 3.4 CPU s focused and 29 CPU s fuzzy. The figures await the user (`hearth/docs/design/domain/testing.md`, 11).
