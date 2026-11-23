# Render: the rendered diff

## 1. The rendered diff

### 1.1 Typed

```
diff
  from, to             commit ids
  targets              map target → change
change
  added { spec digest } | removed | same
  | changed { parts: list of part change, rebuild }
  | failed { side: from | to | both, errors }
part change
  packages             request: added, removed; sources, variant, preseeds, initramfs, apt: before, after;
                       seeds: users and groups added, removed, changed, by name;
                       resolution: same | new key | new key, resolved: added, removed, versions changed
  software             downloads and artifacts added, removed, changed, by path; scripts changed
  config               tree entries added, removed, changed { path, before, after };
                       groups, users, members, presets, tmpfiles: added, removed, changed, by key;
                       scripts changed
  data                 entries added, removed, changed, by name
  secrets              names added, removed, changed; never a value
  boot                 before, after
  live                 paths added, removed, changed
  exports              task.name: before, after
  checks [M5]
  meta                 base, member, environment, placement: each before, after, when it changed
rebuild
  from packages { resolution: known | needed } | from software | from config | slot only | nothing
```

- **A changed file carries its two digests and sizes, and its hunks.** The domain asks for them (`crates.md`, 1.2) when both sides are under 1 MiB: git computes them from the two contents in the store's cache (`README.md`, 2.2) with three lines of context, and `hearth-protocol-git` hands up at most 200 lines a file, with the count of those left out, or that the file is binary. The lines count from the first `@@`, each `@@` line among them, and each is a hunk's header, context, removed, added, or `\ No newline at end of file`. The domain holds one target's hunks at a time, and step code computes no text diff. The hunks are part of the typed diff, so its JSON carries them (hearth/docs/design/client/README.md, 3.2).
- **Secrets show names and places,** never values: render has none.
- **`failed` is a target whose spec did not render** on a side that has it, or on `both`. Its errors are those sides' (`errors.md`, 1) and print after the changes; it counts among the diff's targets, neither changed nor unchanged. A resolution that fails leaves the spec rendered, so its change stands (`spec.md`, 2.7).
- **Every part's change says what moved its digest:** a live-only change rebuilds `nothing` (`spec.md`, 1.4), as does a change to meta, and no changed target shows an empty list of part changes.

### 1.2 Targets touched and costs

- **A change touches the targets whose spec digest differs,** and those it adds or removes: a change to a task, every target that has it; to an export, every reader in that environment; to `envs/lab.json`, no production target.
- **Rebuild** is the lowest layer whose key changed: from packages (and whether the new resolution is known), from software, from config. A change to data, secrets or boot alone is a new slot of the same output; a change to exports alone builds nothing on the exporting target.
- **Time,** when the builder has built the target before, is its last recorded cost for each layer to rebuild. Whether activation needs a reboot is known only after the build, from the kernel's and the initramfs's digests, and whether it can be live from the two outputs' tree files and live parts (hearth/docs/design/domain/slots/trial.md, 7).

### 1.3 Printed

A commit changes west-prod's public endpoint, in `hosts/west/west-prod.json`:

```
$ hearth diff main HEAD
carbon-prod                                         rebuild from config
  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root
               @@ -9,5 +9,5 @@ ListenPort=51820
                [WireGuardPeer]
                PublicKey=AAAA…=
                AllowedIPs=10.0.0.1/32
               -Endpoint=203.0.113.157:51820
               +Endpoint=203.0.113.158:51820
                PersistentKeepalive=25
west-prod                                           nothing to build
  exports  ~ vpn.endpoint   "203.0.113.157" → "203.0.113.158"

2 of 11 targets change; 9 are unchanged.
```

- The client prints it as text, or as JSON for agents (hearth/docs/design/client/README.md, section 3); the typed diff is the same either way.
- The coupling shows: carbon-prod changes because it reads west-prod's exports, and carbon-lab doesn't, because exports stay in their environment.
- **Hunks print as git gives them,** three lines of context and each `@@` line with git's own context after it (hearth/docs/design/protocol/format/exports.md, 1.6's template rendered for carbon-prod). A failed target prints `failed (from)`, `failed (to)` or `failed (both)` where the rebuild goes, and its errors after the changes.

### 1.4 Who posts it

- **Open:** the lab, CI or temper's gate could post it at a change's head, as a status is; and whether a change that touches many targets is folded by identical part changes follows from who reads it.
