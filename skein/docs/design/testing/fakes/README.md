# skein's fakes

Provisional, 2026-11-11.

These documents cover scripted LLM peers, fake forges and their hosted
peers, and local checkout mechanics for worlds. Read the forge's shared
parts before its GitHub layer and the checkout that can use its git route:

- [`llm.md`](llm.md), `skein-fake-llm-domain`, `skein-fake-llm-protocol`: bounded provider-neutral scripts and independent wire peers in place of an LLM provider.
- [`forge.md`](forge.md), `skein-fake-forge-domain`, `skein-fake-forge-http`, `skein-fake-forge-forgejo`, `skein-fake-forge-github`, `skein-fake-peers`, `skein-throwaway-forgejo`: forge state and git, protocol layers, loopback hosting, a pinned real Forgejo and dialect conformance.
- [`github.md`](github.md), `skein-fake-forge-github`, `skein-fake-forge-http`, `skein-fake-peers`: GitHub's API and browser faces over the shared domain, signed job-log blobs and hosted peer scenarios.
- [`checkout.md`](checkout.md), `skein-fake-checkout`: deterministic files, scripted commands and local git mechanics, with policy left to the caller's world.
