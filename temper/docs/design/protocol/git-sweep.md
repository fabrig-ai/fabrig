# The checkout's io: the sweep

Provisional, 2026-11-21. How the git component, `temper-worker-protocol-git`,
empties `trash`, where `Make` sets aside what it replaces (`git.md`, 4.1),
a start what the last one left (`git.md`, 2), and `Store` a store it
replaces (`git.md`, 4.11). io never removes a tree (skein's
`runtime/io/files.md`), so the sweep removes one entry at a time, in the
background, beside the operations of `git.md`, section 4. Moved from
`git.md`, 4.1, and revised the same day so that an entry it can't remove
no longer stops it: on a deployment, one entry owned by root stopped a
worker's sweep for good, and its disk filled until a person fixed the
entry by hand. Why an entry belongs to another user is the deployment's.

## 1. In one page

- **A pass walks `trash` once,** by its open directory's position: it
  meets each entry once, removes what it can, and empties a directory
  before it removes it.
- **What it can't remove it sets aside** for the rest of the pass, and
  goes on to the next entry: one entry never holds back the rest.
- **Reported once:** an entry set aside is reported with its path and
  the kernel's error, as a line of the worker's, and again only when its
  error changes.
- **Passes run again:** at once after a pass that removed or moved
  something; a minute after one that changed nothing but set something
  aside; otherwise at the next `Make` or the next start.
- **One file operation at a time,** and none once the worker shuts down;
  what is left then is swept after the next start.

## 2. A pass

A pass runs one file operation at a time, beneath the worker's root:

