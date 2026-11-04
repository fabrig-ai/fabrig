# Status

Updated 2026-10-26. How far hearth's code is from its design.

| Area | Built | Not yet |
|---|---|---|
| `README.md` | 4.2: render's libraries, builder root, protocol layer and service. | 8.1: remaining crates/support. |
| `protocol/format.md` | Format. | 11: checks. 10.2: artifacts. |
| `domain/render.md` | Render and client render/diff. | — |
| `domain/store.md` | 3.1, 3.2: canonical encoding and H. 2.1, 2.4, 4.4, 6.1, 6.4 to 6.6, 6.8, 8.1, 8.2: layerless store/resolutions/cache/lock/scan. 6.2, 5.3: layer record/accounts. 4.2, 7.3: stage/seal/publish/discard, fake helper/digests. | Other layers/store; 10: peers. |
| `domain/build.md` | 3.3, 4.1, 4.3: build keys/plan/stage/seal/discard; steps refused. | Steps and remaining build. |
| `domain/machine.md` | None. | 6.1: tokens. 7: initramfs /boot; EFI/BIOS, containers/pool hosts. |
| `domain/slots.md` | None. | EFI slots/bless/live changes/GRUB; 12: containers. |
| `domain/activator.md` | None. | Slot/live operations, restarts, snapshots, containers/backups/disks. |
| `domain/steward.md` | Builder domain in client (`client/README.md`, 3.5). | Steward; other detail unknown. |
| `protocol/wire.md` | None. | 2, 4: faces/standalone. 5, 9: wire. 7: telemetry. 8: bytes by digest. |
| `client/README.md` | 3.1 to 3.3, 3.5: in-process render/diff. | 3.5: proxy; remaining client/retarget/development. |
| `domain/operating.md` | None. | All. |
| `domain/testing.md` | 10: render/git/mmdebstrap fakes. | 6.6: boot test; other tiers. 3.11: shared fake machine. |
| `domain/development.md` | None. | Mutable machines, capture/apply, Incus, clusters/containers and VMs. |
| `domain/artifacts.md` | None. | All. |
| `domain/reproducible.md` | None. | All. |
