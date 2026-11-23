# Requests through ctl

This document holds request delivery through ctl, split from [`README.md`](README.md).

## 1. Requests: once, and named

- **Through jig's request table.** Every act but one that carries a
  value (below) is a keyed request: `jig-client` keeps it before sending
  it, sends it again under its key until its answer is durable, and
  answers it once (jig's `client/domain/client/requests.md`).
- **The agent names the key** of every tool marked `K`: `--key NAME`, a
  word of 1 to 64 bytes of `a` to `z`, `0` to `9` and `-`, such as
  `health-chat`. A call without it is refused as a usage error, before
  anything is sent. So is `--key` with no name, or a name outside that
  rule, and that refusal says the rule, as `results.md`, 3 has it and the same for argv
  and `--call`: `temper ctl: task message: --key KEY takes 1 to 64
  lower-case letters, digits or hyphens; temper ctl help task message`.
  `temper help ctl` says the rule too.
- **A named key is the engine's key:** the first 16 bytes of the SHA-256
  of `temper-tools`, a zero byte and the name. jig scopes it to the party
  (jig's `client/domain/contract/requests.md`, 1). The same name asked again by
  the same party, within the engine's retention, is the same request: a
  rerun after a lost answer, an agent's own timeout or a lost context
  gets the first answer, never a second chat. The same name with another
  ask is refused by the engine as a key conflict. The name, not
  DIRECTORY, makes a request the same: a rerun from any directory signed
  in as the party is the same request, and DIRECTORY's store keeps a
  request only until it is answered (`command-line.md`, 1.2). Past the engine's retention,
  which `me` prints, the name is free again, and asking under it is a
  new request.
- **A named request is built from the call and `me` alone,** never from
  the clock, so a rerun builds the same ask: a deadline a template gives
  as a span is sent as that span, which the engine makes a wall when it
  makes the task (jig's `client/domain/contract/requests.md`, 2). A rerun
  after `me` changed what the request is built from (a template, a
  bound) is another ask, refused as a key conflict.
- **A name the client holds already** is answered `Held`, nothing sent
  (jig's `client/domain/client/requests.md`). A call sends what
  DIRECTORY kept again before it sends its own (`command-line.md`, 1.2). A call whose name
  a kept request of its party holds, the same ask, is that request: it
  waits for its answer as for its own, and prints it, so that a rerun
  still gets the first answer; one under that name with another ask, or
  under a name another party's parked request holds (`command-line.md`, 1.2), meets
  `Held`, and exits 1.
- **A spent-pool refusal keeps its own figures first:**
  `chat start: refused: your pool in project P had LEFT left, NEEDED needed`,
  in ctl's amount (`results.md`, 1).
  If today's `me` pool still has less than NEEDED, the `read it:` and
  `raise it:` calls follow. Where that pool also holds a reservation
  for live work it funds (its `reserved` above zero), as an earlier
  start's `--as-allowed` leaves it (`areas.md`, area 2), a line before
  them says so and what leaves room: `reserved: R, for the live work
  your pool funds; a goal or milestone started with --as-allowed takes
  all your pool may give: start them with --budget AMOUNT to leave
  room`, R the pool's `reserved`. With room now, the next line is
  `now: your pool has L left; budget B; this key keeps its first answer: start new work with a new --key`.
  Today's pool never replaces the refusal's figures.
- **Names are the party's.** Agents sharing a party give their names a
  prefix of their own; the page `ctl` says so (`testing.md`, 5).
- **Acts that decide name the revision the agent saw;** a moved one is
  refused, and the agent reads again. A decision asked again under its
  key prints its first answer. A retained decision, by the party itself
  or another, answers `decided first:` with exit 0 and changes nothing.
  An unknown proposal answers exit 2: "this proposal isn't waiting; it
  may have been decided or replaced." A task's automatically replaced
  settings proposal keeps no decision to report (jig's
  `client/domain/contract/requests.md`, section 2, and `jig/docs/design/client/domain/contract/README.md`, section 3).
- **The other acts** (`task stop`, `task cancel`, `proposal decide`,
  ...) take `--key` optionally; without it the table draws one, which
  the small store keeps until answered (`command-line.md`, 1.2).
- **A request carrying a value,** `account add-key`'s and `account
  replace-key`'s (`areas.md`, area 5), and area 6's `forge connect` with a FILE
  and `forge replace`, is sent once and never kept: `jig-client` writes
  no record of it to DIRECTORY's store and never sends it again (jig's
  `client/domain/client/requests.md`; jig's `protocol/client/README.md`, section 3, and `protocol/client/small-store.md`, section 1
  ), and the engine takes the value out as it decodes
  the request, so no committed request or answer holds it (jig's
  `domain/accounts.md`, section 5). A forge's pasted value is kept by
  jig's clients' component in the secrets' store under temper's tag, at
  a number jig gives it, before temper's core sees the request; a
  request refused, or a duplicate, has jig erase it
  (`../../protocol/credentials.md`, pasted values). The tool names no
  number. The client's own answers *unsent*
  (the engine did not make it) and *lost* (it may have) exit 1 with
  nothing kept, as a call cut or a link down before its answer does, and
  `pending` lists nothing of it. The agent asks again with the same
  `--key` and the same FILE: within the engine's retention, `account
  add-key` gets the first answer, never a second account, and `forge
  connect` never a second connect. `account replace-key` and `forge
  replace` take `--key` optionally, as the other acts do, but the name
  drawn is not kept, so only a name the agent gives can be asked again;
  a rerun without one replaces the key or the tokens again.
- **An account's uses** (area 5): `account uses ACCOUNT --party N...`
  and `--project N...` set exactly the named parties and projects, at
  least one and at most 16 together, the wire's bound. Either flag may
  repeat or stand alone. `account uses ACCOUNT --default` restores
  jig's defaults (`uses: None`). Both forms take an optional `--key` and
  print `done: uses`. The tool resolves the holder as `account rank`
  does, sends `SetUses` and leaves policy to the engine. A person's
  plan may pay for its holder's services and projects whose other
  members are services; another person, or a project with another
  person, is refused. Provider refusals say that rule in words; an
  unknown object says unknown account, party or project, as the
  refusal distinguishes none of them. `account list` prints each use.
  Naming a service lets its work run on its holder's subscription.
- **An account's move** (`areas.md`, area 5): `account import`'s `move_in` is
  a request carrying a value, as `account add-key`'s is, under its
  `--key`. `account export`'s `move_out` is a request whose answer
  carries one, the record: sent once under a name the table draws,
  never kept and never sent again, since the record comes with the
  first answer alone and the name asked again is answered with none,
  *lost* (jig's `client/domain/contract/requests.md`, 1). So `account export`
  takes no `--key`: an export asked again is a new request, refused
  "moving" until the first move is cancelled at its deadline. Its
  `written` is an ordinary request under a drawn name, kept in
  DIRECTORY until answered (`command-line.md`, 1.2).

## 2. Open questions

None.
