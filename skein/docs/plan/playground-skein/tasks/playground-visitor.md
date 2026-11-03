+++
id = "playground-visitor"
part = "visitor"
kind = "feature"
needs = ["playground-crate", "ui-web-surface"]
files = ["skein/crates/skein-playground/src/visitor.rs", "skein/crates/skein-playground/src/visitor/**", "skein/tests/playground/src/site.rs", "skein/tests/playground/src/site/**", "skein/tests/playground/tests/suite/visitor.rs", "skein/tests/playground/tests/suite/visitor/**"]
effort = "high"
minutes = 180
+++

# skein-playground: the visitor

## Goal

The visitor, a `Host` of the world that speaks HTTP to the application's listener as a
browser's same-origin fetch would: an exchange a connection of its own, the stream one
connection read a piece per read-on, the `Host`, `Origin` and the jar's cookies added, and
fetch's reports out (`skein/docs/design/playground.md`, 4). Its world, `tests/playground`,
holds it against a scripted server on skein's HTTP.

Acceptance:

- `skein_playground::visitor` holds exactly `notes/contracts.md`, section 8.0: `Config`,
  `Visitor` with `new`, `enter`, `take`, and `impl skein_shell::Host for Visitor`.
- **Its inputs** (`bridge::Input`, contracts section 1), entered between steps, acted on at
  its next turn (`work_pending` is true while one waits):
  - `Exchange`: a connection of its own to `config.listener`, one request (the exchange's
    method, its path as the target, its headers, its body with `Body::Length`, `close:
    true`), its answer's body read up to the exchange's bound, then the connection closed;
    `Answered { exchange, status, body }`, or `Failed { exchange }` past the bound, on a
    refused connection, a reset, any client error, or past `config.exchanges` in flight.
  - `Abandon { exchange }`: its connection closed (aborted if a close would wait); nothing more
    is reported of it.
  - `Stream(Open { path, headers, bound })`: one connection, a GET; `StreamOpened` for 200,
    `StreamRefused { status }` otherwise; then its body as `StreamPiece`s of at most `bound`:
    the first as it comes, each further after a `ReadOn`; `StreamEnded` at its end, on a reset
    or a refused connection (with no `StreamOpened` before it), and after `Close`. One stream
    at a time: an opening while one is open closes the first, as fetch's `open_stream` does.
  - `Stream(Close)` and `ReadOn` as fetch's (`skein/docs/design/ui-web.md`, 10).
  Keep the exchanges and the stream in `skein_ui_web::fetch::Fetch`, the browser's pure
  bookkeeping, so the reports are fetch's by construction (playground.md, 4, "Fetch's reports
  by construction").
- **What it adds** (playground.md, 4, "The headers"): `Host: <config.host>`;
  `Origin: http://<config.host>` on every method but GET and HEAD; one `Cookie` header joining,
  `name=value` by `; `, the jar's cookies whose path the request's path-matches, in the order
  they were set; none when it holds none that match.
- **The jar** (playground.md, 4, "The jar" and "What the jar reads"): every `Set-Cookie` of
  every answer, the stream's opening among them; name and value, `Path` (absent: the request
  path's directory), `Max-Age` from the answer's wall (`0` or less clears it); expiry checked
  against the world's wall at each request; past `config.cookies` or `config.cookie_bytes`,
  refused. The jar is a private module (`visitor/jar.rs` or the like) with step tests in its
  module's `#[cfg(test)]` file under `src/visitor/`.
- **Bounded:** `Fetch::new(config.exchanges)` in flight; one stream; the jar; each body by its
  bound; the io and HTTP limits the config gives. No growth past them
  (`skein/docs/foundation/programming-model.md`, 6).
- **The site world** `skein/tests/playground/src/site.rs`: a process enum of the visitor and a
  scripted server Host over `skein_http::server` on io (no generic server Host exists:
  write a small one, modelled on `skein/crates/skein-fake-peers/src/face.rs` and the routes of
  `skein/examples/ui/server`), its routes scripted per test (a status and body; headers to
  set; the request's headers echoed in the body; a chunked body in pieces over time; an
  answer that never ends, for abandons); a referee that enters inputs into the visitor at
  scripted moments through `Visitor::enter` and gathers `Visitor::take`'s reports; built with
  `skein_world::World` and `skein_sim::Config::calm()`, the listener at a fixed loopback
  address.
