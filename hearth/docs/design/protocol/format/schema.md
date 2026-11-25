# The IaC format: the schema

## 1. The schema

### 1.1 Form

`settings` in `task.json` is a map from setting name to its spec:

```
spec  = { "type": type, "doc": string,
          "default"?: value, "optional"?: true,
          "min"?: int, "max"?: int, "max_length"?: int }
type  = "bool" | "int" | "string" | "path" | "mode" | "duration" | "secret" | "target" | "file"
      | { "enum": [string, …] }
      | { "list": type }
      | { "map": type }                               keyed by name
      | { "object": { field: spec, … } }              fixed fields, each a spec
```

- **`doc` is required:** one line, at most 200 bytes, for every setting
  and every object field. It is what an agent reads to edit the setting.
- **`min` and `max`** bound an int; **`max_length`** bounds a string's
  bytes, a list's elements or a map's entries. Each is refused on any
  other type: `min` and `max` on all but `int`, `max_length`, 0 or more,
  on all but `string`, `list` and `map`, so on a `path`, a `secret` or an
  `enum` too.
- **An `enum` lists one value at least,** each once.
- A schema is checked when its `task.json` is decoded: a default that does
  not have its type, `default` with `optional`, or `min` above `max` is an
  error in the task, wherever it is used. Whether a `target` in a default
  exists depends on the environment, so it is checked on each target
  whose settings hold it (1.3).

### 1.2 Types

| Type | JSON | Notes |
|---|---|---|
| `bool` | `true`, `false` | |
| `int` | integer | i64 |
| `string` | string | UTF-8, no U+0000 |
| `enum` | string | one of the listed values |
| `list` | array | ordered; replaced whole on merge |
| `map` | object | keyed by name; merged per key; iterated in key order |
| `object` | object | only its declared fields; merged per field |
| `path` | string | absolute; no empty, `.` or `..` segment; no trailing `/`; `/` alone is one |
| `mode` | string | three or four octal digits, at most `"7777"` |
| `duration` | string | a positive integer with no leading zero and one unit, `s`, `m`, `h` or `d`: `5s`, `1m`, `14d`; a number of seconds once decoded, which fits an i64 |
| `secret` | string | a name in the secret store (`README.md`, 2.4), never a value |
| `target` | string | a target of the same environment, which must exist |
| `file` | string | a path relative to the repository's root, with no empty, `.` or `..` segment and no leading or trailing `/`, of a file git records at the revision, mode `100644` or `100755`: never a link, a submodule or a directory |

- **No floats and no null.** Nothing in the fleet needs a fraction, and
  null is RFC 7386's deletion.
- **`secret` keeps values out of the repository.** A template that prints
  one prints a name.
- **`file` brings a repository file into a target,** such as the lab's
  certificate authority: `"extra_ca": {"lab": "lab-ca/lab.crt"}` with a
  `copy` entry in `files`. It replaces os's `copy` from a workspace
  outside the repository.

### 1.3 Defaults, optional and required

- **Defaults are the bottom layer** (`targets.md`, 1.2): the schema's defaults form a
  document that the composition merges first, so a layer can extend a
  default map or remove one of its entries with `null`.
- **Inside maps and lists,** whose entries the schema cannot name in
  advance, each object entry has its absent fields filled from the
  object type's defaults during validation: `"alice": {"uid": 1000}` gets
  `/bin/bash` from the users task's `shell` default.
- **`optional`:** the setting may be absent, and reads as undefined
  (`templates.md`, 6). A setting with neither `default` nor `optional` is required.
- **Validation fills what is absent,** after the composition: a setting
  from its `default`, and an object's field, at any depth, from its
  field's. So `null` for a setting that has a default restores the
  default, and `null` for an entry of a default map removes the entry.
  A default that is a whole value, an object or a map or list of them, has
  its fields checked and filled like any value, and a `target` in it is
  checked on each target whose settings hold it.

### 1.4 Reads

```json
{
  "reads": {
    "core": { "settings": ["domain"] },
    "wifi": { "settings": ["ssid"], "optional": true },
    "mail": { "exports": true }
  }
}
```

- **A task reads another task's settings only if it declares them:**
  `tasks.core.domain` in its templates and expressions. The read value is
  the other task's setting with its default applied; its type is the
  other task's.
- **The other task must be on the target,** unless the read is
  `optional`; then `tasks.wifi` is undefined where wifi is absent, and
  `tasks.wifi is defined` tells. That is a settings read: a read of
  `exports` alone needs the task on no target, and `exports.<task>` is
  an empty map where no target of the environment has it (`exports.md`, 1.2).
- **`exports: true`** reads that task's exports from the environment's
  targets (`exports.md`, 1.2).
- **References are checked statically:** every `settings.<name>`,
  `tasks.<task>.<setting>` and `exports.<task>` in a task's templates and
  declarations must be declared. `tasks` and `exports` take only literal
  attribute names after them. So the typo `settings.alow` fails at
  render, where os dropped a file.
- **What may stand alone:** `tasks` is read only as
  `tasks.<task>.<setting>`, both names literal, so bare `tasks`,
  `tasks[x]`, `tasks.<task>[x]` and `tasks.<task>` used whole are
  errors, but `tasks.<task> is defined` for an optional read.
  `exports.<task>` may be used whole, the map of 8.2. `settings` may be
  used whole or looked into with `settings[x]`, which is checked when
  evaluated: a name the task does not declare is an error there, and an
  absent optional one is undefined (`templates.md`, 6).

### 1.5 What is an error

Each with its file and line (hearth/docs/design/domain/render/errors.md):

- a setting the task does not declare, or an undeclared field at any
  depth of an object;
- a value of the wrong type, an enum value not listed, an int out of
  range, a malformed path, mode or secret name, a `target` that does not
  exist in the environment, a `file` absent from the revision;
- a required setting missing after composition;
- a read of a task or setting not declared in `reads`, or of a setting
  the other task does not declare;
- `null` for a task in `fleet.json` or `envs/`, `placement` in a base or
  in `fleet.json`, `members` in `fleet.json` or `envs/` (`targets.md`, 1.6), a task
  name that `tasks/` lacks.

### 1.6 Example: the users task

`tasks/users/task.json`, whose `users` declaration is `tasks.md`, 2's example:

```json
{
  "doc": "People's accounts: their ids, shells, groups and password hashes.",
  "settings": {
    "accounts": {
      "type": { "map": { "object": {
        "uid": { "type": "int", "min": 1000, "max": 59999, "doc": "The account's id." },
        "shell": { "type": "path", "default": "/bin/bash", "doc": "The login shell." },
        "groups": { "type": { "list": "string" }, "default": [], "doc": "Supplementary groups, such as sudo." },
        "password": { "type": "secret", "optional": true, "doc": "The secret holding the password's hash." }
      } } },
      "default": {},
      "doc": "Accounts, by user name."
    }
  },
  "data": {
    "home-{{ key }}": {
      "for_each": "settings.accounts",
      "path": "/home/{{ key }}", "mode": "0700", "owner": "{{ key }}", "group": "{{ key }}"
    }
  }
}
```

- `hosts/x1/x1.json`'s `"alice": {"uid": 1000, "groups": ["sudo"], "password": "users/alice.passwd"}`
  validates, and gets `/bin/bash`; its home is the directory
  `home-alice` in `@x1-prod.data`, bound at `/home/alice`.
- `"alice": {"uid": "1000"}` is a wrong type; `"alice": {"shel": "/bin/sh"}`
  an undeclared field; `"alice": {}` misses the required `uid`. Each error
  names `hosts/x1/x1.json` and the line.
