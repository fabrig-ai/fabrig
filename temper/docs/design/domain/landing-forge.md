# Landings at the forge

## 1. What the forge shows

- **The round's pull request,** the landing's own: from the top's
  branch into the train's branch, titled `Land #N: TITLE`, N and TITLE
  the member's pull request's number and title, for a stack of one, and
  `Land K changes: #N1, #N2, …` for a stack of K, the numbers in its
  order, as many as the title's bound holds whole; its body naming the
  round and its base, then a line per member in the stack's order: its
  change, pull request, head and prefix, then, after a search, the
  search's line (`landing-search.md`, 8). It is merged by the turn, or
  closed as its round ends unlanded or its search cuts the stack.
- **Each gate as a commit status on the candidate,** and on each prefix
  a search probed,
  `temper/gate/<name>`: pending with its key's first 16 hexadecimal
  digits once dispatched; success or failure with the run as its link
  once its receipt is made; success with the receipt's run as its link
  when a receipt held with no run. A fast-forward makes the branch's
  commit the candidate, so the branch shows the statuses that proved
  it, and protection may require them.
- **A merged member's pull request is marked merged** by the forge's
  detection of a manual merge (Forgejo's "autodetect manual merge",
  which adoption reads, `landing-settings.md`, 1.2): its head is in the branch, its merge
  commit its prefix. temper writes nothing to it at the turn.
- **A squashed member's is recorded merged** by the landing at its
  squash commit (`landing-turn.md`, section 1), Forgejo's "manually merged" with that
  commit, which needs the repository to allow a manual merge (`landing-settings.md`, 1.2).

## 2. Open questions

- **Reviews in a landing:** before forming (the default here), or on
  the candidate.
- **A judgement gate's key** as a patch id of the member's diff, which a
  merge over conflicting lines would keep and a clean merge (the
  mechanism here) may not; kept as clean merges until a project needs
  more.
- **A gate for a goal's branch's train:** none here (`landing-proof.md`, 2.1); a project
  that wants one names it when one does.
- **A merge raced by a push** waits out the client's tries before its
  round ends (`landing-turn.md`, section 1), its pull request's lane holding the round's
  close behind it. A typed answer for Forgejo's `500`, or a pending merge
  withdrawn once its round is stale, would end it at once.
