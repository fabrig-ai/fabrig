# box-small-2: the plan

From `inputs/request.md`: the two things `box-small` missed. One task, as the request asks. The trunk is `box-small-2/integration`. `walk = false`: the user tries it.

## Parts and tasks

| Item | Part | Task | Needs |
|---|---|---|---|
| 1, 2 | `words-and-link` | `words-and-link` | none |

## What the cut decided

- **The design moved with the first push:** `temper/docs/design/client/README.md`, 4.6, one sentence; `temper/docs/design/protocol/setup.md`, 7.2, *On a terminal*, and a clause of 13.1; a **Not yet** in each status file, which the task removes.
- **No defaults known still names all three.** `resolve` is all or nothing, and with no `HOME` there is nothing to compare with, so `no_home_names_all_three` stays.
- **An unset `TERM` isn't `dumb`.** The request names `dumb` alone, read literally. A terminal that doesn't know the sequence ignores it.
- **The URL's bound stays 4096, refused in today's words.** The request has "the line's bound" hold the URL twice plus the markers. The URL is read through its own cap, and the line is made after that, so only the output bound needs it, and `terminal::failure_bytes` already holds that far above 8,207 bytes. `LINK_LINE_BYTES` states the need in the `.max`; removing that `.max` can fail no test, and the spec says so.
- **`LinkForm` sits in `temper-box`, the choice in the binary.** The step crate has the rule and the bytes as pure functions; `temper`, which has `std`, reads the real stdout and hands the form in through `Settings`.
- **`program_login::line` keeps its three flags.** The request names `program_environment` alone; anyone but the owner still reads the flags.
- **`docs/help/` doesn't change.** "The link alone is on stdout" stays true, and those pages are read by tests.

## What isn't proved

No test runs a real `temper box` on a real terminal: the repository has no pty harness for one, and the request has the user try it. The form's rule and bytes are held on the simulated host, and `link_form` on real descriptors with `/dev/ptmx`. The one line left is `&std::io::stdout()` in `run_configuration`. The bare form on a real pipe stays held by `temper/crates/temper/tests/suite/box_listeners.rs`.

## Overlaps

- `temper/docs/design/client/README.md`: `chat-reading-c/reading-close` and `model-check/trial-close` also change it. This push's hunk is one sentence of 4.6.
- `temper/docs/design/client/README.status.md`: those two and `ui-pause-close/close-words` change it. The **Not yet** sits beside the 4.6 entry, in the middle of the file, and leaves with the task.
- `temper/crates/temper/src/tests.rs`: one `mod` line.
- Nothing else: `temper-box` and `directories.rs` were `box-small`'s alone, and it has merged.

## Critical path

`words-and-link`, alone.
