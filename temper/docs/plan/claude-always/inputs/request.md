# claude-always: Claude is always offered on a fresh temper box

At the user's request (2026-11-05, through fabrig/ui): "I just built main, and 'Claude' is still currently not available as option in the welcome journey provider choice, make sure that choice is always available in a fresh 'temper box' run."

## The scope

`temper/docs/design/draft/claude-provider.md`, whole, and its tests. It decides the design; its status lists where the design documents lag.

1. **First, alone, so the bug is fixed early:** sections 1 and 2: `temper init`, with `--box` and without, writes the Anthropic provider, its program, endpoint and model and `documents.lines` at least 3, whether or not `claude` is on `PATH`. The Model step of a fresh `temper box` offers ChatGPT and Claude. The fake backends' rewrite then finds the provider written. Prove it on a real, non-fake `temper box` start with no `claude` on `PATH`, reading the welcome's Model step, not only with init's unit tests. Release this task first and let it land into the trunk before the rest.
2. **Then section 3, fetching Claude Code:** pinned, checked and fetched as Forgejo's binaries are (`temper-forge-setup-shell`'s fetch, `protocol/setup.md`, 6.6), when a command of the source first needs it.

## Decisions already taken (the draft's open questions)

The user gave fabrig/ui the freedom on formats, protocols and interfaces: there is no supported deployment. These are settled; do not ask.

- **Where the pinned build comes from:** the native build at `https://downloads.claude.ai/claude-code-releases/VERSION/PLATFORM/claude`, the file Anthropic's own installer fetches. Pin: version `2.1.311` (the stable channel on 2026-11-05), SHA-256 per platform from its `manifest.json`:
  - `linux-x64` `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada`, 293776752 bytes
  - `linux-arm64` `2e2496c9a8065df93acb0fca2d341f59a6b8777649b2e025ab8c9bcd5eef61f3`, 293658808 bytes
  - `linux-x64-musl` `cd40dd327145ee47aa0deacd571801c1d782388f709d71242eee6b91de8ed11f`, 287442392 bytes
  - `linux-arm64-musl` `172150e3f9ea07e8e2e2e29862d7d9a25f5c234d8e909e4063df237665bf09b9`, 285822152 bytes
  - `darwin-arm64` `85d8c08707fcfd1e78ffa0204eefbc6ec9e4fe22f375fb2767ef85d957ed3161`, 276185504 bytes
  - `darwin-x64` `d34e1cb82015b050471a97574aa6ea6f698ecf3a97a61c980961f43ad9f838f5`, 287420928 bytes
- **Its own updater off:** `DISABLE_AUTOUPDATER=1` in the program's environment.
- **The model:** `claude-opus-5-5` by default. Its window, output and prices as `temper-fake`'s `PROGRAM_MODEL` declares them, or the nearest shipped figures; the figures matter little here, since a subscription bills them to nobody. Note the choice in the draft's status.
- **The "fetching Claude Code" words:** in the Claude way's first step, as the draft's section 3 says.

Record each in the draft and in the design documents its status lists (`protocol/configuration.md`, 12.1; `protocol/setup.md`, 6.6 and 8.2), and bring `claude-provider.status.md` to none lagging.

## How the cut shapes it

- Two to four tasks. No real network in tests: a fake download, as Forgejo's fetch has.
- A change to a stored format, a protocol or a public interface needs no `Question:` in this plan: decide, and note it in `plan.md`, so that it reaches the user.
- **Beside it:** `look-round-4` is landing the frame and Home; the welcome's view is otherwise quiet. Expect catch-ups.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
