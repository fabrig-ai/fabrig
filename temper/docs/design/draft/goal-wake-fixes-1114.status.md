# goal-wake-fixes-1114: status

2026-11-14. Where the design lags or departs from [goal-wake-fixes-1114.md](goal-wake-fixes-1114.md).

- **Design lags:** none.
- **Design departs:** 2: a repair that changes nothing checks its head again only for a CI repair; a gate's, a review's, a person's request's, a landing's, a resolution's or an ejection's repair finding nothing to do holds the change as before, since checking the same head again would only ask the next repair (`../domain/forge/change/repairs.md`, 1, *A repair of another reason that changes nothing*). One job runs again for each such repair, not every failed job at once (`repairs.md`, 1, *Every failed job at once*). Where no job can run again and the head has its base's tip (a GitHub forge, whose adapter refuses a re-run), the red stands and the next repair is asked, counted, rather than holding at once. The goal's designer, 2026-11-14, keeping the draft's intent.
- **Open:** 1: which exemptions of jig's "A question left open wakes it again" (jig's `domain/tasks/lifecycle.md`, 2) go: the designer recommends dropping the timer and the task watch, keeping a pending proposal and a live person delegate; asked of the user, 2026-11-14.
