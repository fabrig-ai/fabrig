+++
id = "outward-check"
part = "outward-check"
kind = "feature"
needs = []
files = [
  "tests/boundary/tests/outward.rs",
  "tests/boundary/outward.txt",
  "tests/boundary/Cargo.toml",
]
effort = "high"
minutes = 75
+++

# The outward check: no file names the lab beyond its inventory

## Goal

fabrig is to name nothing of whoever runs its development: no repository, tool, machine or
session of that lab. About 230 such names stand today outside the plans; later steps remove
them. This task stops new ones arriving: a new test binary `outward` in `fabrig-boundary` reads
every tracked text file and fails on a name of the lab beyond the counts of a new inventory,
`tests/boundary/outward.txt`, generated at the task's base. The package is already in every
gate.

Acceptance:

- `tests/boundary/tests/outward.rs` holds the module doc, matcher, listing, inventory and tests
  of `## Steps`; its ten tests pass, and with `direction`'s three the package's 13.
- `tests/boundary/outward.txt` is the header of `## Steps`, 3, then exactly the entries the
  tree needs at your base; the cross-check of `## Steps`, 4 agrees with it line for line.
- The scan test takes under 1 s as nextest reports it under heavy. The commit (`## Steps`, 6)
  states that time and the number of files read, without the machine's name (the
  report names the machine).
- `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings` is clean, `cargo fmt`
  changes nothing, and `python3 -I -B tools/citations/citations.py --all` reports 0 broken.
- No other file changes, but for what `## Owns and may change` allows.

## Inputs

- `temper/docs/plan/outward-rule/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-rule/inputs/request.md`, "The check" and "Bounds": the request. This
  spec settles what it leaves open (the decisions under the table of `## Steps`, 2).
- `tests/boundary/tests/direction.rs`: the model. Its module doc states the rule and how the
  check reads; its unit tests are tables of short inputs; its failure lists every breach;
  `skein_scratch::paths::workspace()` gives the root.
- `AGENTS.md`, 2 ("Public code names ... nothing private: ... session names"): what the module
  doc cites.
- The root `Cargo.toml`, `[profile.dev.package.jig-boundary]` (~line 602): the form of the
  profile line below.
- `.config/nextest.toml` (the default profile selects `package(=fabrig-boundary)`) and
  `.config/gate/always.txt` (lists it): the gate already runs the new binary; nothing to add.

## Owns and may change

- **`tests/boundary/tests/outward.rs`** (new) and **`tests/boundary/outward.txt`** (new), whole.
  Cargo finds `tests/outward.rs` by itself: no `[[test]]` table.
- **`tests/boundary/Cargo.toml`:** the `description` line only, which becomes
  `"Checks the repository's boundaries: the trees' dependencies point one way, and no file names an outside lab beyond its inventory"`.
- **The root `Cargo.toml`:** only if the scan test takes over 1 s without it, add after
  jig-boundary's two lines, with a blank line between:

  ```toml
  [profile.dev.package.fabrig-boundary]
  opt-level = 3
  ```

  The report gives both times. Nothing else in that file; `Cargo.lock` does not change.
- **If `git` cannot run where heavy runs the test** (the spawn fails, or git refuses the
  directory): list the files by a walk from the root instead, which skips the `.git` entry,
  every directory named `target` and every symlink, and reads every other file; the module doc
  says so in place of the `git ls-files` sentence, and the report says why, with git's error.
  The rest of this spec stands.
- **If the cross-check and the test disagree,** the table of `## Steps`, 2 decides: fix the
  side that departs from it. A file that `.gitattributes` marks binary (so `git grep -I` skips
  it) but with no NUL in its first 8000 bytes is text to the test: the test is right, and the
  report names the file.
- **If the scan stays over 1 s** with the profile line: find the cause first (`## Steps`, 2,
  "Speed"). Still over: push, and the report's first line is `Question:` with both times, the
  files and bytes read and what you tried; the request's bound ("well under the gate's CPU
  budget") is the user's.

## Steps

1. **The module doc** states, in this order and in the request's words: the rule, with its
   citation `(AGENTS.md, 2)`, written as `direction.rs` cites section 1; how the check reads (git's tracked files from the workspace root,
   `git ls-files`, so `target/`, `.git/` and other untracked build output are never listed);
   what it skips (symlinks, whose targets are tracked files read under their own paths; binary
   files, a NUL byte in the first 8000 bytes, as git decides); what it leaves out (its own file,
   its inventory, and every `docs/plan/` directory "until the plans move out"); the names it
   fails on and the words left off on purpose (a bare `director`, a bare `heavy`, a generic
   `/home/<user>/`); the inventory's rule (3 below); and "A line it cannot read is a failure,
   never a skip."

