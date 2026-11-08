+++
id = "tui-redact"
part = "tui-front"
kind = "feature"
needs = ["tui-seam"]
files = [
  "temper/crates/temper-client-shell-tui/src/journey/redact.rs",
  "temper/crates/temper-client-shell-tui/src/journey/scan.rs",
  "temper/crates/temper-client-shell-tui/src/journey/redact/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/scan/tests.rs",
  "temper/docs/design/client/tui/journeys.status.md",
]
effort = "high"
minutes = 45
+++

# A `box-line` record's printed link is redacted and scanned as a link original

## Goal

The welcome draws the box's printed link, and a recording in front keeps it in its `box-line` records (`tui-seam`). Make its secret a *link* original, as a `found` record's is, so that `add` replaces it in every field and every frame, and its last scan refuses it anywhere left (`temper/docs/design/client/tui/journeys.md`, section 5, *The TUI's originals* and *A pasted link's encoding*).

Acceptance, by `notes/contracts.md`, section 8, *The recording*, and journeys.md, section 5:

1. **The original:** each `Line::BoxLine(bytes)` that `BoxLine::read` reads as `Link { url }`, whose URL `jig-client-protocol-http`'s `printed` reads under the prefix `/jig`, gives its secret as a `Kind::Link` original, as `Line::Found(Some(bytes))` does in `redact.rs` (about line 138); a `box-line` of another word, or a link that doesn't read, gives none.
2. **Its encodings:** the secret's canonical URL-base64 in the `box-line`'s URL is replaced by the redacted secret's, as in a `found`'s, so that the replay reads the line back and the welcome draws the redacted link; the same encodings are sought by the scan (`scan.rs`).
3. **Tests,** a failing test first:
   - `journey/redact/tests.rs`' `found_links_are_redacted_even_when_never_sent` (about line 344): a row, or a second walk in it, for a `box-line` link never sent: its secret is in no field after the redaction, the line still reads as a `Link`, and a mark's print that drew it whole is redacted too.
   - `journey/redact/tests.rs`' `each_original_is_replaced_and_scanned_in_each_field` (about line 71): a `box-line` row among its places.
   - `journey/scan/tests.rs`: a row refusing a `box-line` link's secret left in a field (its kind named, never its bytes).
   Name the link in words and compare by equality; never print it (`notes/common.md`, *Never shown*).
4. **Status:** delete the **Not yet** bullet naming `tui-redact` in `temper/docs/design/client/tui/journeys.status.md`. No other design line.
5. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-tui -E 'test(/journey::/)'`, then the crate whole once; `cargo nextest run -p temper-client-journeys` (the committed journeys still redact and replay as they stand).

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, section 8.
- `temper/docs/design/client/tui/journeys.md`, section 5 (and 8.3, *The welcome's link, wrapped*: not this task's; a link wrapped across a frame's rows is left to 8.3, whose status says so).
- The code: `journey/redact.rs` (field table about 60-90, the originals about 120-160), `journey/scan.rs`, their tests; `tui-seam`'s `Line::BoxLine`.

## Owns and may change

Owns the four journey files listed and the status bullet. Not `host.rs`, `run.rs`, `line.rs`, `record.rs` or `replay.rs` (`tui-seam`'s, landed); not `temper-client-shell-web`'s shared redaction (the *link* kind is already there).

Options, each decided:

- **Where the original is taken:** beside `Found`'s arm, with the same helper; if the helper is `Found`'s alone, generalize it to take the URL's bytes and say so.
- **A wrapped link in a frame** (a print at 80 columns cuts the URL across rows): out of scope; its proof fails and nothing is written (journeys.md, section 5, *What is left to the person*; 8.3). Don't add the across-the-wrap search.

## Steps

1. The failing tests of acceptance 3, seen failing; then `redact.rs` and `scan.rs`; the narrowest test after each.
2. The status edit; the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: a recording in front keeps the box's printed link, which `add` must never commit), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each test added and the rule broken by hand for it (for instance the `box-line` arm dropped).
