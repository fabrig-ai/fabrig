# skein's fakes

Provisional, 2026-11-11.

These documents cover scripted LLM peers, fake forges and their hosted
peers, and local checkout mechanics for worlds. Read the forge's shared
parts before its Forgejo and GitHub layers and the checkout that can use its git route:

- [`llm.md`](llm.md), `skein-fake-llm-domain`, `skein-fake-llm-protocol`: bounded provider-neutral scripts and independent wire peers in place of an LLM provider.
- [`forge.md`](forge.md), `skein-fake-forge-domain`, `skein-fake-forge-http`, `skein-fake-forge-forgejo`, `skein-fake-forge-github`, `skein-fake-peers`: forge state and git, its CI and Actions, protocol layers and loopback hosting.
- [`forgejo.md`](forgejo.md), `skein-fake-forge-forgejo`: Forgejo's API over the shared domain, signed deliveries, the git route and setup.
- [`throwaway.md`](throwaway.md), `skein-throwaway-forgejo`, `tests/forge/conformance`: a pinned real Forgejo on loopback and the dialect's conformance against it.
- [`github.md`](github.md), `skein-fake-forge-github`, `skein-fake-forge-http`, `skein-fake-peers`: GitHub's API and browser faces over the shared domain, signed job-log blobs and hosted peer scenarios.
- [`checkout.md`](checkout.md), `skein-fake-checkout`: deterministic files, scripted commands and local git mechanics, with policy left to the caller's world.
