# The QA loop, first milestone

**Retired lab**, 2026-10-27: temper's experiments and lab binding were
retired for `../qa.md`'s process. This draft is kept as historical design.

**Parked** with the lab (the retired lab binding), 2026-10-26. QA after landing is
agents' `docs/design/qa-flow.md`, with temper's binding in `../qa.md`.

Draft, 2026-10-20, revised 2026-10-23. Agents use temper end to end, as
processes, through its agents' tools, and report what they find in
records of a fixed shape. It records a session with the user on
2026-10-20. The loop runs by hand for now, but its records, its runner
and its roles are the lab's (`factory.md`, section 8), so that they move
into the lab application later and inform its design; `qa.md` is the
inspiration. This draft covers the loop's mechanics and its first
milestone, M1: the loop's process and tooling, proven on the simplest
story, on fake backends. M2 (a real forge, the fake LLM) and M3 (real
forges, real LLMs) are drafts of their own: M3 begins on ChatGPT and an
owned Forgejo in `qa-live.md`, which takes M2's real forge with it; the
mapping of the owners' process onto temper is `temper-on-temper.md`.

## 1. In one page

- **What it explores:** temper's model end to end, on real processes: a
  person asks a chat for a change, and it lands. It does not test
  `temper box` or the first run, which are QA subjects of their own
  (section 2).
- **The lab's names, by hand:** an experiment is data in the product's
  repository; a run executes it with a seed; the runs of a key are its
  evidence, judged into a verdict; a failure with a signature is a
  finding (section 3).
- **The cluster is a list of processes,** not the box: M1's are the fake
  LLM, the fake forge, and `temper serve`. A later cluster
  adds an engine alone and workers by adding processes (section 4),
  as the retained cluster fixture does (`../protocol/README.md`, section 15).
- **A driver agent** (Codex) follows the experiment's charter through
  `temper ctl`, every call recorded (section 5).
- **Checks** read the cluster from outside: stops and panics, temper's
  promises on the fake forge, the story's end state, and the agent's
  report of what confused it (section 6).
- **Every finding ends lower down:** its fix carries a test in the
  lowest world that owns the behaviour; the experiment that found it is
  run again before it closes (section 7).
- **Generic tooling in skein, the product's data in temper:** the
  runner in the retired lab runner, naming no product; experiments in
  temper's retired lab; runs and findings in the retired lab's run directory
  (section 8).
- **On demand,** a campaign at a time, by a lab owner (section 9).
- **M1's bar:** the process and the tooling work: one story, as simple
  as it can be, passes on several seeds, and a planted fault comes out
  as a classed finding (section 10).
- **Before it:** the fakes as processes, the fake git program among
  them, reaching the fake forge across processes; real git comes with
  M2's real forge (section 11).

## 2. What it is for, and what not

- **For:** temper's underlying model, end to end, on real processes:
  the binary's startup and configuration, the engine and the worker
  together, git as a program the worker spawns (real git from M2), the
  agents' tools as an agent meets them, the
  forge's deliveries and polls across a network, and the loop from a
  request to a landing. What the worlds reach only in parts.
- **For the tools' use:** the tools are the agents' interface; an agent
  that cannot see how to do a step is a finding of the tools' or their
  guide's (`../client/tools.md`, section 2).
- **Not the box,** nor the first run: `temper box` and its steps would be
  an experiment of their own, checking the box's own work, not the
  system beneath it. The first run's suggested requests inspired M1's
  story (section 10); M1 does not check the first run.
- **Not domain corner cases,** which the worlds own
  (`testing-strategy.md`; `../domain/testing.md`). An experiment that keeps
  finding domain bugs says a world lacks stories.
- **Not documentation, yet.** Driver agents may read temper's code and
  documents in M1. Agents limited to the user-facing documents, whose
  findings would also shape those documents, come later (section 12).

## 3. The records

