+++
id = "sign-in-dns"
part = "sign-in-dns"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-shell/src/serve/accounts_runtime.rs", "temper/crates/temper-engine-shell/src/serve.rs", "temper/crates/temper-engine-shell/src/serve/tests.rs", "temper/crates/temper/tests/process.rs", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/domain/testing.status.md"]
effort = "medium"
minutes = 60
+++

## Goal

A box made by `temper init --box` on a machine without DNS refuses to
start: its LLM provider's sign-in registration names `auth.openai.com`,
and `temper-engine-shell`'s `accounts_runtime` resolves each sign-in host
at start and returns the lookup's failure. The design now holds such a
host at `127.0.0.1:0`, so the start goes on and that provider's
sign-in exchanges fail at their connect until a start resolves it. Build
that, and the box's no-DNS process story runs again.

Acceptance:

- `accounts_runtime::resolve` takes its lookup as a parameter, as
  `github_runtime` does (sketch), and a host whose lookup fails or gives
  no address gets the held address, `127.0.0.1:0`, with its transport as
  configured. Everything else it refuses now, it still refuses, with the
  same words: a URL that is not `http` or `https`, a bad port, a server
  name or trust file that is wrong.
- `serve.rs`'s `accounts` passes the machine's lookup
  (`std::net::ToSocketAddrs`, first address).
- A new lib test in `temper-engine-shell`, `serve/tests.rs`,
  `accounts_runtime_holds_unresolved_hosts`, beside
  `github_runtime_holds_unresolved_hosts_and_uses_the_machine_socket`
  and built the same way, with a lookup that resolves nothing: a
  provider whose sign-in registration's token, issuer, keys and
  revocation hosts don't resolve gets the held address for its endpoint
  and each origin, and the same origins it gets with a lookup that
  resolves them; with a lookup that resolves them, the addresses are the
  lookup's; a scheme other than `http`/`https` still refuses. No real
  name is looked up in a test: every lookup in it is the test's own
  function.
- `temper/crates/temper/tests/process.rs`,
  `a_default_box_with_a_sign_in_registration_starts_without_dns`: no
  `#[ignore]`, and it passes run by name and alone in its profile
  (common notes): `heavy cargo nextest run -p temper --profile
  process-temper --test process -E
  'test(=a_default_box_with_a_sign_in_registration_starts_without_dns)'`,
  count 1. Nothing else of `process-temper` runs. Its assertions don't
  change, but this one, which names the old cause:
  `"no-DNS start must reach serve's listening; the current sign-in
  endpoint resolution refuses it: {diagnostic}"` becomes `"no-DNS start
  must reach serve's listening: {diagnostic}"`.
- `heavy cargo nextest run -p temper-engine-shell` and the focused
  tests of `temper` (`heavy cargo nextest run -p temper`) pass.
- The status entries below are gone.

## Inputs

- `temper/docs/plan/ignored-misc/notes/common.md`: every rule there
  holds.
- `temper/docs/design/protocol/configuration.md`, section 2 (*A
  sign-in's hosts*), the bullet this cut wrote.
- `temper/docs/design/protocol/setup.md`, 8.3 (*No forge*): the held
  forge, whose address is the same `127.0.0.1:0`.
- skein's `skein/docs/design/oauth.md`, 6.2: the driver takes each
  address resolved before its loop and resolves nothing while it runs;
  the held address is one so given.
- `temper/docs/design/protocol/sign-in.md`, 3.1 is Google's sign-in, not
  this one: `google_runtime.rs` still refuses a host that does not
  resolve, and stays as it is.

## Owns and may change

- Owns: the files in the header.
- Sketch, binding the outer layer:

  ```rust
  // temper/crates/temper-engine-shell/src/serve/accounts_runtime.rs
  /// A sign-in host that did not resolve at start (temper/docs/design/protocol/configuration.md,
  /// section 2): loopback port 0, which no listener can hold, so its exchanges fail at their connect.
  const HELD: std::net::SocketAddr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
      std::net::Ipv4Addr::LOCALHOST, 0));

  pub(super) fn resolve(
      engine: &config::Engine,
      lookup: fn(&str, u16) -> Option<std::net::SocketAddr>,
  ) -> Result<service::AccountsRuntime, String>;
  ```

  The held address is chosen where `endpoint_for_origin` turns a lookup
  into an address; the rest of the module keeps its shape.
- Options, each pre-approved:
  - If `temper_service::forge::HELD_ADDRESS` is the same value and
    reads well here, you may use it in place of a local `HELD`; its doc
    then says it serves both, in one line.
  - If the process story, past the start, fails on another lookup (a
    host other than a sign-in's), that is a finding of its own: keep the
    `#[ignore]` with a reason naming the host and the member that names
    it, report it first, and keep the code change and its unit test.
  - If the story is past a slow period or its timeout, report its time
    and the cause you find; never change its timeout.
- Not yours: `google_runtime.rs`, `github_runtime`, skein's accounts
  driver, and every other ignored test.
- Status entries, in these words:
  - `temper/docs/design/protocol/configuration.status.md`: the last
    bullet, which begins `- **Not yet:** 2: a sign-in's host that does
    not resolve at start refuses`, is removed whole.
  - `temper/docs/design/domain/testing.status.md`: the last bullet,
    which begins `- **Not yet:** 2.6:` and names
    `a_default_box_with_a_sign_in_registration_starts_without_dns`, is
    removed whole.
  - Each file's date line becomes the day you commit (`date -u +%F`).

## Steps

1. The lookup parameter and the held address; `serve.rs`'s call.
2. The lib test. Break it once (return the lookup's failure again),
   see it fail for the held host, restore.
3. Remove the story's `#[ignore]` and its stale words; run it by name
   and alone, as above.
4. The status entries.
5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-shell -p temper --all-targets --
   -D warnings`; the focused tests in the acceptance.
6. Commit, run the citations check, push.

## Report

The implementer's report, with: the process story's result and time;
the unit test and what broke it; any option taken.
