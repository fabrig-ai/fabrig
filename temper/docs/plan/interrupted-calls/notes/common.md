# interrupted-calls: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `skein/AGENTS.md`. The task stays in skein, in `skein-shell`.
- **The design is written:** `skein/docs/design/shell.md`, 7.4 states the rule (every
  interrupted operation but the eight transfers completes `Other(EINTR)`, made once by the shared
  part and by the system's call) and section 10 no longer holds it open. Never change `shell.md`;
  what the code taught goes in the report.
- **The status files are the task's:** this departs from the implementer's text ("Not yours:
  any `docs/design/`"). The task edits `skein/docs/design/shell.status.md` and
  `skein/docs/design/STATUS.md` exactly as its spec says, and no other design file. A status
  file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), it holds only the gaps
  as they stand after the change, never history.
- **The transfers stay as they are:** `PipeWrite`, `Write`, `Append`, `Send`, `PipeRead`,
  `Read`, `Recv` and `ReadSignal` are still reissued (7.4), in `attempt` and in each system's
  call. So are `Open`'s `EAGAIN` rule and every call that sets up or inspects around an
  operation's own (`setsockopt`, `getsockopt`, `getsockname`, `fcntl`, an `Open`'s check of what
  it opened, a `Spawn`'s steps).
- **Callers outside the kernel keep their behaviour:** a public function of `skein-shell` that
  shares a call the task changes (`open_listener`, `poll_child`) makes it again on `EINTR`, as
  today.
- **Both systems:** Darwin's code is built here, never run (`shell.md`, 9, the Darwin check).
  Check it with `heavy cargo clippy -p skein-shell --all-targets --target aarch64-apple-darwin
  -- -D warnings`. If that target is not installed in the slot or heavy refuses it, say so
  under `## Left` with the command's output; it is not yours to install.
- Before pushing: `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root shows no new broken citation.
- One commit for the code and its tests, one for the status files, each saying why. Push your
  branch.
