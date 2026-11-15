# logs: status

2026-11-15. Where the tree's code lags or departs from [logs.md](logs.md).

- **Not yet:** 1, 2 and 4: the component gives its store no log rule; no `keys::logged`, no `logs` ids, no history record codec, no `HistoryLog`, `TurnLog` or `LogAttempt` row, no census names for `00 09` to `00 0c`; no read reads a task's history log or runs again after a fetch, and the component has no `Limits::fetch`. [in flight: change 3547]
- **Not yet:** 3, 6 and 8: no commit writes a history record or moves a row out of the map, no `Write::DropHistory`, no job `task-logs`, and log-held value types' layouts retire as any others. [in flight: change 3549]
- **Not yet:** 5: no turns log; every turn goes to its task's transcript file, and `Write::DropTranscript` drops the file alone. [in flight: change 3550]
- **Not yet:** 7: a new generation's copy refuses a directory in `A`'s `store/`, links no task log, and a dropped generation or stray is removed without its `logs/`. [in flight: change 3545]
- **Not yet:** 3 and 9: no `Write::DropHistory`; the retention walks and a project's deletion drop no history log, and the deletion finds its ended tasks by walking their records. [in flight: change 3548]