The names are `factory.md`'s, 8.1. Each record is a file; the lab
would keep them as its own state.

- **Experiment:** a directory in the product's repository, one per
  story:
  - *the cluster:* its processes and their configuration, templated on
    the addresses and directories a run gives (section 4);
  - *the seed state:* the fake forge's repositories and accounts, and
    the fake LLM's scripts, one per conversation the story makes;
  - *the workload:* the driver agent's charter, in prose: the persona,
    the goal, what it may and may not do, what to report;
  - *the checks:* the story's end state, beside the generic ones
    (section 6);
  - *a cost class:* how long a run may take and what it may spend.
- **Subject:** the digests the experiment uses: the product's build (by
  M1, the head's commit and the built binaries' digests) and the fakes'.
- **Key:** the experiment's digest and its subject's. Two runs of one
  key differ only by their seed.
- **Run:** one execution of a key with a seed: provision, drive,
  collect, check. Its record holds the seed, the transcript of every
  tool call, each process's stderr and exit, the fake forge's
  observations, each check's result, the agent's report, and the time
  and spend.
- **Evidence:** the runs of a key. It only grows.
- **Verdict:** the evidence judged by a policy: pass, fail, flaky, or
  not enough evidence. M1's policy: three seeds; all pass is pass, all
  fail is fail, mixed is flaky.
- **Finding:** a failure with a signature (section 7).

