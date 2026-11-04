+++
id = "forge-tidy"
part = "forge"
kind = "migration"
needs = ["link-surface", "forge-close"]
files = ["skein/crates/skein-forge-connection/src/connection.rs"]
effort = "medium"
minutes = 15
+++

# The forge connection reads the link's face

## Goal

`link_env` is seven lines `Env::with` says in two, the optional body is a `match` `Upload::of` says, and `upload`
has a `Send` arm it can't reach. A refactor: no behaviour changes, and about 12 production lines go.

Acceptance:

- `rg -n 'Upload::new|Down::Send\(_\) => unreachable' skein/crates/skein-forge-connection/src/connection.rs` finds
  nothing.
- `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world` passes, no test changed.
- `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 2
- `skein/docs/design/link.md`, 3.4

## Owns and may change

- **Owns:** `skein/crates/skein-forge-connection/src/connection.rs`.
- **Options:** if an `#[expect]` turns unfulfilled, delete it.

## Steps

1. `link_env`, as the contracts' section 2, reading `self.shared.io.close_timeout`.
2. `self.upload = Upload::of(upload);` in `start`.
3. `upload`, keeping the order of what it does:

   ```rust
   let (piece, next) = upload.room(room);
   self.http_down(env, http::Request::Upload(piece));
   if next == Down::Finish {
       self.upload = None;
       self.phase = Phase::Reading;
   }
   self.http_down(env, http::Request::Upload(next));
   ```

4. No new test: a refactor pinned by `a_json_upload_sends_its_body_and_content_headers` and the machine world.
5. Fmt, clippy, tests. Commit and push.

## Report

The report of your implementer text, and net production lines.
