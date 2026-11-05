+++
id = "kit-tls"
part = "protocol-kit"
kind = "feature"
needs = ["protocol-kit"]
files = ["skein/tests/tls/Cargo.toml", "skein/tests/tls/src/world.rs", "skein/tests/tls/src/world/scenario.rs", "skein/tests/tls/src/world/check.rs", "skein/tests/tls/tests/suite/fuzzy/world.rs", "Cargo.lock"]
effort = "medium"
minutes = 20
+++

# skein: the TLS world on the protocol-world kit

## Goal

`skein/tests/tls` takes `meets`, `delivers`, `fault`, `draw` and the tally from `skein-protocol-kit` (`notes/contracts.md`, 1), and
its copies go. It has no transcript loader.

Acceptance:

- `rg -n 'fn (meets|delivers|fault|draw)\b' skein/tests/tls` and `rg -n 'struct Seen\(BTreeMap' skein/tests/tls` find nothing.
- The end check in `src/world/check.rs` holds what it holds today, its assert's text unchanged, and goes red if the `Read::Nothing`
  case below is dropped.
- Every test in `skein-tls-world` passes unchanged, focused and fuzzy.
- The branch's net lines fall by at least 35.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 1

## Owns and may change

- **Owns:** the files in `files`. `skein/tests/tls/Cargo.toml` gets a `[dependencies]` line; `Cargo.lock` follows.
- **TLS's `meets` differs from the kit's in one arm, and it matters.** For TLS `Read::Nothing` is met by any byte left, for http by
  none. `run.ended` can really be `Read::Nothing` here, so with the kit's form the assert in `check_ending` could never fire. Keep
  it in `check.rs`:

  ```rust
  /// What a demand is met by at the end: any byte left meets `Read::Nothing`.
  fn met_at_end(read: Read, bytes: &[u8]) -> bool {
      matches!(read, Read::Nothing) && !bytes.is_empty() || meets(read, bytes)
  }
  ```

  Compare the other arms as you delete TLS's copy. A second difference is a question.
- **`scenario.rs`'s `fault` goes through `pick`,** one `below(3)` over the same array in the same order, so the kit's draws the same.
  `pick` stays for its other callers. `below_acts` isn't touched.

## Steps

1. The dependency, the four functions, `met_at_end`, then `fuzzy/world.rs`'s tally (`record` a free function).

2. Drop `met_at_end`'s `Read::Nothing` case by hand, and see what fails: if a test goes red, report it. If none does (the world may
   never hold bytes there on these seeds), make a throwaway `Run` that shows the assert fire, report that, and restore.

3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-tls-world --all-targets -- -D warnings`; `heavy
   cargo nextest run -p skein-tls-world`, then with `--profile fuzzy`; the citations check. Commit and push.

## Report

The common note's items, what you broke and what failed.
