# bench-without-sqlite: smith-bench finds a Codex run's threads in its rollouts, not in SQLite

`smith-bench` links `rusqlite` with its `bundled` feature, whose build script compiles SQLite's C source, for one reader. `smith/benchmarks/src/agent/codex/rollouts.rs` opens the attempt's `CODEX_HOME/state_5.sqlite` read-only and asks it two things: each thread's children (`thread_spawn_edges`) and each thread's rollout file (`threads.rollout_path`). Nothing else in the workspace uses `rusqlite`. Yet `smith-bench` is a workspace member, so every build of the workspace compiles SQLite. The rollouts already hold most of what the database gives. Each rollout's `session_meta` names its thread and, for a child, its parent (`source.subagent.thread_spawn.parent_thread_id`), which the reader already checks. And each attempt's `CODEX_HOME` is its own, so every rollout in it is the attempt's (`smith/docs/design/benchmarks/README.md`, 6, its prepare and collect rows). The user, 2026-11-12: drop `rusqlite` this way, and if it doesn't work, run `sqlite3` as a subprocess.

## What's wanted, ranked. Drop from the bottom if it passes three tasks, and say what you dropped

1. **The reader finds the threads from the rollouts, with today's results.**
   - **The threads known today stay known:** the root, the children its stream's `spawn_agent` calls name (`smith/benchmarks/src/agent/codex/prepare.rs`), and the children the rollouts link to a thread already known. A known thread with no rollout, or whose rollout names no known parent, is an unavailable scope, as today. The total stays a lower bound wherever one is (`smith/docs/design/benchmarks/README.md`, 8.2).
   - **The files are the attempt's rollouts:** every regular file under `CODEX_HOME/sessions/` and `CODEX_HOME/archived_sessions/`, whatever its name, canonical and inside those two directories, as the reader accepts today. A symbolic link out of them is refused, not followed. A thread's file is the one whose `session_meta` names it, and two files naming one thread are refused.
   - **Discovery is bounded on its own:** a count of files and a count of bytes read from each file's opening, apart from today's bounds. Today's bounds (`MAX_THREADS`, `MAX_BYTES`) still apply to the threads and files selected.
   - **A file whose opening isn't valid metadata is unrelated and skipped,** as today's collector ignores an unrelated file. A selected thread's malformed, repeated or conflicting metadata is refused as today.
   - **If a child can be known only from the database,** a case the rollouts and the stream can't show, say so: that is item 3's case.

   Say how you checked it: today's tests in `smith/benchmarks/tests/suite/codex_adapter.rs`, their fixture databases turned into fixture rollout trees, passing unchanged in what they assert, the unlinked child and the lower-bound total among them. Add cases for an archived rollout, an unrelated malformed file and a link out of the directories.
2. **`rusqlite` leaves the workspace.** Neither `smith-bench` nor its tests depend on it. `Cargo.lock` loses `rusqlite`, `libsqlite3-sys` and whatever only they brought. Say what left.
3. **Only if item 1 can't hold, `sqlite3` as a subprocess.** It applies when codex-cli 0.160.0's rollouts lack something the database has, for a case the benchmark design needs. Say which case. Then the reader asks the database through the `sqlite3` program:
   - spawned directly, its arguments as a list and no shell, with `-init` naming an empty file so that no `~/.sqliterc` runs, and `-readonly`, `-bail` and `-json`;
   - the same two queries, each thread id checked as a well-formed id before it reaches the query's text;
   - its output read as a stream within the same bounds, the child killed past them or past a deadline, its standard error bounded, and a nonzero exit a fault;
   - a missing program treated as today's missing database, every child unlinked, with a fault that names the program;
   - within the benchmark's isolation (`smith/docs/design/benchmarks/README.md`, 10).

   The tests use a fake `sqlite3` on their `PATH` or fixture rollouts, so the focused suite needs no `sqlite3` on a builder. `sqlite3` becomes a prerequisite of the live runs alone, which the benchmark design names, and which the machines that run them install. `rusqlite` still leaves (item 2).
4. **The design says so:** `smith/docs/design/benchmarks/README.md`, 6's collect row says where the threads and their files come from, and section 14's open question on Codex's later versions names what a new pin checks.

## Not in this plan

Other agents' collectors. What the benchmarks measure. Codex versions other than the pinned one.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
