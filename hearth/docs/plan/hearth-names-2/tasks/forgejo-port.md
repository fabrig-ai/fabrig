+++
id = "forgejo-port"
part = "forgejo-transcript"
kind = "migration"
needs = []
files = ["skein/tests/http/transcripts/forgejo-repository-captured.http", "skein/tests/http/transcripts/forgejo-repository-captured.expect"]
effort = "medium"
minutes = 15
+++

# The captured Forgejo transcript's SSH port becomes neutral

## Goal

The SSH port in `forgejo-repository-captured.{http,expect}` becomes `10022`: the same number of digits, so the `.http`'s chunk size holds and that file's size doesn't change. The transcript's tests pass as they did.

Acceptance:

- With the value of Step 1, `git grep -n -I -w "$old" -- .` prints nothing.
- `wc -c` of the `.http` reads what it read at base, and its chunk-size line is unchanged.
- The `.expect`'s `body` and the `.http`'s body say the same thing, each with `10022` in `ssh_url` and `wiki_ssh_url`.
- `git diff --stat` against your base shows these two files and nothing else.
- The tests of Step 3 pass.
- `tools/citations/citations.py --all` reads 0 and `outward` passes (`hearth/docs/plan/hearth-names-2/notes/common.md`, 4).

## Inputs

- `hearth/docs/plan/hearth-names-2/notes/common.md`: binding
- `hearth/docs/plan/hearth-names-2/inputs/request.md`, item 5
- `skein/docs/foundation/testing-strategy.md`, 4.1: the transcripts
- `skein/tests/http/src/transcript.rs`: `#` lines of an `.expect` are comments, and every transcript in the directory is read by name

## Owns and may change

- **Owns:** the two files.
- **Not to touch:** anything else. `smith/benchmarks/summaries/` holds the digits by chance, inside a float; a word-bounded search doesn't find it.
- **The `.expect`'s header comment changes too,** since it says the bytes are as they came but for the host and the chunk size, and that stops being true (`AGENTS.md`, 2). It isn't a port "matching in the file", so the plan records it.

## Steps

1. **Read the old value, before any edit,** into an untracked `values.sh` at the slot's root, which each later command sources. Check that it is non-empty.

   ```sh
   echo "old=$(grep -oE 'git\.example\.org:[0-9]+' skein/tests/http/transcripts/forgejo-repository-captured.http | head -1 | cut -d: -f2)" > values.sh
   ```

2. **Replace it, word-bounded, in both files.** At base that is two hits in each, one line each (`ssh_url`, `wiki_ssh_url`). A hit anywhere else in these files is replaced too, and named in the report.

   ```sh
   . ./values.sh
   sed -i "s/\b$old\b/10022/g" skein/tests/http/transcripts/forgejo-repository-captured.http skein/tests/http/transcripts/forgejo-repository-captured.expect
   ```

   Then the `.expect`'s header reads:

   ```
   # Captured from the user's own forge, an unauthenticated read of a public
   # repository: curl --http1.1 --raw -i https://git.example.org/api/v1/repos/ai/temper
   # on 2026-10-04, through its nginx, bytes as they came but for the forge's host,
   # written git.example.org here, its SSH port, written 10022 here, and the chunk
   # size that follows from the host. A Forgejo repository, chunked in one chunk, its
   # JSON ended by an LF.
   # Drafted by the reference readers, then checked by hand.
   ```

3. **Run the transcript's readers,** one command at a time. No Rust changes, so there's no clippy or `cargo fmt` to run.

   ```sh
   heavy cargo nextest run -p skein-http-world --test suite -E 'test(/^transcripts::/)'
   heavy cargo nextest run -p skein-http-world
   heavy cargo nextest run --profile fuzzy -p skein-http-world
   ```

4. **Commit with a message that spells no old value** (`skein: the captured Forgejo transcript's SSH port is neutral`), adding the two files by name. Delete `values.sh`, push, then run the checks of `notes/common.md`, 4.

## Report

The hits replaced in each file, the `.http`'s size before and after, test counts. It spells no old value.
