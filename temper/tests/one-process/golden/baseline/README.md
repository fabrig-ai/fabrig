# Baseline deployment

Made with the production code at **bc23255eb7** by
`golden::baseline_making_story_stops_a_whole_deployment` in
`temper-one-process-world` (`tests/suite/golden.rs`, support in `src/golden.rs`).
The kept schema-text-only changes do not alter what that build writes.
**Nothing writes this directory again.** The making test exports only into its
own `CARGO_TARGET_TMPDIR/baseline-making-story`; it is not a fixture generator.

The person follows the printed sign-in link, starts the playground, adds a
second provider's deployment API-key account through the account form, and
asks for Story A's CSV change. The change is pushed, checked, reviewed and
landed on the fake forge. Its chat replies and waits. A second chat delegates
a coding change; its editor reads the repository and is stopped mid-run by a
signal. The service and worker drain, recording that attempt's Recheck.
All credentials and identities here belong to the world's fakes.

`state/`, `worker/`, `client/` and `setup/` hold every root's files.
`configuration.json` is the world's service-start input, including its inline
catalogue and test limits; the composition strips the catalogue into its typed
defaults just as for other world starts. `modes.txt` records every node as
`<d|f> <octal mode> <owner> <machine path>`, including empty directories. Git
keeps neither empty directories nor private mode bits: the loader recreates
them, including the service user's 0600 secrets map. A physical `dot-git`
directory represents a machine `.git` component, which Git cannot track.
The manifest retains the machine's spelling.

Copied once using a temporary tar/gzip/base64 test-output transport, removed
from the test afterwards. Transport archive SHA256:
`bf24da28b03ed5c594ff6a9ada56d2a13c750796256fe3a1f0200231c472e273`.
Only the host-path escaping and node-kind manifest metadata were added after
copying; no root file's bytes were changed.

The focused reader is `golden::baseline_golden_store_survives_upgrade`. It
loads a fresh machine, uses this build's service and patch sets, and compares
`Note::Patched` lines with `patched.txt` in order. Its current ctl client uses
the kept terminal token (only ctl's loopback address file is added to the
loaded copy), reads the old chat's explicit turns, and creates a new chat.
`patched.txt` is empty at the baseline; a patch landing adds its reviewed line.
