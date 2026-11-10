# A render's errors

## 1. The record

```
error
  kind        one of 7.2, with its own typed fields
  class       configuration | operational
  at          { path, line, column } in the revision, when there is one
  also        a second place: a collision's later task in name order, the layer that set a value
  target      the target, when the error is a target's
  task        the task, when it is a task's
  side        from | to, in a diff: the revision the error is of
```

- **Configuration errors** mean the revision is wrong and must change.
  **Operational errors** mean a tool or a resource failed (git, mmdebstrap,
  the network, a timeout) and the same revision may succeed later.
- **Render collects every independent error:** a broken document fails
  the targets that compose it, a broken task the targets that have it,
  and the rest goes on. A target asked that does not render always has
  an error printed for it: its own, or the revision's error it is left
  out under (`README.md`, 3.1, 3.4). Errors are sorted by path, line, column and
  target, then side, those with no place after those with one, by
  target; and bounded (`README.md`, 2.3), one bound over a command's answer, a diff's
  two sides together. The errors kept are the longest start of that
  order that holds at most 256 errors and 16 MiB of the bytes they carry,
  every string of every field counted; the rest are counted, not kept,
  and still decide the exit code (hearth/docs/design/client/README.md, 3.3).
- **One fault gives one error, where render stops reading:**
  - a document or a declaration gives its first fault, and the targets
    it fails print it once, without a target;
  - each setting is validated on its own: every setting at fault gives
    its error, and one at fault enters no value;
  - a schema fault stops the target before its guards: no guard is
    evaluated, and nothing renders;
  - every guard of a target is evaluated and each false one prints; a
    false guard stops its target's entries and files;
  - every file of a target renders, each giving its first fault.
- **An error's place is where the fault is read,** and an absent thing's
  is where its owner is: a missing setting at the key that gives its
  task to the target, a missing field at its object's `{`, a bad read
  at the read, and a template's fault at its own line and column.
