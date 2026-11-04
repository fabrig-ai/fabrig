# macos-box-start: the box starts on macOS

Follow-up to `macos-cheap`, under `temper/docs/design/draft/macos.md`'s rule, the user's: **"make sure that anything we do is relatively cheap"**. Linux-only things may be degraded on macOS; rough edges are fine.

## The red

The macOS workflow's first run (https://github.com/fabrig-ai/fabrig/actions/runs/41920647283, on `028201d75c`) proved a lot: every target compiled in 3m57s, `temper forge fetch` took both bottles through ghcr.io, and both bottle binaries run standalone. The story failed after 0.126 s, at `temper/crates/temper/tests/process.rs:552`:

```text
temper box: LISTEN_FDS: fd 4: errno 42
```

exit 1, right after "First run: wrote the configuration". It never reached Forgejo.

## What's known

- Errno 42 on Darwin is `ENOPROTOOPT`. The message is `ListenersRefusal::Kernel`, from `listener_at` (`skein/crates/skein-shell/src/listener_calls.rs`), worded by `temper/crates/temper/src/lib.rs`'s `listeners_refusal`, which says `LISTEN_FDS` for any fd.
- **Fd 4 is probably not an inherited listener.** `run_configuration` (`temper/crates/temper/src/box_command.rs`, around line 47) also calls `listener_at` on `held`, descriptors the box holds itself. So find what `held`'s fd 4 is, and how it was made.
- `inspect` makes `fstat`, then `getsockopt` for `SO_TYPE` and `SO_ACCEPTCONN`, then `bound_addr`. skein's listener tests ran green on macos-26 on 10-23, so `listener_at` on a plain listening TCP socket works there. So what differs here is the socket: its family, its options, or the way it was made or passed, for example across an exec.

## The task

1. Find the cause by reading. No builder runs macOS.
2. Fix it cheaply. A degraded check on macOS is fine: for example, if the check isn't needed there, `cfg` it or accept `ENOPROTOOPT` for that one socket and say why in a comment.
3. Make the refusal name the step that failed, if it is cheap to do, so that the next red on macOS says more. And don't say `LISTEN_FDS` for a descriptor the box holds itself.
4. Linux stays green.

Say in the report what the cause is believed to be and how sure you are: nothing here can be run on a Mac before landing.

## Proving it

The workflow can run on a branch by hand (`workflow_dispatch`). fabrig/draft dispatches it, so report the branch name, and the trunk stays open for what the run shows. If the run shows the next failure in the story, it joins this plan as a further small task. Step 5 of `macos.md`, section 6, is the bar: the story green.

skein's interface here is `skein-shell`, which hearth uses, so keep any change to its public refusal additive and name it in `plan.md`.

## Size

One or two tasks. One cut is at most 30 minutes' worth of reading.
