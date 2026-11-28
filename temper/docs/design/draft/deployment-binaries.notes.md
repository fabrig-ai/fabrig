# deployment-binaries: the reasons behind it

This holds the reasons for `deployment-binaries.md`, moved from the shared notes file.

The reasons behind `deployment-binaries.md`, from the user, 2026-11-08.

- **Two sets, two jobs:** "The current one at https://fabrig.ai/temper/... would be the "try temper via a box" one, stripped and gzipped (I'll update the site accordingly), always pointing to the latest release (main for now). The new set would be the "real deployment" one, under https://fabrig.ai/temper/v{MAJOR}/..., comprised of these binaries (gizip'ed, but not stripped): "temper" just the TUI, "temper-engine" engine plus web, "temper-worker" worker process, "temper-agent" smith. I think that'd be it for a full deployment, unless I'm missing something."
- **Not stripped:** the draft reads it as keeping a deployment's crash reports readable, while a trial download's size matters more. That reading is the draft's, not the user's words.
- **No new CI:** "Our development process stays unchanged for now, no special build jobs for these binaries for now, build-level breakages would be caught when I ask for the one-off release build."
- **The name `temper`** in both sets is the user's to settle (`reference-code-production.notes.md`, section 5 of the draft).
- **`ctl` beside the TUI:** the user asked for "just the TUI". The draft puts `temper ctl` in the same binary, since it's a client too and a deployment needs one. That's the draft's reading.
