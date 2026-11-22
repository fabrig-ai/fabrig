# records-deployment: status

2026-11-22. Where the tree's code lags or departs from [records-deployment.md](records-deployment.md).

The document and `../tui/box.md` have no known implementation gap in the TUI in
front of a box, by the look request `tui-box-front` (its account
`0030c4f597`): the forges document's `linked`, set at jig's
`LinkSignedIn` and false after a restart, and `welcome::signed_in`; the
handed link on descriptor 3 and the box's lines on descriptor 4,
`BoxLine` read and printed back; `Start.front` and `Report::Front`; the
welcome before and after the sign-in, its steps read-only at
`welcome::decision`'s step, with the forge under way; the playground's
header, banner and quiet line; the quit question by `work_runs`, a give
up's note and `/log`, offered in front alone; and the place "box" or
"box:PORT" by `place(address, true)`. Goldens at 80 by 24, the state,
view, theme, host, run and domain rows, and the joint world's TUI stories
(`temper/tests/client/tests/suite/tui.rs`) hold them; the theme's rows
map the new tokens in both schemes, which establishes no rendered look.
The quiet line names a step as the box names it ("forgejo-3
restarting"), where the request's frame 05 has "forge restarting", and
`/log` shows the box's own lines, as `../tui/box.md` has them
(`../../draft/tui.status.md`). A journey with the TUI in front of a box
has no driver (`../drive.md`, 7); the real terminal's check, Linux and
macOS, is `../README.md`, section 5's after-landing work, owned by temper/tui.

- **Design departs:** *The program's environment*, from the request `box-small-2`
  of 2026-11-06, item 1, decided by its plan's cut, which takes `resolve` as all or
  nothing: with no `HOME` and an XDG variable placing some of the directories, not all,
  `program_environment` (`temper-engine-shell/src/serve.rs`) knows no default and names
  all three, one at its own default among them, where the request has each only where it
  differs. The command finds the same directories either way, and no test holds this
  case: `no_home_names_all_three` gives no XDG variable. Seen by reading.

- **Design departs:** *The words of a command for this machine*, from the request
  `box-small` of 2026-11-06, item 1, which has these words in any other command handed
  to the person for this machine: `temper-box forge run`'s refusal
  (`temper-engine-shell/src/forge.rs`, `run_refusal`) says `temper-box forge prepare` and each
  directory whole, and the terminal client says `temper-box open`
  (`temper-client-shell-terminal/src/terminal.rs`), so with another `temper-box` first on
  `PATH` each names that one. That plan's cut names the pages' commands alone, and no
  reason is recorded. Seen by reading.
