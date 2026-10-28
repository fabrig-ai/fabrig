# jig

The kit for agentic applications; temper is its first application. Read the root `AGENTS.md` first, then `jig/README.md` and `jig/docs/design/README.md`.

- **The design:** `jig/docs/design/`, cited from code at module or type level, never on every field.
- **Every crate is step code under the strict subset** (`skein/docs/foundation/programming-model.md, 10`), except jig's test kits (ordinary Rust, like every world) and the client's shells (ordinary Rust, like skein's shell). No traits and no generic items in step code: a mechanism that must be generic, such as the journal (`jig/docs/design/README.md, 6.2`) or the client's request table (`jig/docs/design/client/domain/root.md, 4`), goes into skein-lib before its jig consumer.
- **jig owns every invariant it promises.** An application's root only routes and translates (`jig/docs/design/README.md, 6`); a change that would make it decide or order something is a change to jig's design first.
- **The core never depends on smith.** Only the charter's translation into smith's terms, the client's decoding of turns (below its domain), and an application's roots that compose smith's agent capability, with their protocol layers, do. The hub, `jig-domain-host`, carries smith's payloads as bytes it never reads; it, the core and its children, and the client's domain and view link no smith crate. An application without agents never links smith (`jig/docs/design/README.md, 9`; `jig/docs/design/domain/hosts.md, 6.8`).
- **No system is named in jig's code:** no forge, repository, branch, pull request, issue or git. Systems are an application's connectors.
- **Markdown read by tests:** `jig/docs/design/protocol/store.md`, compiled into its words tests.