1. **It opens `trash`** (`Open`, a directory, no link followed): the
   pass's cursor, since an open directory keeps where its last listing
   stopped (skein's `runtime/io/files.md`). It lists it one entry at a
   time (`List`, a page of one entry within `sweep_bytes`), each from
   where the last stopped, until a listing comes back empty, and then
   closes it.
2. **Each entry `trash/<e>`** it removes as a file (`Remove`, in the
   layout's `trash`). Removed, the entry is done; `NotFound`, there was
   nothing to remove; `IsADirectory`, it empties the directory (3); any
   other error sets the entry aside (section 3).
3. **A directory `trash/<e>`** it opens (no link followed) and lists by
   pages of `sweep_entries` entries within `sweep_bytes`, each from where
   the last stopped. Each child it removes as a file; a child that is a
   directory it renames to `trash/<n>`, refusing replacement as `Make`
   does (`git.md`, 4.1), so that this pass or the next meets it as an
   entry of `trash`; a child gone (`NotFound`) is nothing. A child whose
   removal or rename fails any other way is set aside, as
   `trash/<e>/<child>`. A page that comes back empty ends the listing: it
   closes the directory and removes `trash/<e>` as a directory. If the
   directory's open, a listing of it, its close or its removal fails,
   `trash/<e>` is set aside.
4. **The pass ends** when `trash`'s listing comes back empty, or when
   `trash` itself can't be opened, listed or closed: then `trash` is set
   aside, as an entry is.

So the sweep holds at most two open directories, `trash` and the one it
empties, with one name of `trash`'s and one page of the other's,
whatever the depth of the tree: every directory it lists is at most two
levels below the worker's root. Nothing is followed through a link. A
pass *changed* `trash` when it removed or renamed anything. A listing may
or may not show an entry added or removed since the last one (skein's
`runtime/io/files.md`), so what a pass's cursor passed over, the next
pass meets (section 4).

## 3. What it can't remove

- **Set aside:** an entry set aside stays where it is for the rest of the
  pass, which meets it no more, its cursor being past it, and goes on
  with the next entry. A directory with a child set aside can't be
  removed: its removal's failure sets it aside too, without a report of
  its own, since its child's says why it stays.
- **Reported:** the component reports an entry set aside as an event of
  its boundary, `Event::Unswept { path: Box<[u8]>, error: kernel::Error }`:
  `path` is the entry's beneath the worker's root, `trash`, `trash/<e>`
  or `trash/<e>/<child>`, at most 517 bytes; `error` is skein-io's kernel
  error of the operation that failed. It is no operation's terminal: each
  operation still ends exactly once (`git.md`, 2).
- **Once per error:** the component keeps the entries it has reported,
  each with its error, across passes, at most 64 of them. An entry set
  aside is reported when they hold none of its path, or hold it with
  another error; they then hold it with this one. At a pass's end they
  drop each entry that pass didn't set aside: it was removed, or is
  gone, and if it comes back and fails it is reported again. An entry
  set aside while they hold 64 others is not reported, until a pass's end
  makes room. They start empty at each start, so a start reports again
  what is still there.
- **The worker's line:** the worker takes each report as one of its notes,
  `worker_alone`'s `Note::Unswept { path, error }`, which its shell writes
  to stderr (`services.md`, 7), as it does a stopped agent's
  (`../domain/worker/README.md`, 4.3):

  ```
  <binary> worker: trash entry not removed: <path>: <error>
  ```

  `<path>` is beneath the worker's root (`workers[].root`), its newlines,
  control bytes and non-ASCII bytes escaped as the other lines' details
  are; `<error>` is the kernel's error by its name, as `Permission` or
  `NotEmpty`. No wire carries it.
- **A shutdown's stop** is no failure: the pass ends, setting nothing
  aside and reporting nothing.

## 4. When a pass runs

- **It starts** once the layout is made at a start (`git.md`, 2), when a
  `Make` ends, succeeded or failed (`git.md`, 4.1), and when its time
  comes (below). One pass runs at a time: a `Make` that ends while a pass
  runs has the next pass start as soon as that one ends, since its cursor
  may be past what the `Make` added.
- **At once again** after a pass that changed `trash` (section 2): what it
  renamed into `trash`, or what was added beside its cursor, may be
  behind it.
- **A minute later** after a pass that changed nothing but set an entry
  aside, `trash` itself among them: the next pass starts 60 seconds after
  that one ended. This is the component's one deadline of its own
  (`git.md`, 8): `Component::next_deadline` gives the earliest of it and
  its operations' deadlines, and `Component::fire` starts the pass once
  it is due. So an entry a person makes removable is swept within a
  minute, and one still failing is tried once a minute and reported only
  when its error changes (section 3).
- **Otherwise** the sweep waits for the next `Make` or the next start: a
  pass that changed nothing and set nothing aside found `trash` empty.
- **At shutdown** no pass starts and no deadline is given; a pass running
  ends at its next step, closing what it holds, and what is left is swept
  after the next start. A pass to come holds nothing open, so it never
  keeps the component from being empty.

## 5. Testing

What checks this document's contract, beside `git.md`, section 9:

- **Step tests** in `temper-worker-protocol-git`, on scripted kernel
  completions: a pass over files, a directory and a nested one, each
  listing from where the last stopped; an entry that can't be removed
  ahead of others that can, the others swept and the one reported once,
  with its path and error; a child that can't be removed or renamed, its
  directory set aside unreported; `trash` that can't be opened; an entry
  failing the same way in a later pass, unreported, and another way,
  reported again; the next pass due a minute after one that changed
  nothing but set an entry aside, its deadline given and fired, and
  sweeping the entry once it can be removed; a pass at once after one that
  changed `trash`; a `Make` ending during a pass; none after shutdown,
  and no deadline then; the reported entries' bound.
- **The worker's line:** `Note::Unswept`'s line, its path escaped, in
  `temper-worker-shell`'s line tests; and the worker taking the
  component's report as that note.
- **The git component's machine world,** `tests/worker/protocol-git`: the
  sweep over deep and wide trees in pages of two; an entry whose removal
  fails ahead of others, the others swept and the one reported, then
  swept by a later pass once it can be removed. Its io lists an open
  directory from where its last listing stopped, as every backend does
  (skein's `runtime/kernel.md`, 6.1).

## 6. Later

- **Who sees it:** a report is a line of the worker's alone. Whether the
  engine should hear of a worker's trash it can't empty, and show it to
  the deployment's people, waits for a deployment that needs it.
- **The minute** between passes is fixed. A figure under `git_component.`
  waits for a deployment that needs another.
