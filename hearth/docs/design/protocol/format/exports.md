# The IaC format: exports

## 1. Exports

### 1.1 Declaring

- **`exports` is a map from export name to a spec** like a setting's
  (`type`, `doc`, `optional`) and a `value`, written as one
  `{{ expr }}`.
- **The value is checked as a setting's is** (`schema.md`, 1.5), on each target that
  evaluates it: its type and form, an object's fields declared and its
  required ones present, a `target` that exists in the environment, a
  `file` the revision holds.
- **Computed from settings only:** the task's own settings and `target`.
  Never from another task's reads, never from another target's exports,
  so exports cannot form a cycle; never of type `secret`, at any depth of
  the type (1.4). `debian`, `tasks` and `exports` are refused in a
  value, and a `settings.<name>` the task does not declare is an error,
  as anywhere (`schema.md`, 1.4).
- **An optional export** whose expression is undefined is absent.
- The exports part holds them (hearth/docs/design/domain/render/spec.md, 1.4); a change to an export
  changes exactly the specs that read it.

### 1.2 Reading

- **Exports are read by task, which is to say by role.** A reader
  declares `"reads": {"vpn": {"exports": true}}`, and its templates see
  `exports.vpn`: a map from target name to that target's vpn exports, for
  every target of the environment that has the task, itself included.
- **Heterogeneous pairs read the same way.** The bash fleet's pairs are
  roles, not identical members: mail's primary and backup, the VPN's hub
  and spokes. Each side exports its role, and the reader picks by it
  (`templates.md`, 2's `mynetworks`).

### 1.3 Scope

- **Exports never cross an environment.** A lab target reads only lab
  targets' exports, and production only production's. The bash fleet's
  recurring error, lab variants carrying production's public keys, and a
  container carrying production's forge host key in the lab, cannot be
  written.
- **Render evaluates the exports of the environment's targets** to render
  any one of them (hearth/docs/design/domain/render/README.md, 3.4): rendering stays a function of the
  revision and the target.

### 1.4 Public halves as data

- **Secrets are never exported, but their public halves often must be:**
  a WireGuard public key, an SSH host key for `known_hosts`, a Syncthing
  device id. Render never sees a secret, so it cannot derive them.
- **So the public half is recorded in the repository** as a setting of
  the target that owns it, in its base-env file. For identity, the
  activator generates the item and reports its public half as a fact
  (hearth/docs/design/protocol/wire.md, section 5); a person or an agent records it, and the change's
  rendered diff shows every peer that picks it up. Until then the
  setting is absent, the optional export with it, and peers skip that
  target.

### 1.5 Fleet-wide constants

- **A value every target shares is a default in `fleet.json`, not an
  export:** the domain, the operator's SSH keys. Exports are for what
  differs by target and must agree between targets.

### 1.6 Example: the VPN

`tasks/vpn/task.json`:

```json
{
  "doc": "A WireGuard network between the targets of an environment, through a hub.",
  "settings": {
    "address": { "type": "string", "doc": "This target's address on the network, such as 10.0.0.2." },
    "hub": { "type": "bool", "default": false, "doc": "Whether this target relays for the others." },
    "endpoint": { "type": "string", "optional": true, "doc": "The public address peers dial, for a hub." },
    "port": { "type": "int", "min": 1, "max": 65535, "default": 51820, "doc": "The UDP port." },
    "public_key": { "type": "string", "optional": true, "doc": "This target's public key, as its activator reported it." }
  },
  "reads": { "vpn": { "exports": true } },
  "identity": {
    "wireguard": {
      "kind": "wireguard-key", "path": "/etc/wireguard/wg0.key",
      "mode": "0640", "owner": "root", "group": "systemd-network"
    }
  },
  "exports": {
    "address": { "type": "string", "value": "{{ settings.address }}", "doc": "Its address on the network." },
    "hub": { "type": "bool", "value": "{{ settings.hub }}", "doc": "Whether it relays." },
    "endpoint": { "type": "string", "optional": true, "value": "{{ settings.endpoint }}", "doc": "Where to dial it." },
    "public_key": { "type": "string", "optional": true, "value": "{{ settings.public_key }}", "doc": "Its public key." }
  }
}
```

`hosts/west/west.json` holds `"vpn": {"address": "10.0.0.1", "hub": true}`,
and `hosts/west/west-prod.json` what belongs to west in production,
`"vpn": {"endpoint": "203.0.113.157", "public_key": "AAAA…="}`, the
key as west-prod's activator reported it.

`tasks/vpn/tree/etc/systemd/network/wg0.netdev.j2`:

```jinja
[NetDev]
Name=wg0
Kind=wireguard

[WireGuard]
PrivateKeyFile=/etc/wireguard/wg0.key
ListenPort={{ settings.port }}
{% for name, peer in exports.vpn | items %}
{% if name != target.name and peer.public_key is defined and (settings.hub or peer.hub) %}

[WireGuardPeer]
PublicKey={{ peer.public_key }}
AllowedIPs={{ peer.address }}/32
{% if peer.endpoint is defined %}
Endpoint={{ peer.endpoint }}:{{ settings.port }}
PersistentKeepalive=25
{% endif %}
{% endif %}
{% endfor %}
```

- carbon-prod, a spoke, gets west-prod's key, address and endpoint from
  west-prod's file; west-prod gets carbon-prod's key from carbon-prod's.
  Nobody copies a key, and carbon-lab sees only lab peers, with lab keys.
- **`wireguard-key` is [later]** (`README.md`, 7.3). Until it exists, the vpn task
  names the private key as a secret file at the same path, an operator
  imports it, and records its public half in the target's file just as
  above. The group `systemd-network` owns the key, so the task declares
  it from its package with a fixed id (`tasks.md`, 6).

### 1.7 Example: the backup peer [M6]

- **Backups are encrypted to the peer's key** with `age` before any byte
  reaches a steward (hearth/docs/design/domain/operating.md, section 7). The peer's `backup_peer`
  task holds that key as identity and exports its public half, recorded
  as data like any other (1.4):

  ```json
  {
    "doc": "Receives the environment's backups, encrypted to its own key.",
    "settings": {
      "recipient": { "type": "string", "optional": true, "doc": "This peer's age recipient, as its activator reported it." }
    },
    "identity": { "backup": { "kind": "backup-key" } },
    "exports": {
      "recipient": { "type": "string", "optional": true, "value": "{{ settings.recipient }}", "doc": "Where backups are encrypted to." }
    }
  }
  ```

- **The backed-up target's render reads it.** A class that the `hearth`
  task's `backups` names (`tasks-hearth.md`, 1) makes render read `backup_peer`'s
  exports in the target's environment and put the peer and its
  recipient into that class's policy in the data part (hearth/docs/design/domain/render/spec.md, 1.3),
  where the activator takes it at prepare. A target with backups and no
  peer that exports a recipient is an error.