2. **The matcher.** A file is matched as bytes, split into lines at `\n`, numbered from 1.
   Each name below matches ASCII case-insensitively; ‹b› is a boundary: the start or end of the
   line, or a byte that is not an ASCII letter, digit or `_`. On a line, names are taken left
   to right and the search goes on after the end of the last one, so names never overlap; no
   two rows match at one place. A file's count is the number of names in it.

   | Name | Before | After | The request's item |
   |---|---|---|---|
   | `fabrig-ai/agents` | | ‹b› | `fabrig-ai/agents` |
   | `agents repo`, `agents repos`, `agents repository`, `agents repositories` | ‹b› | ‹b› | "the agents repository", "agents repo" |
   | `fab` | ‹b› | ‹b› | `\bfab\b`, `fab's`, `bin/fab` |
   | `fab_home` | ‹b› | ‹b› | `FAB_HOME` |
   | `heavy cargo`, `heavy env` | ‹b› | ‹b› | `heavy cargo`, `heavy env` |
   | `heavy --` | ‹b› | | `heavy --…` |
   | `"heavy"`, `'heavy'` | | | `"heavy"` as an argv element |
   | `hel8` | ‹b› | ‹b› | `hel8` |
   | `hetzner` and one or more digits, all of them | ‹b› | | `hetzner` followed by digits |
   | `debian-16gb`, `status.fabrig.ai` | ‹b› | ‹b› | the same |
   | `~/src/fabrig/`, `~/src/rust/tmp/`, `/home/free/` | | | the same |
   | `fabrig/qa`, `fabrig/dogfood`, `fabrig/debt`, `agents/loop`, `agents/status`, `agents/creds`, `agents/ops` | ‹b› | ‹b› | the same |

   Spaces in a name are one space. Decisions this table makes:
   - "the agents repository" is found as `agents repository`; `fab's` and `bin/fab` as `fab`.
   - `agents repo` needs a boundary after it, so "agents report" (smith's
     `smith-host-domain/src/parent.rs`, `smith/docs/design/benchmarks.md`) passes; likewise
     "a heavy environment" and "heavy cargoes".
   - `"heavy"` and `'heavy'` count in any file, not only scripts: today's holders are
     `tools/tiers/` and temper's `forge_land.rs` stubs, which stand for the same tool.
   - The three paths are literal, with their trailing slash, as the request writes them, so
     `/home/free/src/rust/tmp/x` is one name, `/home/free/`, and `~/src/fabrig/qa` is one,
     `~/src/fabrig/`.

   Sketch (inside one test file: these names are the ones the tests call; the rest is yours):

   ```rust
   const OWN: &str = "tests/boundary/tests/outward.rs";
   const INVENTORY: &str = "tests/boundary/outward.txt";

   /// The lab's names on `line`, left to right, never overlapping, each as the line spells it.
   fn names(line: &[u8]) -> Vec<&[u8]>;

   /// Every name in a file's `text`: its line, from 1, and its text.
   fn found(text: &[u8]) -> Vec<(usize, String)>;

   /// Whether the check leaves out `path` (from the root, `/`-separated): its own file, its
   /// inventory, and anything with a `docs/plan/` directory in it, at the root or in a tree.
   fn left_out(path: &str) -> bool;

   /// Whether a file's `bytes` are binary: a NUL among the first 8000.
   fn binary(bytes: &[u8]) -> bool;

   /// What the scan read: each text file's path, and the names of those that hold any.
   struct Scan {
       read: Vec<String>,
       found: BTreeMap<String, Vec<(usize, String)>>,
   }

   /// `git -C root ls-files -z`, then each listed path not left out: a symlink is skipped,
   /// anything else that is not a file is an error, a binary file is skipped.
   fn scan(root: &Path) -> Result<Scan, String>;
   ```

   `scan` fails, naming the path or git's status and stderr, when git cannot be spawned or
   exits non-zero, a listed path is not UTF-8, cannot be read (a tracked file gone from the
   working tree included) or is neither a file nor a symlink.

   **Speed.** About 16,500 text files, 122 MB, are read; nearly all hold no name. Keep the
   per-line matcher off the files that cannot hold one (a cheap whole-file pre-filter, for
   instance), and take the profile line of `## Owns and may change` if the test is still over
   1 s. Measure with nextest's figure, never a clock in the test (`tests/clippy.toml` bans
   `Instant`).

