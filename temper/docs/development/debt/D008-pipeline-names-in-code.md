# D008: Code names the old pipeline's tasks and rulings

- **Kind:** naming
- **Where:** `temper-service`, `temper-forge-setup-shell`, `temper-forge-setup`, `temper-engine-domain`, `temper-engine-smith`, `temper-protocol-records`, `temper-client-domain`, `temper-client-shell-web`, `temper/tests/engine/forge`, `temper/crates/temper/tests/support`
- **Seen:** 2026-11-03 at `41d63da8c3`, while checking D005 and D006 (new here)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (comments a reader can act on; nothing private in public code)
- **Size:** S
- **Status:** open
- **Paid when:** no comment, `expect` reason or panic message in temper names a pipeline task (`temper-NN`), a session's ruling time or a commit as history; each says what is true now, or the gap is in its status file (the measure is 0).
- **Measure:** `rg -c --type rust 'temper-[0-9]{2}' temper | awk -F: '{s+=$2} END {print s}'`: 22

## What

AGENTS.md (section 2) asks for comments that say what is true now, with
no history, and for public code that names no session. temper's code
names the pre-fab pipeline's tasks 22 times, as promises or as history:

- "temper-15 replaces them with records" (`temper-service/src/forge.rs`),
  "temper-15 removes the forge files" (`temper-forge-setup-shell/src/reconcile.rs`, twice),
  "temper-15 retains old-form access; 14a2 moves the template"
  (`temper-service/src/setup/lifecycle.rs`), "temper-15 removes this old
  form and temper-01a's Adopt stand-in" (`temper/crates/temper/tests/support/owned.rs`);
- "temper-07 consumes it" (`temper-engine-domain/src/boundary.rs`),
  "temper-07 handles the requesting client" (`temper-engine-domain/src/route.rs`),
  "temper-07 adds not_installed to the client wire" (`temper-service/src/family.rs`);
- "temper-10 supplies picker.page" (`temper-forge-setup/src/limits.rs`),
  "temper-13 tells the root and the person" and "temper-14 decides
  whether the draw stays" (`temper-forge-setup-shell/src/reconcile.rs`),
  "temper-01 adds the configuration row" (`temper-service/src/limits.rs`),
  "temper-05 connects GitHub" (`temper/tests/engine/forge/src/translate.rs`, twice);
- in strings that ship: `unreachable!("temper-12 gives merge queue answers meaning")`
  (`temper-engine-smith/src/answers.rs`), an `expect` reason "temper-01
  requires Display" (`temper-protocol-records/src/forge.rs`) and a panic
  text naming "temper-01/13" (`temper-forge-setup/src/place.rs`).

Beside them, two comments cite a session's ruling by its time:
"Interim view counts until jig owns receiving counts (owner, 17:51Z;
x1g)" (`temper-client-domain/src/limits.rs`, see D009) and "16:52Z
ruling" (`temper-client-shell-web/src/host_tests.rs`), and one cites a
commit as history ("Since 75e20fd7 skein's reconcile draws no secret",
`temper-forge-setup-shell/src/reconcile.rs`).

The comments' emphasis words ("actual", 65; "genuine", 9) are counted by
`temper/scripts/old-forms.txt` and not this item.

## Why it matters

The pipeline that numbered those tasks is gone (AGENTS.md, section 8):
nobody can look a promise up, and some promise what no plan will do.
They mark real gaps, which belong in status files, where they are kept.

## What pays it back

Most go with the code they mark: D005 and D006 remove the temper-15 and
temper-14 ones. The rest are rewritten to say what holds now, each
open promise checked against its status file and added there if
missing. One task.
