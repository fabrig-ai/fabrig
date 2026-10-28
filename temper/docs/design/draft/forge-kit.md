# The forge kit: the connector's top

Draft, 2026-10-18. The forge kit's crates are skein's (skein's
`forge.md`, section 2). temper keeps the connector,
`temper-engine-domain-forge`: the client's owner, whose top is the
service's (skein's `forge.md`, section 2.3). Part of that top is
neutral. This draft says where a later move would cut it.

## 1. In one page

- **Not cut now.** The top stays temper's until a second application
  that works on a forge is designed.
- **The seam** (section 2): resources, holds, capabilities, routing to
  the client and the topics' mechanism are neutral; changes, issues,
  gates, settling, judging and adoption's checks are temper's.
- **Where it would go** is open (section 3).

## 2. The seam

A later move separates the following in `temper-engine-domain-forge`.

- **Neutral:**
  - resources and holds (`held.rs`, `resources.rs`);
  - capabilities (`capabilities.rs`);
  - routing to the client, and keeping its rows;
  - the topics' mechanism;
  - most of `domain.rs`.
- **temper's:**
  - `change` and `issues`;
  - the top's `gates.rs`, `settle.rs` and `judge.rs`;
  - `domain.rs`'s change and issue parts;
  - adoption's checks, in `adoption.rs`;
  - `Repository`'s project, home, prefix, CI and checks fields, in
    `boundary.rs`;
  - the effects offered to agents (`describe_agent_effect`);
  - the `Stored`, `Key`, `Event` and `Request` enums, which carry both
    sides and are split along the line above.
- **Its one use of jig:** `jig_domain_people`'s `Outcome`, `Refusal` and
  `Again`, in adoption (`boundary.rs`, `adoption.rs`). A move replaces
  them with the top's own three-way outcome and an opaque refusal, which
  the root maps, as it maps the top's other words: the forge's crates
  name no type of jig's core (`domain/root.md`, section 1).
- **The root's translation** (`temper-engine-domain`'s `route/`) maps the
  top's words onto jig's connector words. Every application writes its
  own.
- **When:** once a second application that works on a forge is
  designed, such as factory.md's lab.

## 3. Open questions

- **Where the neutral top goes:** a crate of skein's forge kit, or of
  jig's. Default: decided with the second application's design; until
  then it stays in `temper-engine-domain-forge`.