3. **The inventory.** `tests/boundary/outward.txt` starts with this header and a blank line:

   ```
   # The files that name the lab running fabrig's development, and how many names each holds
   # (tests/boundary/tests/outward.rs). A file not listed holds none; each count may only fall.
   # Raising a count or adding a file needs a `# Raised: <who, why>` line directly above its entry.
   ```

   Its lines: blank (empty or only spaces), a comment (starting `#`), or an entry `PATH COUNT`:
   a path from the root with no whitespace in it, one space, and a count from 1, in decimal without
   a leading zero. Entries go in strictly ascending byte order of their paths. Any other line,
   a path listed twice or out of order is an error naming the line's number and text.

   - A file not listed may hold no name; a listed file at most its count. Fewer passes, and so
     does an entry whose file holds none today or is gone: the failure never asks for a lower
     count.
   - **The failure** opens with one line saying that files name the lab beyond
     `tests/boundary/outward.txt`, that each count may only fall, and that raising one or adding
     a file needs a `# Raised: <who, why>` line directly above its entry. Then each file beyond
     its count, with its count and its entry's (or "not listed"), and every name in it as
     `PATH:LINE: NAME`. Then "The inventory the tree needs:" and that text whole.
   - **The inventory the tree needs** is today's file with each entry at today's count; an
     entry whose file holds none dropped, with the `# Raised:` lines directly above it (no
     other line between); and each file not yet listed added above the first kept entry that
     sorts after it (above that entry's `# Raised:` lines), or after the last line if none
     does. Every other line stays as it is, and the text ends with a newline.

   ```rust
   /// The inventory's entries: each listed path and its count.
   fn inventory(source: &str) -> Result<BTreeMap<String, usize>, String>;

   /// The verdict on `found` against the inventory's `source`: the failure, when a file holds
   /// more names than its entry allows. An unreadable `source` is a failure too.
   fn verdict(found: &BTreeMap<String, Vec<(usize, String)>>, source: &str) -> Result<(), String>;

   /// The inventory the tree needs, from the inventory's `source` and today's `counts`.
   fn needed(source: &str, counts: &BTreeMap<String, usize>) -> String;
   ```

   **The tests**, by these names; each unit test is a table of short inputs, as in
   `direction.rs`:

   - `no_file_names_the_lab_beyond_the_inventory`: `scan(&skein_scratch::paths::workspace())`;
     asserts it read files under each of `skein/`, `smith/`, `jig/`, `temper/`, `tools/` and
     `tests/`, and the root's `AGENTS.md`; reads `INVENTORY` (unreadable fails); panics with
     `verdict`'s failure, if any.
   - `each_name_of_the_lab_is_found`: one line or more per row of the table, with the names it
     must give, case kept; for instance `"The agents repository's fab/cpu.py"` gives
     `["agents repository", "fab"]`, `"FAB_HOME=x heavy --gate"` gives `["FAB_HOME", "heavy
     --"]`, `"hel8, hetzner12 and debian-16gb-hel1-1"` gives `["hel8", "hetzner12",
     "debian-16gb"]`, `r#"["heavy", 'heavy']"#` gives both quoted forms. It also checks
     `found(b"one\nfab two\r\nhel8")` is `[(2, "fab"), (3, "hel8")]`.
   - `names_on_a_line_never_overlap`: `/home/free/src/rust/tmp/x` gives `["/home/free/"]`;
     `~/src/fabrig/qa` gives `["~/src/fabrig/"]`; `bin/fab` gives `["fab"]`.
   - `product_words_pass`: each gives no name: the request's "tool-heavy", "the director task",
     "/home/<user>/", "fabric", and "fabrig", "prefab", "a heavy test", "a heavy environment",
     "heavy cargoes", "spawned agents report their errors", "the agent's repository",
     "smith/benchmarks/agents/codex", "hetzner", "hel80", "/home/freebsd/", "~/src/fabrig".
   - `the_check_leaves_out_plans_its_own_file_and_its_inventory`: left out:
     `temper/docs/plan/outward-rule/inputs/request.md`, `docs/plan/a.md`, `OWN`, `INVENTORY`;
     read: `temper/docs/planning.md`, `temper/docs/design/plan/a.md`,
     `tests/boundary/tests/direction.rs`.
   - `a_nul_in_the_first_8000_bytes_makes_a_file_binary`: a NUL at index 0 and at 7999 is
     binary; at 8000, or none, is text.
   - `an_inventory_line_the_check_does_not_read_fails_it`: `a.md`, `a.md x`, `a.md 0`,
     `a.md 01`, `a.md 1 2`, `a.md  1` (two spaces) each fail naming the line; `b.md 1\na.md 1`
     fails as out of order, `a.md 1\na.md 2` as listed twice; and
     `"# c\n\n.config/x 2\nA.md 1\nb/c.md 10\n"` reads as its three entries.
   - `fewer_names_than_listed_pass`: source `a.md 3\ngone.md 2\n`, `a.md` holding one name:
     `Ok(())`.
   - `a_file_beyond_its_count_fails_with_its_names_and_the_needed_inventory`: source
     `"# The header.\n\na.md 1\nc.md 3\n"`; found `a.md` at lines 2 (`fab`) and 7
     (`heavy cargo`), `b.md` at 1 (`hel8`), `c.md` at 4 (`/home/free/`). The failure holds
     `a.md:2: fab`, `a.md:7: heavy cargo`, `b.md:1: hel8`, `# Raised: <who, why>` and
     `"# The header.\n\na.md 2\nb.md 1\nc.md 1\n"`, and no `c.md:4`.
   - `the_needed_inventory_keeps_the_comments_and_takes_todays_counts`: source
     `"# The header.\n\na.md 3\n# Raised: lead, a reason\nb.md 2\n# Raised: lead, another\nc.md 1\nd.md 1\n"`,
     counts `a.md 1, b.md 4, bb.md 2, d.md 1, e.md 5`; needed:
     `"# The header.\n\na.md 1\n# Raised: lead, a reason\nb.md 4\nbb.md 2\nd.md 1\ne.md 5\n"`.

   Run the nine unit tests: `heavy cargo nextest run -p fabrig-boundary --test outward -E 'not
   test(=no_file_names_the_lab_beyond_the_inventory)'` (nine selected; check the count).

4. **Generate and cross-check the inventory.** Write the header and a blank line, then the
   entries this prints at your base, from the repository root:

   ```sh
   R='fabrig-ai/agents\b|\bagents repo(?:s|sitory|sitories)?\b|\bfab\b|\bfab_home\b|\bheavy (?:cargo|env)\b|\bheavy --|"heavy"|'"'heavy'"'|\bhel8\b|\bhetzner[0-9]+|\bdebian-16gb\b|\bstatus\.fabrig\.ai\b|~/src/fabrig/|~/src/rust/tmp/|/home/free/|\bfabrig/(?:qa|dogfood|debt)\b|\bagents/(?:loop|status|creds|ops)\b'
   git grep -P -i -o -I -e "$R" -- . ':(exclude,glob)**/docs/plan/**' \
       ':(exclude)tests/boundary/tests/outward.rs' ':(exclude)tests/boundary/outward.txt' \
     | cut -d: -f1 | LC_ALL=C sort | uniq -c | awk '{print $2, $1}'
   ```

   At this plan's cut (trunk `58edb47912`) it printed 71 entries, 231 names, `AGENTS.md 24`
   among them; your base may differ a little. Then run the scan test alone: `heavy cargo
   nextest run -p fabrig-boundary --test outward -E
   'test(=no_file_names_the_lab_beyond_the_inventory)'` (one selected). It passes: the test and
   the cross-check agree. A failure prints the inventory the test needs: compare it with the
   cross-check's and settle the difference by the table (`## Owns and may change`). Record
   nextest's time for the test.

5. **The whole package:** `heavy cargo nextest run -p fabrig-boundary` (13 selected). The
   package has no dependents. Then `cargo fmt`, and `heavy cargo clippy -p fabrig-boundary
   --all-targets -- -D warnings`.

6. **Commit and push:** one commit for the test, its inventory, the description line (and the
   profile line, if taken), so that no commit is red; its message says why (new names stop
   arriving while later steps remove the old), that the inventory is the base's, and the scan's
   time and the number of files read. Then `python3 -I -B
   tools/citations/citations.py --all` (0 broken). Push your branch.

## Report

As the implementer's text says, and: the scan test's time and the machine (both times if the
profile line was tried); the files and bytes the scan read; the inventory's entries and names;
the cross-check's agreement; each run's selected count; whether git ran under heavy, or the
walk replaced it and why.