The seed draws what varies between runs: the fake LLM's latency and
injected failures (skein's `fake-llm.md`, section 2), the fake forge's
faults and delays, and the order the runner starts independent
processes in. A run is reproduced from its key and its seed, as far as
real processes allow; how far is part of what M1 learns.

## 4. The cluster

A list of processes, each:

- **its command:** argv and environment, templated on the run's
  directories and the addresses the runner allocates;
- **its files:** documents written before it starts, from the
  experiment's templates;
- **its readiness:** a line on its output, or a port that accepts;
- **its dependencies:** the processes that must be ready first;
- **its seed actions:** commands the runner runs once it is ready, such
  as `temper ctl` adding the fake LLM's key and adopting the fake
  repository.

The runner starts them in dependency order, each on loopback in the
run's scratch directory, keeps their output, stops them in reverse
order, and records each exit. A process that ends early is a check's
failure (section 6), not the runner's.

M1's cluster:

```
run R, seed S, scratch directory D
├── fake-llm      temper's fake LLM program over skein's peer, on loopback (TLS with skein's test root), the experiment's scripts
├── fake-forge    temper's fake forge program over skein's peer, on loopback: its API, deliveries, and git's smart HTTP (section 11)
├── temper        temper serve, on D's directories, its configuration naming fake-llm and fake-forge
└── seed          temper ctl: the LLM's key added, fake/widget adopted into a project
```

The runner knows processes, not temper: an engine alone and a worker or
several are more entries, as the retained cluster fixture has them, each a `temper
serve` of its own (`../protocol/README.md`, section 15), and a
real Forgejo (M2) replaces `fake-forge` with skein's throwaway Forgejo.

## 5. The driver

- **A Codex agent** per run, started by the runner, given the charter,
  the run's `temper ctl` directory and the agents' guide
  (`../client/tools.md`, 11.5).
- **The tools are its only hand on temper.** It reaches temper through a
  wrapper of `temper ctl` that records each call: its argv, its output,
  its status and its time. The transcript is the run's, and a finding's
  reproduction (section 7).
- **It reports,** at its end, in a fixed shape: done or not; for each
  step that took more than one try, what it tried and why; anything that
  surprised it.
- **It never edits** the product's repository or the run's processes'
  files; the runner gives it a working directory of its own.
- **Bounded:** the cost class's time and turns. Past them the run ends
  as not done.

## 6. Checks

What says a run failed, beyond the agent thinking so (`qa.md`, 4.3):

- **A stop:** any process that exits before the runner stops it, or a
  panic in any stderr.
- **temper's promises,** seen on the fake forge's observations, as the
  worlds' referee sees them: two creations under one key, a forced push,
  a write to an object temper does not own, a merge at a head its
  decision did not see.
- **The story's end state,** from the experiment: read through
  `temper ctl` and the fake forge's API (for M1, the change landed on
  `fake/widget`'s branch, with the content the script asked for).
- **Bounds:** the cost class's time, and each process's memory against
  its configured bound.
- **The agent's report:** not done, or a step it could not see how to
  take, is a failure of its own kind: use.

## 7. Findings

- **A signature,** for deduplication: a panic's message and place, a
  promise and its subject, a check and its end state, or, for use, the
  triager's judgement against open findings.
- **A reproduction:** the key, the seed, the transcript.
- **Reproduced** on more seeds before it is filed, and marked flaky or
  not.
- **A class,** where its fix goes: a world lacks a story; a fake differs
  from its system; a bug below the domain (protocol, io, the binary);
  the tools or their guide; the experiment itself.
- **A state:** open, taken (by whom), fixed at a commit, checked again,
  closed; reopened if it returns.
- **Its fix carries a test in the lowest world that owns the
  behaviour,** never in the lab: a world story, a pinned seed, a fake's
  correction, a protocol test; a transcript becomes a world's story as
  `../client/tools.md`, 11.6 says. The lab only checks it again, by running
  the experiment's key at the fixing commit.

## 8. Where it lives

- **The runner:** the retired lab runner. It reads an experiment
  directory and the run's settings, provisions the cluster, starts the
  driver, collects, runs the checks, and writes the run's record. It
  names no product; each experiment's templates and checks carry what
  is temper's.
- **The experiments:** temper's retired experiments, beside the code
  they test, so a head's experiments are the head's.
- **Runs, evidence and findings:** the retired lab's run directory, the machine's
  working files, one directory per experiment and key; findings one
  file each, with an index.
- **Generic checks** (stops, panics, bounds) are the runner's; the
  forge's promises are a check skein's fake forge provides; the story's
  end state is the experiment's.

## 9. The process

- **A campaign:** a set of experiments run at one head, on demand, by a
  lab owner the director starts (`owners.md`), on Claude for the calls
  that need taste.
- **The lab owner** writes and curates the experiments, judges the
  verdicts, classes findings that are not mechanical, and decides what
  each campaign runs. Writing an experiment's scripts and checks is
  Codex's, from the owner's sketch.
- **Codex** runs the driver agents and the triager, which deduplicates
  new failures against open findings and proposes their class.
- **Findings reach owners** through the director: a finding becomes
  work for the owner of the area it names, or a small fix the lab
  owner's Codex makes under its own landing. Fixes land through `land`
  as any.
- **Closing a campaign:** a short note: verdicts, findings opened and
  closed, cost, and what the runner or the records lacked.
- **owners.md** gains a section on the lab once the loop has run.

## 10. M1

M1's goal is the loop's process and tooling, and a first validation
of temper end to end: the story is as simple as it can be, and more
stories wait until the loop works.

**One story,** on one fake repository, `fake/widget`, inspired by the
first run's suggested requests: a chat is asked for a one-file change;
the change is proposed, its branch pushed, its CI passes, and it lands.
It stays within the tools' area 1, a chat to a landed change
(`../client/tools.md`, 5.5). Its exact script is the lab owner's.

**A planted fault:** an experiment whose cluster kills `temper serve`
mid-run, or whose fake forge is set to break one of temper's promises,
so that a finding goes through every step: signature, reproduction,
class, and a fix checked again.

**Its bar:**

- the story's verdict is pass, on three seeds, from one command;
- the planted fault gives one finding, classed, reproduced from its key
  and seed;
- every finding of the campaign is fixed with a lower test, or filed;
- the closing note says what the lab application should take from the
  records and the runner, and what to change in them.

## 11. Before M1

1. **The fakes as processes,** each a few lines of `main` in the
   consumer, over skein's libraries: skein ships the fake LLM peer and
   the fake forge peer as libraries that host on loopback, as now, and
   no binary. temper adds its two programs (for example `temper fake
   llm` and `temper fake forge`), with what it needs of its own: its
   seed, its scripts' files, its configuration's addresses. The fake
   forge serves its API, its deliveries and its observations. This is
   the piece of `first-run.md`'s 3.5 that moves the fakes into product
   crates, pulled forward.
2. **The fake git program across processes,** so that `temper
   serve`'s worker spawns it by the absolute path `workers[].git`
   names, as it spawns real git:
   - **a program of its own,** for example `temper fake git`, over the
     fake git program's step core, which the real loop already runs on
     the real file system (`tests/real-loop`'s hosted fake git); its
     working trees keep the fake checkout's `.git`;
   - **its transport over the network:** the typed calls of skein's
     `fake-forge.md`, section 3, encoded, to a private route on the
     fake forge's listener, found from the remote URL already on git's
     command line, with the token from its `http.extraHeader`;
   - **one store:** the fake forge's domain stays the only state, with
     its commits' counts, policies, faults and observations, so the
     checks of section 6 read it as the worlds' referee does.

   Not real git: M1's agent is the fake LLM, whose commands are the
   shell subset, and nothing but the git component reads a working
   tree's git. Real git's own behaviour is the transcripts' and the
   process tier's (`../protocol/git.md`, section 9), and the loop's
   from M2, whose real forge serves it. The real-git bridge stays later,
   for real agents on the fake forge (section 12). Its CI stays by cue
   (`../domain/testing.md`, 2.4).
3. **The runner,** with the records' formats.
4. **M1's experiments,** and the first campaign.

## 12. Later

- **M2:** a real forge with the fake LLM: skein's throwaway Forgejo,
  then GitHub, and with it real git in the loop. Its first part is in
  `qa-live.md`: the owned Forgejo set up by `ctl` with no LLM, and a
  real failure's transcript replayed by the fake LLM on it (its 3.1 and
  section 8).
- **M3:** real forges and real LLMs, ChatGPT then Claude; verdicts
  become rates over many runs. Its first part is `qa-live.md`: a ladder
  of the fakes' roadmap's stories on ChatGPT and an owned Forgejo.
- **More stories,** growing with the tools' areas 2 to 4, until
  temper-on-temper (`temper-on-temper.md`).
- **Driver agents on documents alone:** no code, only the user-facing
  documents and the tools' guide, so their findings shape those
  documents too; a later milestone.
- **Explorers** that write experiments of their own, aimed at what
  landed.
- **The cluster grown:** an engine alone and workers, the retained cluster fixture
  (`../protocol/README.md`, section 15), once a driver runs it.
- **Real agents on the fake forge:** a real LLM's agent with the fake
  forge's seeded faults, cheaper and more repeatable than M3's real
  forges. Such an agent runs git in its working tree (`git status`,
  `git diff`, `git log`), which a fake checkout's `.git` cannot answer,
  so it needs real git against the fake forge: the real-git bridge
  (real-loop's `later.md`).
- **The box's own experiments,** apart from these.
- **Into the lab application:** its records as its state, the runner as
  its run procedure, the driver and the triager as its agents
  (`factory.md`, 8.3 and 8.4).

## 13. Open questions

- **Reproducing on real processes:** how much a seed fixes once real
  scheduling, and from M2 real git, are in the run; whether M1 records enough to
  replay a failure that does not come back.
- **The records' format:** JSON on skein-json, or Markdown with a JSON
  head; and how the runner's checks are written per experiment
  (commands with an exit status, or a small vocabulary of its own).
- **Where findings go after M1:** files under `tmp/`, issues on temper's
  forge, or both.