- **The world tests** in `tests/suite/visitor.rs` (playground.md, 8, "The visitor"), each over
  a few seeds, each failure naming its seed:
  - `an_exchange_is_answered`: status and body, a GET and a POST with a body;
  - `an_answer_past_its_bound_fails_the_exchange`;
  - `an_exchange_past_the_bound_in_flight_fails`;
  - `a_refused_connection_fails_the_exchange` (no listener at the address), and
    `a_refused_connection_ends_the_stream`;
  - `an_abandoned_exchange_reports_nothing` (its connection closed: the server sees it);
  - `the_stream_reads_one_piece_per_read_on`: pieces of at most the bound, no piece without a
    read-on after the first, then its end;
  - `a_refused_stream_reports_its_status_then_its_body`;
  - `the_host_and_origin_are_the_applications`: echoed: `Host` on every request, `Origin` on
    a POST and not on a GET;
  - `the_jar_sends_a_cookie_under_its_path`, `a_cleared_cookie_is_not_sent`,
    `an_expired_cookie_is_not_sent` (the clock moved past its `Max-Age`),
    `a_cookie_past_the_jar_is_refused`;
  - `the_visitors_peak_is_bounded`: every bound full at once (exchanges in flight,
    bodies at their bounds, the stream, the jar), under `Memory::Within(CEILING)`, the
    ceiling the measured peak plus a quarter, as `skein/tests/world/tests/suite/world.rs`
    sets its own.
- `heavy cargo nextest run -p skein-playground-world -E 'test(/visitor::/)'` and
  `heavy cargo nextest run -p skein-playground` pass, their counts in the report; clippy over
  both packages and `heavy cargo clippy -p skein-playground --target wasm32-unknown-unknown -- -D warnings`
  are clean; the world tests sit within the default profile's budget
  (`skein/docs/foundation/testing-strategy.md`, 8.1).

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1 and 8.0.
- `skein/docs/design/playground.md`, 4 and 8; `skein/docs/design/ui-web.md`, 10;
  `skein/docs/design/world.md`, 5 (input between steps); `skein/docs/design/http.md` for the
  client's and server's records.
- Models to read, not to depend on: `skein/tests/oauth-accounts/src/sign_in/browser.rs` (a
  scripted HTTP client Host over io and `skein_http::client`), `jig/tests/protocol-accounts/src/visitor.rs`
  (the same, one GET per connection), `skein/crates/skein-fake-peers/src/forge/delivery.rs`
  (connections bounded in flight, io's errors mapped), `skein/tests/llm-connection/src/simulated.rs`
  (`io::stage::up` and `down`, a two-process world), `skein/crates/skein-ui-web/src/fetch.rs`
  and `fetch/web.rs` (fetch's bookkeeping and its use).

## Owns and may change

- Owns the files in the header. `skein/crates/skein-playground/Cargo.toml` and
  `skein/tests/playground/Cargo.toml` may gain a dependency line the visitor or its world
  needs (`skein-fake-peers` for its `Face`, say), each listed in the report; nothing else of
  either.
- Read only: `skein-ui-web` (its `fetch::Fetch` is used as it stands; a change it needs is a
  question), `skein-http`, `skein-io`.
- Pre-approved: if `fetch::Fetch`'s ticket for the stream does not fit a connection that
  closes on its own, keep a private map beside it and say so; if skein's HTTP server needs a
  route type the tests' server writes better itself, write it in `site.rs`.

## Steps

1. The jar, pure, with its step tests.
2. The visitor's connections: an exchange, then the stream; its `Host` impl.
3. The site world and the world tests, one by one; the peak last, its ceiling measured.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs; the citations
   check.
5. Commit (why: the visitor carries a page's fetch into the world as a browser would); push
   your branch.

## Report

As the implementer's text says, and also: the measured peak and the ceiling, and the
visitor's states in a few lines.

## Answers

- 2026-11-03. Q: rename the memory test, or allow an old-form exception for
  `the_visitors_worst_case_is_bounded`? A: rename it to
  `the_visitors_peak_is_bounded`; the test is otherwise as specified (every
  bound full at once, under `Memory::Within(CEILING)`, the measured peak plus a
  quarter). `worst_case` is an old form (`skein/scripts/old_forms.py`) whose
  count only falls; no exception, and the baseline stays as it is. The name
  in the acceptance above is amended.