- **In a JSON string or key,** a fault of what is written, which skein's
  tokenizer finds (`Escape`, `Surrogate`, `Utf8`, `Control`,
  `StringTooLong`), is where the tokenizer places it: the first byte of
  the character or the escape at fault (skein/docs/design/data/json.md, 3.6). A fault
  of the text decoded, which hearth finds once the tokenizer has read
  the string (U+0000, a name out of form, an expression's syntax), is
  at its opening quote: a decoded string keeps no escapes, so its bytes
  have no column of their own.

## 2. Kinds

| Kind | Examples |
|---|---|
| decode | not JSON, a duplicate key, a fraction, invalid UTF-8, a `null` inside a list, a document or a declaration not of its shape as written (a key, a name, a JSON form, or a value outside an entry that hearth/docs/design/protocol/format/README.md bounds itself, such as `doc`'s one line or a script's `order`: hearth/docs/design/protocol/format/tasks.md, 1, 2 and 8), a key of a later milestone (hearth/docs/design/protocol/format/README.md, 2.2), a template's syntax, an unknown filter; a limit met while reading the revision: its listing's, a file's, the decoded documents', declarations' and templates', the 1,025th target or task, a task's 513th setting (`README.md`, 2.3) |
| repository | a document out of place, an environment that does not exist, a directory named out of form, a path that is not UTF-8, a submodule, a stray file in a task, two sources for one destination, a `.j2` link or a bare `.j2` |
| composition | two targets of one name, a target's name past 63 bytes, a task that does not exist, `null` for a task in a default (the `hearth` task's is `reserved`), `placement` in a base, `null` for `placement` or `members`, `members` in `fleet.json` or `envs/`, a target with no placement, a container's `on` that names a container, a target of another environment or one with no placement (`README.md`, 3.4); [M5] a container's parameters on another kind, or an id range that overlaps another's (hearth/docs/design/protocol/format/README.md, 3.5) |
| schema | an undeclared setting, a wrong type, a missing required setting, an undeclared read, a bad default, an entry's field out of its kind's form once evaluated, such as a tmpfiles type or a boot parameter's `order` (hearth/docs/design/protocol/format/tasks.md, 2), a value of a later milestone (hearth/docs/design/protocol/format/README.md, 2.2) |
| evaluation | an undefined value used, a type error in an expression, an overflow, a `for_each` that is not a map or list; a limit met while composing, validating, expanding, evaluating or rendering a target, or holding its spec (`README.md`, 2.3) |
| exported | another target's exports that this one reads cannot be computed (`README.md`, 3.4): `target` is the reader, the kind names the exporter and its task, `at` is the cause's place, and the class is the cause's |
| guard | a false `requires`, with its message |
| reserved | a command-line key, a path, a unit name, a drop-in that sorts after `zz-hearth-watchdog.conf`, a tmpfiles line, an initramfs setting or a preseed that hearth reserves; anything but a directory above a reserved path, and a tmpfiles path with a glob or a specifier; a `live` entry render can tell is never live; a secret or identity file `hearth` could read, or one whose `mode` has a bit outside `0770`; `null` for the `hearth` task, in any layer; an initramfs module, a boot parameter or the `hearth` task on a container; [M5] a host's preset that enables one of its containers' nspawn units (hearth/docs/design/protocol/format/README.md, 3.4, and hearth/docs/design/protocol/format/tasks.md, 11, 12 and 14, and hearth/docs/design/protocol/format/README.md, 7.3; hearth/docs/design/domain/build/README.md, 6.3, 6.4 and 7.2; hearth/docs/design/domain/slots/prepare.md, 5; hearth/docs/design/domain/machine/boot.md, 3.2) |
| ids | an owner of a data path, a secret or an identity file without a fixed id; a download owned by anyone but `root` or a package user with a fixed id; a package user with one of `uid` and `gid` alone (`README.md`, 3.3; hearth/docs/design/protocol/format/tasks.md, 6, and hearth/docs/design/protocol/format/README.md, 8.1) |
| checks [M5] | a `blessing` check with `from`, a `from` no target has, a command's `user` that is `root`, or neither `nobody`, a sysusers user nor a declared package user (`README.md`, 3.9) |
| collision | one path from two tasks, two users that differ, a key a declaration makes twice |
| resolution | configuration: a package apt does not know, dependencies apt cannot satisfy, a suite the archive does not have (`spec.md`, 2.5), a requested package apt does not install (`spec.md`, 2.3); a closure past 8,192 packages, or a record past its bounds (hearth/docs/design/domain/store/records.md, 1). Operational: an `Inst` triple with no stanza, stanzas that disagree (`spec.md`, 2.3) |
| tool | git or mmdebstrap failed, timed out, could not start or gave output it should not (`README.md`, 2.2, `spec.md`, 2.3), or a resource failed, as `spec.md`, 2.5 says: operational; mmdebstrap's failure of no cause `spec.md`, 2.5 knows: configuration |
| store | a store full at a limit, or whose upkeep failed (hearth/docs/design/domain/store/README.md, 6.1): operational, the call's, naming the store and the limit and count or the step. A store that cannot be opened, or is another uid's or another range's (hearth/docs/design/domain/store/records.md, 6), is no error of the answer: nothing renders, and the client says so at startup (hearth/docs/design/client/README.md, 3.2) |

## 3. Printed

```
hosts/west/west-prod.json:4:5: west-prod: vpn: undeclared setting "adress"; vpn declares address, endpoint, hub, port, public_key
tasks/mail/tree/etc/postfix/main.cf.j2:60:42: mail: exports.mail is not in mail's reads
tasks/hardening/task.json:9:7: east-prod: /etc/ssh/sshd_config.d/local.conf is given by hardening and by ssh (tasks/ssh/tree/etc/ssh/sshd_config.d/local.conf.j2)
tasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware
tasks/users/task.json:14:9: east-prod: users: the key "ops" is made twice (tasks/users/task.json:9:9)
```

- **An undeclared setting lists the declared ones,** so an agent fixes a
  typo in one step; a task with none says so, `motd declares no
  settings`.
- **`also` prints as a place,** in parentheses after the message:
  `path:line:column`, or the path alone when the place is a whole file,
  as a file of the tree is.
- **What an error echoes is quoted:** a key or a value from the revision
  prints between `"`, with `\"`, `\\` and `\n` for those bytes and
  `\xNN` for every other control byte, so an error is one line whatever
  the revision holds.
- **What would fail later fails here.** Every path, key and line that a
  build's scan, the activator or the boot would refuse, and that render
  can see in the declarations, render refuses first, at its line:

  ```
  tasks/serial/task.json:12:7: east-prod: boot: "rootwait" is reserved: tiny-initramfs would wait for ever for a missing root (../machine/boot.md, 3.2)
  tasks/web/task.json:30:9: mail-prod: web: /etc/ssl/private/mx1.key is a secret's path; declare it under secrets (../../protocol/format/README.md, 7.1)
  ```
- **The errors left out are counted:** the text prints `<n> more errors`
  (`1 more error`) after the last one kept, and JSON's `outcome` record
  carries `left_out` (hearth/docs/design/client/README.md, 3.2).
