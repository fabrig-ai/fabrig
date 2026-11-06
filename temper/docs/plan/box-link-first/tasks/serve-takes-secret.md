+++
id = "serve-takes-secret"
part = "serve-takes-secret"
kind = "feature"
needs = ["jig-link-secret"]
files = ["temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/clients/printed.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/tests/clients.rs", "temper/crates/temper/src/lib.rs"]
effort = "high"
minutes = 50
+++

# `temper internal serve --box` takes the first link's secret from its standard input

## Goal

Serve's half of the given secret, with the surface the box's wiring reads. Under `--box`, serve reads its standard input to its end before it starts anything and gives a secret it read to jig's component. temper-service also exposes what the box needs to build the link: whether a link prints, the public origin, and the URL.

Nothing gives serve a secret yet; `box-draws-link` does. Today's behaviour holds with stdin at `/dev/null`.

Acceptance:

- The shapes of `temper/docs/plan/box-link-first/notes/contracts.md`, sections *temper-service* and *serve's standard input*, exactly.
- With a secret given, the service's `links()` queue stays empty at start, and that secret's link signs the owner in.
- A later link, a spent secret's fresh one, is printed as today.
- `temper internal serve --box` with stdin at `/dev/null` behaves as today: the test `suite/startup.rs`'s boxed form at about line 1589 stays green unchanged.
- Plain `temper serve` never reads its standard input.
- `first_link_url` gives the same bytes `printed::link_public` gives, with and without a public origin.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1, *The first link's secret*, and 7.2, *The sign-in block*.
- `jig/docs/design/protocol/client.md`, section 8, *A secret given*.
- `temper/docs/plan/box-link-first/notes/contracts.md`.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns `temper-service/src/settings.rs` (`Seeds`), `clients.rs` and `clients/printed.rs`, `startup.rs` (`Named` and where it is built, about line 577), `temper-service/src/tests/clients.rs`, and in `temper/crates/temper/src/lib.rs` `serve`, `prepare_with_limits` and their callers.

Reader and neighbour files, each only as said:

- `temper/crates/temper-service/src/tests/fixtures.rs`, about line 448: `link_secret: None` in the `Seeds` literal.
- `temper/crates/temper-playground/src/documents.rs`, about line 524: the same.
- `temper/crates/temper/src/worker_alone.rs`, and any other caller of `prepare_with_limits` or `prepare_in_with`: pass `None`, call sites only.

Options, each pre-approved:

- `clients.rs` builds `kit::Settings` twice and rebuilds the component at the first clock (`initial`). Give both the secret: only the rebuilt one's expiry and link count, and the first is dropped with its queue. If that is awkward, give it to the second alone, and say so in the report.
- `Named` has one builder today. If `parsed.engine`'s `clients.link` needs another path to reach it, do what `clients.rs`, about line 151, reads, and say which.

## Steps

In `temper/crates/temper/src/lib.rs`, the outer shape:

```rust
/// The first link's secret on a boxed serve's standard input (protocol/setup.md, 7.1).
fn given_secret(input: &mut impl std::io::Read) -> Result<Option<skein_lib::Secret>, String>;

fn serve(directories: &Directories, boxed: bool) -> Result<Exit, String> {
    // under --box, before listeners are adopted or the store opened:
    // let link_secret = if boxed { given_secret(&mut std::io::stdin().lock())? } else { None };
    // ...
}
```

`given_secret` reads at most `LINK_SECRET_BYTES + 1` bytes, to its end. The refusal's words are in the contract. Never put the bytes in an error.

Write the tests first:

- `temper-service/src/tests/clients.rs`: `a_given_secret_prints_no_first_link_and_signs_in`; `a_spent_given_secret_prints_a_fresh_link`; `first_link_url_is_what_serve_prints` (with and without an origin).
- `temper/crates/temper/src/lib.rs`, its `tests` module: `a_boxed_serve_reads_none_or_a_whole_secret`: empty gives `None`; 32 bytes gives the secret; 31 and 33 refuse.

Narrow runs: `heavy cargo nextest run -p temper-service --lib -E 'test(given_secret) or test(first_link_url)'` and `heavy cargo nextest run -p temper --lib -E 'test(a_boxed_serve_reads)'`. Then fmt, clippy and the focused tests as your text says, with `temper-service`, `temper-playground` and `temper`. Commit and push.

## Report

As your text says, and the place `Named.link` reads from.
