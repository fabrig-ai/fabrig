# processes: status

2026-11-12. Where the tree's code lags or departs from [processes.md](processes.md).

- **Not yet:**  the metadata and filesystem operations remain `files.status.md` (Not yet, the whole document). the whole document:
  cgroups and contained trees.
- **Not yet:** 5: a signal's reason: neither io's `Request::Signal` nor
  the kernel's `Op::Signal` carries one, and io's close sends its kill
  unnamed. [in flight: change 4906]
