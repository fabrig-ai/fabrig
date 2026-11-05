# brief-failure: what every task holds to

- **The design is settled:** `jig/docs/design/domain/engine.md`, 9.1 says which failure counts how, `temper/docs/design/domain/forge.md`, 8.4 which forge read says what. `notes/contracts.md` holds the shapes that cross tasks. Follow them; a case neither names is a `Question:`.
- **A reason is fixed words.** It carries no run's words, no LLM's, no secret, no name out of a forge's answer. In the kit's types its parts are `&'static [u8]`, so this holds by construction: never build one from data.
- **Formats change in place** before v0.1.0: `tasks-v9`, `client-v19` keep their numbers. Generated codecs and goldens come from their own tools (`skein-codegen`; commit `9ef746a800` is a recent field added through both), never by hand.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and your report gives the measurements. The brief's deadline, the retries and their allowances keep their values.
- **fabrig/ui has priority.** Its drafts are changing `temper-client-domain`, `temper-client-view`, `temper-client-shell-*`, `temper/tools/gallery`, `temper/tools/journeys`, `temper/tests/client*` and `temper/tests/one-process`. Leave those files alone but for a literal the compiler forces, named in your spec. No phrase, page or look of temper's client changes here.
- **`jig/crates/jig-domain-notes` and `jig/crates/jig-protocol-store/schema/core-v6.schema`** are moving on another draft: this plan needs neither.
- **hearth is untouched:** its manifests name no jig crate, and `skein-forge-client` is not on its surface.
- **No test reaches a real host,** and no probe touches a real service (`AGENTS.md`, 3). A fixture of a real answer carries no real address, token or name: `tests/boundary/tests/outward.rs` checks it.
- **A change of meaning runs its readers' tests.** Whoever changes how a brief's failure ends also runs `temper-engine-domain`'s package and world tests, which have stories that lean on the pause.
