+++
id = "terminal-namespace"
part = "terminal-namespace"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-text/src/terminal.rs", "skein/crates/skein-ui-text/src/tests.rs", "skein/docs/design/io.status.md", "skein/docs/development/debt/D044-io-token-tags.md", "skein/docs/development/debt/D018-io-pipe-and-file-states.md", "smith/docs/development/debt/D003-host-io-route-table.md", "hearth/docs/development/debt/D004-service-drives-io-by-hand.md"]
effort = "medium"
minutes = 20
+++

# skein: the terminal names its operations through `Token::tagged`

## Goal

Pay `skein/docs/development/debt/D044-io-token-tags.md`. `skein_ui_text::Terminal` sets bit 62 by hand to tell its `Io`'s
operations from its service's. Namespace 4 of `Token::tagged` is the same bit, so every raw token stays what it is.

Acceptance:

- `rg -c 'const TAG: u64 = 1 << 6' skein -g '*.rs'` finds nothing.
- `Terminal`'s submissions and completions hold the same raw tokens as before, and `Terminal::owns`'s signature stays.
- `rg -n 'raw\(\) [&|]|TAG' skein/crates/skein-ui-text/src/terminal.rs` finds nothing.
- Production lines of `terminal.rs` fall.
- `skein/docs/development/debt/D044-io-token-tags.md` is deleted, and the entry below leaves `io.status.md`.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D044-io-token-tags.md`
- `skein/docs/design/lib.md`, 4; `skein/docs/design/io.md`, 5

## Owns and may change

- **Owns:** `skein/crates/skein-ui-text/src/terminal.rs` and a test in `skein/crates/skein-ui-text/src/tests.rs`.
- **A change of meaning, for tokens nothing makes:** `owns` reads true today for any token with bit 62 set, and after
  this only for namespace 4 exactly. No token that reaches it changes side: `Io`'s tokens have their top four bits clear
  (`io.md`, 5), and neither reader's other tokens carry bit 62 (`temper-client-shell-terminal/src/terminal/run.rs` tags
  its own `Io` at bit 61; `jig/tests/client-protocol/src/native.rs` passes the shell's tokens). So this task runs their
  tests. If a reader's other token does carry bit 62, stop and ask.
- **Stay out of** the hand tags of temper and smith (`run.rs`'s `TAG`, `temper-box`'s, `temper-client-shell-cli`'s, smith's
  bits): this item's measure is skein's, and they are their trees'.
- **Debt and status lines, each as given, outside this spec's code:**
  - `skein/docs/design/io.status.md`: the sentence of its `5:` entry from "`FileIo`'s lane tells no two `Io`s apart" to
    "`skein/docs/development/debt/D044-io-token-tags.md`)." goes, and the next `2:` entry and the earlier `5:` lines stay,
    with the punctuation between them kept right. Nothing is left to record: `io.md`, 5 leaves two `Io`s to the service,
    and the service's terminal does it through `Token::tagged`.
  - `skein/docs/development/debt/D018-io-pipe-and-file-states.md`: the sentence "The operation-token question (D044) touches
    `FileIo` too." goes.
  - `hearth/docs/development/debt/D004-service-drives-io-by-hand.md`: "skein's D044 is a second `Io` on one kernel, so it
    doesn't block this." becomes "A second `Io` on one kernel is the service's to tell apart (`skein/docs/design/io.md`, 5),
    so it doesn't block this."
  - `smith/docs/development/debt/D003-host-io-route-table.md`: its status reads `open`, and its "What pays it back" opens:

    ```markdown
    skein gives the lane: a token's top bits name its namespace (`Token::tagged`,
    `skein/docs/design/lib.md`, 4; skein's review's io F1), as skein's terminal
    does. It still lacks one public validity check (its F6). Then
    ```

    The rest of that paragraph stays. `python3 -I -B tools/debt/debt.py smith` still exits 0.

## Steps

1. In `terminal.rs`:

   ```rust
   /// This terminal's namespace among its service's operations (lib.md, 4).
   const NAMESPACE: u8 = 4;
   ```

   `owns` reads `op.tag() == NAMESPACE`. `iterate` clears with `Token::new(complete.op.untagged())`, and the target
   likewise. The submission loop tags with `Token::tagged(NAMESPACE, submit.op.raw()).expect("finite terminal operation
   namespace")`, and the target likewise; the `assert!` goes, since `tagged` refuses what it refused and more. `tagged`'s
   check takes in all four top bits, which `Io`'s tokens leave clear.

2. A table test in `skein-ui-text/src/tests.rs`, `a_terminal_owns_its_namespace_alone`: `owns` is true for
   `Token::tagged(4, 7)`, false for `Token::new(7)`, for namespace 2 (a neighbour's lane) and for namespace 5 (bit 62 with
   another bit). Break it by hand: `owns` back to the bit test turns namespace 5 red.

3. `heavy cargo nextest run -p skein-ui-text --lib -E 'test(=a_terminal_owns_its_namespace_alone)'` (with its module
   path), then `heavy cargo nextest run -p skein-ui-text -p skein-ui-view-example`, naming `skein/examples/ui/view`'s package
   by its manifest.

4. The readers' tests: `heavy cargo nextest run -p temper-client-shell-terminal`, then the packages of `jig/tests/client` and
   `jig/tests/client-protocol`, named by their manifests.

5. The debt and status lines. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-text
   --all-targets -- -D warnings`; `python3 -I -B tools/debt/debt.py`; the citations check. Commit and push.

## Report

The common note's items, and what you checked of the two readers' tokens.
