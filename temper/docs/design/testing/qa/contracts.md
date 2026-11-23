# QA's faces and world: contracts

Provisional, 2026-11-23. The shapes the changes of D11 of
`../../draft/slow-tests-down.execution.md` share while they move QA's
tests to the gate (`README.md`, 6): the goal's own, deleted by its last
change (`../../draft/design-economy.md`, 4.3). Where a section names a
shape and not its Rust, the change that makes it writes the Rust, and the
changes after it read that code.

## 1. The faces

In `temper_qa_tool::faces`. Every kernel call the crate makes is a
method of one of them, and only their production types call the kernel.
A method's error is `std::io::Error` as std answers it, so that each
caller keeps its words (`PATH: WHY`), or `Refusal` where the method is
QA's own today.

- **`FilesFace`** replaces `BuildFilesFace`, and `RealFiles` replaces
  `RealBuildFiles`. Its operations:
  - read a file whole, or open it for reading as a stream (`Read`);
  - write a file: replace it (made or truncated), make it new (refusing
    one that exists), or append to it; with an exact mode set on it
    where asked (a private file is 0600 whatever the umask,
    `private::file`'s rule), and synced where asked;
  - rename a path; hard-link a file (`link`, kept: its "crosses
    devices" answer is the caller's to read);
  - make a directory: one, with its parents, or private (each missing
    parent made, the directory itself 0700 exactly even where it was
    there, `private::directory`'s rule);
  - remove a file, a directory with what it holds, or whatever a path
    names, without following a link;
  - list a directory: each entry's name as bytes and its kind;
  - a path's facts, following a link or not: its kind (file, directory,
    link, socket, other), its size, its permission bits, its device, and
    the identity and link count that hard links share;
  - set a path's permission bits exactly;
  - copy a file's bytes to a path;
  - resolve a path: canonical (following links; the path must exist) or
    absolute (lexical, from the working directory);
  - lock a lock file, made 0600 where missing: waiting, or trying,
    answering "held" where it would wait; held until the lock's value is
    dropped;
  - Unix sockets: bind a path (a stale entry replaced, the socket 0600,
    its accepts never waiting), accept (a connection, or none yet),
    connect (a connection, or "absent" for a path missing or refusing);
    a connection reads and writes without waiting (`WouldBlock`) and
    closes when dropped;
  - the build files' checksums, `hash` and `check`, kept: QA's own
    SHA-256 lines over the face's reads (`builds.md`, 3 and 4), as the
    trait's default bodies, which a test's face may override to answer
    them itself (`builds.md`, 3, its sixth step).
- **`Processes`,** kept, adds:
  - `output`: a program run to its exit, stdin closed, with the given
    environment and directory, its stdout and stderr captured, or both
    appended to a log file made 0600; answering its exit and what it
    captured. The drive tool's `recipe` commands and `tar` run through
    it.
  - `version`: an executable's first line of stdout for `--version`,
    within 30 seconds, stdin closed and stderr discarded, its exit
    unread: `build.rs`'s probe today (`builds.md`, 3).
  - `holder`: `temper-qa pass hold`, with `--ctl` where asked, from the
    running executable, in its own process group, stdin closed and both
    streams to the place's `drive.log`; answering its process for
    `poll` and `signal`.
  - `drive`: the drive session's command in its own process group,
    stderr inherited, its stdin and stdout two connections the holder
    writes and reads without waiting; answering its process and the two.
  - `end_holder`: the loop's end of a place's holder, today's
    `hold::end_from_loop`, which goes: connect at once, so that a later
    pass's holder at the same path is never reached; then, off the loop,
    send `end`, read its answer to the end, wait for the place's drive
    lock, and wake the loop. A missing or refusing socket ends nothing.
    The production face keeps today's thread.
- **`ClockFace`,** kept, adds a monotonic reading finer than a second
  and a sleep, for the holder's and its clients' polls; `Moment` and
  `wait` stay as they are.
- **`DriveFace`, `ArchiveFace` and `GitFace`** stay. `RealDrive` and
  `RealArchive` run `temper-client-journeys` and `tar` through
  `Processes::output` and reach their paths through `FilesFace`, so that
  their arguments, a walk's log and the archive's set-aside paths replay
  (`places.md`, 8). `RealGit` stays git's own face.

## 2. What takes them

- **`Home`** stays `Clone` and `Debug`, holding paths and the
  configuration. `open`, `read`, `transact` and `transact_loop` take the
  files face; `transact`'s wake after a save connects to `run.sock`
  through it, and a saved command stays successful when the wake fails.
- **The helpers both halves share** take the files face:
  `private::directory`, `private::file`, `PlacePaths::ensure` and
  `place_busy`.
- **Each entry point takes the faces it runs on:** `runner::Run` its
  files face beside its git, processes and clock; `place::PassPlace` its
  files and processes faces beside its drive and archive;
  `build::keep_with` its files and processes faces;
  `installation::install` its files face; `hold::Drive` and
  `hold::Holder` the files, processes and clock faces.
- **Production faces are made in `cli::main` alone,** once, beside the
  captured arguments and environment, and handed to the command entry
  (section 3); `temper-qa run`'s `Run` takes them from there. Until the
  holder's commands take the entry's faces, their dispatch
  (`cli::drive_command_run`) makes its own.

## 3. The command entry

- **Admission,** `temper_qa_tool::admit`: from the identity (version and
  commit) and the captured arguments, before any effect, either an
  answer, the front's page with exit 0, or its refusal or a command's
  argument refusal with exit 2, each in its words (`README.md`, 2, "Its
  answers"; `../tools.md`, 4.2), or the command to run with its
  admitted arguments. `main` takes admission's answer first.
- **The entry,** `temper_qa_tool::answer`: the faces, the identity, the
  captured arguments and environment, stdin as a reader and stdout and
  stderr as writers; answering the exit: admission's, a command's own
  status (`places.md`, 4 and 5), 0 for a command that ran, or 1 with
  `temper-qa: WHY` on stderr for a refusal, the ledger untouched. Its
  faces are one value, whose shape the entry's change chooses. `main`
  captures, makes the production faces, calls it and exits with its
  answer.

## 4. The world

In `temper_qa_tool::testing` (`#[doc(hidden)]`), compiled once for the
`domain` and `suite` targets:

- **The tree:** a files face in memory, its handles sharing one tree of
  absolute paths; modes, kinds, devices, hard links' identity, locks (a
  test may hold one as another process would), sockets, and failures a
  test injects by path and operation. A path of bytes that is not UTF-8
  is a path like any other.
- **The loop's fakes,** `tests/suite/world.rs`'s today: `Git`, `Range`,
  `Node`, `Agents`, `Agent`, `Clock`, `World`, `Loop`, `baseline`,
  `land` and `id` move here, keeping their names and their scripting,
  over any files face: the tree in `domain`, the production files face
  in a scratch home in `suite`. `World` keeps `new` (its `qa.json` as
  today's), `start`, `start_with_environment`, `ledger`, `build` and
  `installable_build`; `Git`'s worktrees are directories of its files
  face; `Clock` sleeps by moving its time; an agent records through the
  home's transactions over the world's files face.
- **Programs:** a handler a test registers by program name, given the
  command (its arguments, environment and directory) and the tree as it
  stands, answering its exit and its streams and writing what it
  writes, for `Processes::run`, `output` and `version`; the holder and
  the drive session as scripted connections a test feeds and reads.
- **The `suite` witnesses** keep their real calls: their world's files
  face is the production one, and a command with no scripted answer
  runs as a real process, as `Agents::run` falls back today.

## 5. The targets

- **`temper-qa-tool::domain`,** `tests/domain/main.rs`, its line in
  `tools/tiers/layers.toml` at `domain`, its comment naming the QA
  runner's world.
- **A test moved from the tool's `suite`** keeps its module and its name,
  in the file of its module's name under `tests/domain/`, its body and
  assertions kept and its paths the tree's. **One moved from
  `temper-qa::suite`** goes under `cli::` and its module there, keeping
  its name (`cli::builds::commit_form_refuses_before_build_checks_and_accepts_sha256_ids`).
- **`step`** gains `table`, the command table's answers through
  admission, and `tune`, `Ledger::tune`'s figures at a given time.
- **`suite`** keeps the production faces' witnesses (section 6), its
  line's comment naming their real calls; `temper-qa::suite` keeps the
  shipped paths (`README.md`, 6).
- **Shares:** each change writes its own lines of `tests/domain/main.rs`
  and `tests/suite/main.rs`; the domain line in `layers.toml` is the
  world's change's; the suite line's comment, and the suite's helpers
  that the moves together leave unused, are the last change's.

## 6. Holders

For each slow test the goal removes or shortens, what holds its
assertions (`../../draft/slow-tests-down.md`, 3.2):

- **`temper-qa-tool::suite`'s witnesses stay whole:**
  `build_files::real_checksums_keep_a_non_utf8_path_as_its_bytes`;
  `builds::a_deployment_stage_keeps_the_coding_agent_version_and_checksums_all_eight_files`;
  `events::saved_commands_poke_after_commit_and_refusals_never_poke`,
  `events::socket_binding_replaces_stale_paths_and_ignores_the_umask`
  and `events::a_child_waiter_publishes_its_exit_before_waking_the_loop`;
  `hold::open_tightens_the_place_and_privatizes_its_log_before_launch`
  and `hold::the_ledger_watch_ends_the_drive_and_releases_the_place`;
  `place::restores_are_private_under_permissive_umasks`,
  `place::real_copies_leave_out_only_the_pinned_agent_and_allow_its_absence`
  and `place::real_restore_leaves_out_the_pinned_agent_from_an_older_copy`;
  `run::bounded_requests_are_not_filed_and_real_commands_are_killed_at_the_bound`;
  `run_passes::ending_a_pass_tells_the_holder_and_releases_its_place_without_the_ledger_watch`
  and `run_passes::ending_a_pass_captures_its_holder_before_the_socket_can_be_reused`.
  Each of the target's 131 other tests moves to `domain`, its own holder
  there. `builds::a_deployment_stage_keeps_the_coding_agent_version_and_checksums_all_eight_files`
  also gets a twin of its name in `domain`, holding its manifest, its
  eight files and its kept record over scripted versions.
- **`temper-qa::suite`,** by function. *Stays* is a shipped path left
  whole; *keeps* names the rows left, the rest moving; *moves* goes
  whole to `domain`'s `cli::`; *retires* deletes it where the named test
  holds every assertion, checked by its change, its exit and stderr held
  by the entry's tests, and moves it otherwise; *steps* goes to `step`.
  - `main`: `the_binary_answers_its_table_and_refuses_unknown_commands`
    keeps `--help`, `--version` and `nope`, the rest steps (`table`).
  - `builds`: `kept_build_moves_files_writes_sorted_sha256_manifest_and_records_time_without_holds`
    stays; `a_deployment_stage_is_kept_without_a_recording_bundle`
    retires to the twin above; `an_older_recording_bundle_is_kept_in_the_manifest`,
    `the_engine_must_print_the_kept_commit` and
    `each_regular_file_directly_in_bin_must_be_executable` retire to
    `builds::an_older_recording_bundle_is_kept_in_the_manifest`,
    `builds::the_engine_must_print_the_kept_commit` and
    `builds::even_the_coding_agent_and_extra_bin_files_must_be_executable`;
    `keep_arguments_are_refused_through_the_table_before_home_is_read`
    steps; the other six move.
  - `commands`: `a_case_from_file_and_stdin_records_at_once_and_refusals_leave_it_unchanged`
    keeps its `triage case -` row; `wake_pokes_without_a_session_or_ledger_write_and_saves_poke_too`
    keeps its wake on a live socket; the other three move.
  - `gaps`: `triage_gap_records_owners_and_lists_newest_first` keeps its
    first `triage gap`; the other two move.
  - `hold`: `one_holder_keeps_chain_scripts_lines_and_answers_until_close`
    keeps an open, a line and the close; `box_failures_cross_line_and_close_with_status_four`
    keeps its crashing line; the rest of theirs and the other four move,
    to the holder's two halves.
  - `passes`: `findings_from_file_and_stdin_copy_evidence_inherit_the_case_and_choose_area`
    keeps its finding from a file with its evidence;
    `finding_options_with_no_number_or_a_nonnumber_are_usage_refusals`
    steps; the other six move.
  - `place`: `setup_and_recipe_preserve_walk_exits_and_use_the_pass_place_and_captured_processes`
    keeps a setup exiting 4 and a recipe from stdin; the other moves.
  - `tune`: `tune_prints_all_four_figures_excludes_lost_and_unstarted_and_records_nothing`
    stays, one invocation, its figures stepping too; the other two step
    (`tune`).
