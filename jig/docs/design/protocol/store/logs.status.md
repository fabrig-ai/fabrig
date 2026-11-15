# logs: status

2026-11-15. Where the tree's code lags or departs from [logs.md](logs.md).

- **Not yet:** 1, 2 and 4: the component gives its store no log rule; no `keys::logged`, no `HistoryLog` row, no census names for `00 09` to `00 0c`; no read reads a task's history log or runs again after a fetch, and the component has no `Limits::fetch`. [in flight: change VIEW]
- **Not yet:** 3, 6 and 8: no commit writes a history record or moves a row out of the map, no `Write::DropHistory`, no job `task-logs`, and log-held value types' layouts retire as any others. [in flight: change WRITES]
- **Not yet:** 1 and 5: no turns log, `TurnLog` or `LogAttempt` row; every turn goes to its task's transcript file, and `Write::DropTranscript` drops the file alone. [in flight: change TURNS]
- **Not yet:** 7: a new generation's copy refuses a directory in `A`'s `store/`, links no task log, and a dropped generation or stray is removed without its `logs/`. [in flight: change GENS]
- **Not yet:** 9: the retention walks erase a task's log-held rows one by one, issue no `Write::DropHistory`, and a project's deletion finds its ended tasks by walking their records. [in flight: change DOMAIN]
