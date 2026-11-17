# The outward scan

Provisional, 2026-11-17. fabrig names nothing of whoever runs its
development: no repository, tool, machine or session of that lab
(`AGENTS.md`, 1). Two checks hold the rule, with one list of names: the
Rust test `tests/boundary/tests/outward.rs` (package `fabrig-boundary`,
in the focused suite), which the gate and every delivery that builds
run, and the text check `tools/outward/outward.py`, which builds nothing,
for a prose-only delivery (`tools/pre-pr/design.md`, 3 and 4). Written
from `temper/docs/design/draft/small-fixes-1116.md`, 3.

This document spells no name: the list is the only place that does,
beside the Rust test's own cases.

## 1. In one page

- **One list,** `tools/outward/names.txt`: each name, with what its ends
  need (section 2). The Rust test and the text check both read it, and
  neither keeps a name of its own: the Rust test's `FIXED` table, its
  special case for a name followed by digits and its prefilter's
  hard-coded prefixes give way to the list.
- **One scan:** the files git lists, read as text, the same files left
  out, a name found the same way on each line (section 3).
- **One verdict:** the same failure text, file by file and line by line
  (section 4). The text check prints it and exits 1; the Rust test fails
  with it.
- **The Rust test stays** for the suites: the gate and every delivery
  whose tests' set holds `fabrig-boundary` run it as before. The text
  check runs where nothing is built, a prose-only delivery
  (`tools/pre-pr/design.md`, 3).
- **They agree,** and a test of the boundary crate pins it: the text
  check and the Rust scan, on one scratch repository, give one verdict
  (section 5).

## 2. The list

`tools/outward/names.txt`, UTF-8 holding ASCII alone, a line each:

- **A comment:** an empty line, or one starting `#`.
- **A name:** `<left> <right> <name>`, each mark one byte and one space
  after each:
  - `<left>`: `w`, the name's first byte starts the line or follows a
    byte that is no word byte; `-`, nothing asked before it;
  - `<right>`: `w`, the name's last byte ends the line or a byte that is
    no word byte follows it; `-`, nothing asked after it; `d`, one or more
    ASCII digits follow it, which the match takes, with nothing asked
    after them;
  - `<name>`: the rest of the line, at least one byte, printable ASCII
    (0x20 to 0x7E), its first and last bytes no space, written in lower
    case, and matched ASCII case-insensitively.

  A word byte is an ASCII letter, an ASCII digit or `_`.
- **Its order is the matching's:** at a place on a line, the first entry
  that matches there wins (section 3), so a longer name is listed before
  a shorter one it starts with, and the entry whose right mark is `d` is
  listed after every other entry.
- **A malformed line** (another mark, a missing space, an empty or
  non-ASCII name, a space at the name's end) is an error in both checks:
  the text check exits 2 naming the line, and the Rust test fails naming
  it.
- **Read whole by both,** never copied: the Rust test compiles it in
  (`include_str!`, so a change to it rebuilds the test), and the text
  check reads it from its own directory, `tools/outward/`, whatever
  repository it scans.

## 3. The scan

- **Its files:** `git ls-files -z` run at the scanned repository's root,
  as the environment gives git its index: the Rust test runs with the
  repository's own index, and the delivery check gives the text check its
  private index (`GIT_INDEX_FILE`), so that it scans the delivery's tree,
  untracked files included and ignored ones not
  (`tools/pre-pr/design.md`, 3 and 4). `target/`, `.git/` and other
  untracked build output are never listed. A listed path that is not
  UTF-8 is an error.
- **Left out,** and nothing else:
  - `tools/outward/names.txt`, which spells the names;
  - `tests/boundary/tests/outward.rs`, whose cases spell names to pin
    the matching;
  - every path inside a plan's directory, `<tree>/docs/plan/<draft>/…`:
    a path of at least five components whose second and third are `docs`
    and `plan`, the transient run files of whoever runs a plan;
  - a binary file: a NUL byte among its first 8,000 bytes, as git
    decides.
- **Each listed path, read:** a symlink's text is its target's path as
  git stores it, not the file it names; a regular file's text is its
  bytes; anything else (a directory, a missing file) is an error. A file
  that cannot be read is an error, never a skip.
- **On each line:** the text is cut at each `\n` into lines, numbered
  from 1; a `\r` before it stays on the line, a byte that is no word
  byte. From the line's first byte: at each place, each entry is tried in
  the list's order, and the first whose name's bytes equal the line's
  there, ASCII case-insensitively, with its marks met, is found; the
  scan takes its bytes, and any digits a `d` takes, as the line spells
  them, and goes on after them. With no entry found there, it goes on at
  the next byte. Names on a line never overlap.
- **A prefilter,** where kept for speed, is derived from the list at
  run time (a file or a line that holds no entry's name, case folded,
  holds none found), never a second list.

## 4. The verdict

- **No name:** the Rust test passes; the text check prints nothing and
  exits 0.
- **A name anywhere:** the failure's text, the same in both:

  ```
  Files name the lab running fabrig's development (AGENTS.md, 1):
  <path>:<line>: <name>
  ```

  a line for each name found, each `<name>` as the line spells it; the
  files in the byte order of their paths, and a file's names in the order
  of their lines and, on a line, of their places. The text check prints
  it to its standard output, each line ended by `\n`, and exits 1; the
  Rust test fails with it.
- **An error** (git failing, a path not UTF-8, a file unreadable or
  neither a file nor a symlink, a malformed list): the Rust test fails
  with it; the text check prints `outward: <error>` to its standard error
  and exits 2, whatever it found.
- **Its command:** `python3 -I -B tools/outward/outward.py`, no
  arguments, from the scanned repository's root; it reads only the list
  and the listed files, and writes nothing.

## 5. Testing

- **The Rust test's own cases** stand, reading the list: each name
  found, names on a line never overlapping, product words passing, the
  left-out paths, the binary rule, the verdict's text and a symlink read
  as its target's path. A malformed list's lines are refused, from a
  list given as text.
- **They agree,** in `tests/boundary/tests/outward.rs`: a scratch git
  repository holding files with names of each kind of entry (each left
  and right mark, a name with digits, two names on a line, one in a
  symlink's target), a binary file, a file in a plan's directory and a
  copy of the list at its path; the text check, run there from this
  repository's `tools/outward/outward.py`, exits 1 and prints exactly
  the Rust verdict's failure text for that repository; with the names
  taken out, the text check exits 0 and prints nothing, and the Rust
  verdict passes.
- **In the delivery check,** `tools/pre-pr/test_pre_pr.py` runs it on a
  prose-only delivery (`tools/pre-pr/design.md`, 9.2).

## 6. Later

- **A scan of the changed files alone:** the text check reads the whole
  tree, as the Rust test does, so that their verdicts are one; a
  prose-only delivery on a tree that already holds a name refuses unless
  the goal's reds name the Rust test (`tools/pre-pr/design.md`, 5).
