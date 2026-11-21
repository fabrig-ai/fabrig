# The checkout's io: the sweep

How the git component, `temper-worker-protocol-git`, empties `trash`,
where `Make` sets aside what it replaces (`git.md`, 4.1). Moved from
`git.md`, 4.1, on 2026-11-21.

## 1. The sweep

**The sweep** empties `trash`, which io cannot do in one operation (io
never removes a tree: skein's `runtime/io/files.md`). While `trash` is not
empty and the worker is not shutting down, the component runs one file
operation at a time:

- it lists a page of `trash` (`Scan`, within `sweep_entries` and
  `sweep_bytes`) and takes the first entry;
- it tries to remove the entry as a file; one that is a directory it
  lists a page of, and each child it removes as a file, or, if the child
  is a directory, renames to `trash/<n>`;
- it removes a directory once its listing is empty.

Every directory it lists is at most two levels below the worker's root,
so the sweep holds one page, whatever the depth of the tree.
Nothing is followed through a link. What is left at shutdown is swept
after the next start. A sweep's failure is counted and the entry is left
for the next pass, which starts at the next `Make` or the next start.
