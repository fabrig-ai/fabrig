+++
id = "key-words"
part = "key-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/argv.rs",
  "temper/crates/temper-client-tools/src/jig/json.rs",
  "temper/crates/temper-client-tools/src/table.rs",
  "temper/crates/temper-client-tools/src/tests/key.rs",
  "temper/crates/temper-client-tools/src/tests/table.rs",
  "temper/crates/temper-client-tools/src/tests/words.rs",
  "temper/crates/temper-client-tools/src/tests/help.rs",
  "temper/docs/help/ctl.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "medium"
minutes = 25
+++

# A refused `--key` says what a key is

## Goal

`--key K` with a name outside the rule (`temper-client-tools/src/jig/key.rs`, `Name::new`) is refused with the bare usage line, and `temper help ctl` doesn't give the rule. Say the rule in both.

Acceptance:

- `temper ctl DIR task message 41 hello --key Bad` exits as usage does, with: `temper ctl: task message: --key KEY takes 1 to 64 lower-case letters, digits or hyphens; temper ctl help task message`.
- `--key` with no name after it says the same.
- `--call` naming a bad `key` says the same, with its own tool's name.
- A missing required key, a key on a tool that takes none, and `--key` twice keep the words they have today.
- Every tool's line fits 6.3's 240 bytes.
- `temper help ctl` says the rule, in its `**Keys:**` bullet.

## Inputs

- `temper/docs/design/client/tools.md`, 7, the bullet beginning `The agent names the key`, and 6.3, the paragraph on a usage error's line.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. No other file should need a change: `Usage::Key` is read only in `argv.rs`, `json.rs`, `table.rs` and `tests/table.rs`.

`ctl-waits` changes `temper-client-tools/src/tests/**` and one other line of `temper/docs/help/ctl.md`. Keep to your own lines there, and add tests at the end of their files.

Departs from the implementer's text: `temper/docs/help/ctl.md` is open for the one bullet in step 3, and `temper/docs/design/client/tools.status.md` for the one line in step 4.

Options, each pre-approved:

- `refusal` in `tests/words.rs` is private. Default: put the new story in `tests/words.rs`, beside it.
- If a golden under `temper-client-tools/src/tests/golden/` holds the ctl page's Keys bullet, regenerate it with its own tool and say so.

## Steps

1. **Reproduce.** In `tests/words.rs`, `a_refused_key_name_says_what_a_key_is`: for `--key Bad`, `--key a_b`, 65 bytes of `a`, and `--key` last with no name, `refusal(..)` equals the line above. It fails today with the usage line. Run it, see it fail.
2. **The refusal.** In `jig/argv.rs`, add `Usage::KeyName { tool: u8 }`, set by `Usage::at` as `Kind` is. In `parse`, both failures of `Name::new(words.get(next)...)` return it. `--key` twice stays `Usage::Key`, as do `check`'s `Keyed::Never` and `Keyed::Required` arms. In `jig/json.rs`, `Name::new(..)` failing returns it too. In `table.rs`'s `refusal`, it writes `ROW: --key KEY takes 1 to 64 lower-case letters, digits or hyphens; temper ctl help ROW`, with the row found as `Usage::WaitPart`'s is. Take the two bounds from one place if `jig/key.rs` can name them without a second copy of 64; `Name::new`'s own check is then read from it too.
   - `tests/table.rs`, `a_keyed_act_without_its_key_is_a_usage_error`: its bad-name loop now expects `Usage::KeyName { .. }`.
   - A `--call` story beside step 1's, through `fixtures::json`, naming `"key":"Bad"`: `table::refusal` of what it returns is the same line.
   - A story that every keyed row's line fits 240 bytes, going over `table::TOOLS`.
3. **Help.** In `temper/docs/help/ctl.md`, the `**Keys:**` bullet, after `reads and waits take none.`, add: `A key is 1 to 64 lower-case letters, digits or hyphens.` Rewrap as the page is wrapped. In `tests/help.rs`, a story that the page says so: `temper_docs::find(b"ctl")`'s text, its white space collapsed, holds that sentence.
4. **Status.** In `temper/docs/design/client/tools.status.md`, delete the line beginning `**Not yet:** 7: `--key` with no name`.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper-client-shell-cli`; `python3 -I -B temper/scripts/old_forms.py`. Commit and push.

## Report

As the implementer's text has it. Say whether a golden changed, and name the rule you broke for each new test and what it printed.
