# temper ctl landing

Kind: reference

Landings: read one or a project's landings.

## landing show

```
landing show TASK [--project PROJECT]
  TASK (task): a number from 1 to 18446744073709551615; required. The landing task's number
  --project PROJECT: a project's name or number; optional. Its project; the task's when absent
Read a landing: its members, round, search, gate proofs and the turn
Prints: `landing TASK`, its project and step, round and search bounds, each member's prefix, gates and outcome, and the turn
--json: the record landing.
Key: none.
A landing lands its changes by turn: each gate is proved by a receipt
of its key, then main is fast-forwarded to each candidate. A task with
no landing fails, `no landing for task TASK` (exit 2).
Queue changes have no landing task; read them with `change show TASK`.
```

## landing list

```
landing list --project PROJECT [--ended]
  --project PROJECT: a project's name or number; required. The project
  --ended: a switch. Also the landings that ended
List a project's landings, live first, then newest first
Prints: `landings of project PROJECT`, the turn, and a line per landing: its task, phase, changes and repositories
--json: the record landings.
Key: none.
Without --ended, the live landings alone. `landing show TASK` reads one.
These are turn landings. Queue merges create no landing task and do not
appear here, including with --ended. Read a queue change with `change show TASK`.
```

## landing history

```
landing history --project PROJECT [--change TASK] [--landing TASK] [--since TIME] [--until TIME] [--after CURSOR]
  --project PROJECT: a project's name or number; required. The project
  --change TASK: a number from 1 to 18446744073709551615; optional. Only this change; not with --landing
  --landing TASK: a number from 1 to 18446744073709551615; optional. Only this landing; not with --change
  --since TIME: text, at most 1048576 bytes; optional. At or after this UTC minute, YYYY-MM-DDTHH:MMZ
  --until TIME: text, at most 1048576 bytes; optional. Before this UTC minute, YYYY-MM-DDTHH:MMZ
  --after CURSOR: a cursor, as a read prints it; optional. Continue with the 17-byte cursor, in lowercase hexadecimal
Read a page of a project's landing history, in event order
Prints: The project's upper bound and whole-since time, an event per line, then the next command or end
--json: the record history.
Key: none.
Reads at most 32 events, including events whose change record has gone.
--since and --until filter time, not a wait; the range is half-open.
A cursor fixes the first page's upper bound. A swept cursor is refused
(exit 2), naming the first event that remains and the read again.
--json prints every event field, full commits and the next cursor.
```
