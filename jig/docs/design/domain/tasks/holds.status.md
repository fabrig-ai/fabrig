# holds: status

2026-11-22. Where the tree's code lags or departs from [holds.md](holds.md).

No gap but those below.

- **Not yet:** 3, *a restart sees no writer without its live task*:
  temper's forge release lets a delegate retire while a hold still names
  it as its branch's writer, and a start's links then refuse the hold; a
  store that already holds such a writer opens with the writer cleared
  and a fresh read due [in flight: change 11907].
