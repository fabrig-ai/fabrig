+++
id = "issuer-sockets"
part = "issuer"
kind = "feature"
needs = []
files = ["temper/tests/real-loop/tests/suite/sign_in.rs", "temper/tests/real-loop/src/issuer.rs", "temper/tests/real-loop/src/world.rs", "temper/tests/real-loop/src/calls.rs"]
effort = "high"
minutes = 120
+++

# A browser's held issuer sockets don't block a sign-in, in the real loop

## Goal

The process walk
`sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create`
(`temper/crates/temper/tests/fake_backends/walk/sign_in.rs`) guards a bug
a walk found: the fake issuer served two connections, so a browser's
kept-alive authorize socket and an idle spare left the engine's token
request no room, and the sign-in hung. Its fix (`8cba1c07a3`) gave
`temper_fake::oauth::Document::peer` its own `CONNECTIONS`, 8. This task
holds that half in the real loop (`temper/tests/real-loop`, package
`temper-real-loop-world`, where real sockets on one ring meet the real
service), on the production issuer constructor; `issuer-log` holds the
logged turn-away. The walk itself is `walks-go`'s; you never touch it.

Acceptance:

- **`a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away`**,
  new, in `temper/tests/real-loop/tests/suite/sign_in.rs`, beside
  `a_terminal_sign_in_returns_to_the_real_listener`:
  1. The issuer is the one `temper_fake::oauth::Document::peer` makes,
     topped up by `temper_fake::oauth::refill` before each of its
     passes, as `temper::fake`'s host does: never a copy of its limits,
     never the world's `issuer::peer`. The service's provider
     registration points at it.
  2. The person starts a sign-in; the browser's authorize request is
     sent without `Connection: close` and answered 302, and its socket
     is **held open**; a second, idle socket to the issuer is opened and
     held.
  3. With both held, the callback is followed and the engine's token
     exchange completes: the account is usable (the world's existing
     checks for a sign-in: exactly one exchange, one usable sign-in
     source).
  4. Still holding them, `CONNECTIONS + 1` more sockets are opened: the
     issuer's `rejected()` count rises above zero (the peer's own count,
     read from the issuer's process in the world).
  5. The sockets close; the world drains as every story's does.
- **Red first:** change `connections: CONNECTIONS` back to `connections:
  PLAN_SLOTS` in `Document::peer`
  (`temper/crates/temper-fake/src/oauth.rs`, about 114) by hand, run the
  story alone, see it fail at step 3 (the sign-in never completes; the
  world's patience names it), restore, see it pass. The report quotes
  the failure.
- Every existing real-loop story still passes: the harness change keeps
  `world::run`'s other callers as they are.
- Clippy clean, fmt clean, `temper-real-loop-world`'s focused suite
  green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the idle-sockets
  ruling.
- `temper/docs/design/domain/testing.md`, 2.4 (*What runs*, *LLM
  accounts' stories*: a sign-in), and 2.6, the bullet **The fake box
  keeps two walks**, its **The fake issuer's held sockets** item.
- `temper/docs/design/protocol/setup.md`, 7.5, and its status's entry on
  the idle-socket walk (`temper/docs/design/protocol/setup.status.md`,
  about 108), for why a browser holds those sockets.

What the survey found, for your bearings (verify before relying):

- `world::run` reserves a loopback port, starts
  `ServiceHost::with_issuer`, and spawns the issuer process from
  `crate::issuer::peer(address, &crate::issuer::registration(address))`,
  a skein OAuth peer with `connections: 2` of its own
  (`temper/tests/real-loop/src/world.rs`, about 196 to 250;
  `src/issuer.rs`). `run_sign_in` (`src/lib.rs`, about 153) is the
  sign-in story's entry.
- `Step::Authorize`, `Step::Return`, `Caller::sign_in_http` and
  `issuer::Visit` (`src/calls.rs`, about 1177; `src/issuer.rs`, about
  153 to 189) make real socket exchanges on the ring, but `Visit` sends
  `Connection: close` and closes each socket, so a held one needs a step
  of its own.
- `temper_fake::oauth::Document::peer(seed)` builds its `Config` from
  the document (`listen`, `redirect_port`, `redirect_path`); its
  `refill` tops up plans, which bound no connection.
- The world runs on one thread and one ring: a socket the story holds is
  opened on the ring (a step or the issuer visitor), or is a
  non-blocking std socket the referee polls; never a blocking read on
  the loop's thread.

## Owns and may change

Owns:

- `temper/tests/real-loop/tests/suite/sign_in.rs`: the new story.
- `temper/tests/real-loop/src/issuer.rs`: a constructor of the
  production issuer for the world (from a `temper_fake::oauth::Document`
  whose listen and redirect agree with the registration, or a
  registration made from the document), and a held-socket visit.
- `temper/tests/real-loop/src/world.rs`: a way for a story to choose the
  production issuer and refill it each pass, and to read its
  `rejected()` count (for instance a `run` variant, or an issuer kind
  in the world's `Kind`); its other callers unchanged.
- `temper/tests/real-loop/src/calls.rs`: a step that holds an issuer
  socket open, if the visit belongs there.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/real-loop/src/lib.rs`: an entry beside `run_sign_in`, if
  the story needs one.
- `temper/tests/real-loop/Cargo.toml`: none expected (`temper-fake` is
  already a dependency).

Pre-approved options:

- **The rejected count:** if the world can't read the issuer's count,
  observe the turn-away from the socket's side (the extra socket is
  closed or reset by the issuer without an HTTP answer) and say so.
- **The registration:** if `Document::peer`'s config can't agree with the
  world's registration as `temper_fake::oauth` stands, add one public
  constructor or accessor to `temper_fake::oauth::Document` (for
  instance one from a listen address and a redirect), leaving
  `Document::peer`'s limits and `refill` as they are, and name it in the
  report; `temper/crates/temper-fake/src/oauth.rs` is then yours for that
  item alone.

## Steps

1. Read the inputs, `suite/sign_in.rs`, `src/issuer.rs`, `src/world.rs`
   around `run`, and `temper_fake::oauth`.
2. Build the production issuer into the world; run the existing sign-in
   story alone on it unchanged to see the harness hold:
   `heavy cargo nextest run -p temper-real-loop-world --test suite -E
   'test(=sign_in::a_terminal_sign_in_returns_to_the_real_listener)'`.
3. Write the new story; run it alone (the name above).
4. The red: `PLAN_SLOTS` as above, run it alone, see it fail at step 3,
   restore, green.
5. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-real-loop-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-real-loop-world` (check the count; its test
   group runs two at a time). Commit (`walks-lower: held issuer sockets
   leave a sign-in room, in the real loop`, the body saying why: the
   fake issuer's capacity is the issuer's and the engine's, met on real
   sockets in one loop, not a box walk's) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the red and its failure; how the story holds a
socket and reads the turn-away; the harness lines changed; the story's
CPU seconds as nextest reports them; the minutes.
