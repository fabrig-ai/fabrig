# hearth-names: the mapping, what it moves, and what the design changed

The request is `hearth/docs/plan/hearth-names/inputs/request.md`. Line numbers here are at the cut's base.

## 1. The mapping

| Now | Becomes |
|---|---|
| `serverhino` | `provider-a` |
| `ionos` | `provider-b` |
| `vultr` | `provider-c` |
| the host `rhi`, and every name built on it: `rhi-prod`, `rhi-lab`, `hosts/rhi/`, `rhi.json`, `rhi/service.key` | `east`, `east-prod`, `east-lab`, `hosts/east/`, `east.json`, `east/service.key` |
| the host `ion`, and every name built on it: `ion-prod`, `ion-lab`, `hosts/ion/`, `ion.json` | `west`, `west-prod`, `west-lab`, `hosts/west/`, `west.json` |
| `m3NY8…=` | `AAAA…=` |
| `RHI_PROD_CONTAINERS` | `EAST_PROD_CONTAINERS` |
| `BAD_ION_LAB` | `BAD_WEST_LAB` |

- **The last two rows are the request's rule applied to a form its search can't see:** `_` is a word character, so `\b` doesn't match there.
- **Stay as they are:** `x1-prod`, `mail-prod`, `carbon-prod`, `chat-prod`, `mx1`, `mx2`, `7th-gen`, `stoa`, `os`, `ai/`, the digest `5a403c6e…`, addresses and domains.
- **The search leaves out this plan's own directory.** Its files spell the old names and leave main when the plan retires, the same exception `tests/boundary/tests/outward.rs` makes for `*/docs/plan/<draft>/`.
- **New commit messages spell none of the old names:** no history is rewritten, and none is added.

## 2. What the rename moves by itself

These are the rename's own effects: take them and list them in the report.

- **Lengths:** a host's name grows by 1 (`rhi`, `ion`), `ionos` and `vultr` by 5, `serverhino` not at all, and the key fragment shrinks by 1.
- **The padded column:** the diff's first line pads its target to column 52 (`hearth/crates/hearth-client-protocol-records/src/diff.rs`, line 143), so `ion-prod` and 44 spaces become `west-prod` and 43. A run of four or more spaces after a renamed name gives up what the name gained. Fifteen lines outside the documents hold such a run.
- **The constant separator:** `target       ion-lab   base ion   environment lab` has three spaces whatever the name's length. Those stay three.
- **Positions:** a position past a longer value moves by the difference, one before it doesn't.
  - `{"placement": {"kind": "vm", "firmware": "bios", "hardware": "ionos"}, "vpn": {"endpoint": 7}}` reports column 92 at base and 97 after: `hosts/west/west-prod.json:1:97:` in `hearth/tests/builder/tests/real/exports.rs`, lines 93 and 94, and in the `errors/exported` golden.
  - `Position { offset: 44, .. }` for `"on": "rhi-prod"` doesn't move, since the value starts where it did.
- **Digests:** a digest moves when what it hashes holds a renamed name, or is a fleet whose inputs do. `hearth/tests/builder/tests/real/exports.rs`, lines 79 and 80, pin `/etc/vpn/peers`, which holds a target's name. The value is the first 12 hex digits of the file's sha256, so check it against `tests/render/exports/rendered/<target>/etc/vpn/peers`. A digest whose inputs hold no renamed name stays: `x1-prod`'s `THIN_TEXT` and `DISTINCT_TEAMS_TEXT` in `hearth/tests/builder/src/fixtures.rs`, for one.
- **Sort order:** the targets sorted `carbon, chat, ion, mail, mx1, mx2, ok, rhi, stoa, x1` and now sort `carbon, chat, east, mail, mx1, mx2, ok, stoa, west, x1`, so `east` now precedes `west` and `mail`, and `west` follows `mx2`, `ok` and `stoa`. Output that sorts by target or path moves with them. A JSON body keeps its key order, as does `tests/config/examples/tasks/boot.json`, which matches `hearth/docs/design/protocol/format.md`, 5.4 line for line.

## 3. The design: what this cut changed

The cut's own commit carries these, under `hearth/docs/design/`. No section moved and no status file changed: the trunk lands whole, so main never shows the design ahead of the fixtures.

**Sentences dropped or rewritten** (`machine.md` is `domain/machine.md`, `format.md` is `protocol/format.md`):

- `machine.md`, 2.1: dropped "It runs os today, with direct EFI-stub entries labelled `a` and `b`, and already shows the facts apart: running slot `b`, `BootOrder` default `a`." It described one machine's state; the requirement that follows stays.
- `machine.md`, 2.3: kept the fact that two of three production VPSes are BIOS, and which provider each is on under the neutral names. Dropped "(primary MX, Forgejo, Matrix)" and "(backup MX, VPN hub)".
- `machine.md`, 2.5: dropped "The fleet runs three today, `carbon-forge`, `rhi-chat` and `vul-agent-control`, on carbon, rhi and vul."
- `format.md`, 3.5: "The Matrix container `rhi-chat`" became "The container `east-chat`"; "carbon-forge's `runner:600` on carbon" became "a container's `runner:600` on its host".
- `README.md`, 9: the open question on which private-looking names stay is gone, since the user's decision answers it. The list's new last item ends in a full stop.
- `machine.md`, 2.7 and `format.md`, 3.4: the `hardware` values now read `provider-a`, `provider-b`, `provider-c`, in that order.

**The firmware's behaviour stays as a requirement:** `machine.md`, 2.4 now opens "Provider C's firmware can discard its variables", and 8.1, 9 and 12 and `README.md`, 9 use the same name.

**Names only:** `domain/activator.md`, `domain/development.md`, `domain/operating.md`, `domain/render.md`, `domain/slots.md`, `domain/store.md`, `domain/testing.md`, and the examples of `machine.md`, `format.md` and `README.md`. Hand-aligned columns in `domain/development.md`, 2 and `domain/render.md`, 6.3 keep their places.

**Read and left, in the three documents the request names:**

- What the bash fleet and os did (`machine.md`, 2.4, 2.6 and 7.2; `format.md`, 3.5 and 5.10, and others): each is the reason for a requirement and names no provider, site or service.
- `machine.md`, 7.5 ("it boots the fleet today with the lists above"), 12 ("the fleet's BIOS machines") and `format.md`, 5.7 ("on the fleet's machines today"): each gives the reason for a choice or a number, and names no machine.
- `machine.md`, 2.1 ("The MVP's machine is `x1-prod`, a laptop") and 8.1 ("`x1-prod`'s holds os"): the names are on the request's list, and each carries a requirement.
- `format.md`, 10.1: the `forgejo` download is an example of a pin, tied to no machine.

**Left, and put to the user as the plan's question `port-and-zone`:**

- The SSH port `10127`: `format.md`, 3.7 and 5.10; `hearth/tests/config/examples/fleet.json`; `hearth/tests/render/fleet/fleet.json`.
- The zone `Europe/Lisbon`: `format.md`, 3.7, 5.3 and 5.10; `domain/build.md`; `domain/render.md`, 9.2; the fleet's fixtures and goldens; `hearth/tests/builder/tests/suite/goldens.rs`; `hearth/docs/design/draft/brainstorm.md`.
- `domain/store.md`, 9.2, which says how the fleet's containers clear a range today and names `carbon-forge`, `east-chat` and `south-agent-control`. It is outside the three documents the request names, so it took the mapping alone.
