# temper ctl waits

Kind: reference

Read a project's live waits, overdue first.

## waits

```
waits --project PROJECT [--kind KIND] [--party PARTY] [--due WITHIN] [--after CURSOR]
  --project PROJECT: a project's name or number; required. The project
  --kind KIND: one of party, question, dependency, resource, account, host, delegate, news, step, room, turn or sign-off; optional. Only waits of this kind
  --party PARTY: a number; optional. Only waits at or heard by this party or its roles
  --due WITHIN: a duration: seconds, or a number with s, m or h, from 1 to 2592000 seconds; optional. Only waits with bounds due within this duration, including those already past; at most 30 days
  --after CURSOR: a cursor, as a read prints it; optional. The cursor a page printed, to read on with the same filters
Read a project's live waits, overdue first
Prints: The project's waits and the page's time; a line per wait with its task, kind, object, since, bound and who hears, then a continuation if there is another page
--json: the record waits.
Key: none.
Overdue waits come first, then the other bounded waits, then those
without a bound. Within each group, waits are by bound if any,
then by task. Continued pages use the first page's time.
```
