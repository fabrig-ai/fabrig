# Finding `temper-box`

How `temper box` finds the `temper-box` it runs: one of its own build,
beside it, in its cache, or downloaded once from the site, and checked
(`../draft/cli.md`, 5.1 and 5.2). What it does with the box once found
is `tui/box.md`'s, in a terminal, and section 5's without one; the box's
side is `../protocol/setup/box.md`, 6.

## 1. In one page

- **Its identity:** a `temper-box` runs only when its `--version` line
  is the one `temper`'s own build would give it (2).
- **Where, in order:** beside `temper`'s own executable; in the cache, at
  `bin/<COMMIT>/<os>-<arch>/temper-box`, checked by the SHA-256 beside
  it; downloaded from `https://fabrig.ai`, checked against the site's
  `SHA256SUMS`, when `temper`'s commit is clean. A `-dirty` or `unknown`
  build never downloads. No PATH (2).
- **The download** is the forge's fetch's way (`../protocol/setup/owned.md`,
  6): streamed into private staging files under a lock, bounded,
  checked, made executable, synced and renamed into place (3).
- **`temper internal fetch-box`** runs it in a child of `temper box`'s
  TUI, saying how far it has gone on stdout and ending with `found PATH`
  (4).
- **Without a terminal,** `temper box` finds it in its own process and
  runs it in its own place (5).
- **Its code:** a machine in `temper-fetch` and its host in
  `temper-fetch-shell`, beside the forge's fetch (6).

## 2. Finding it

- **The identity wanted** is `temper`'s own: `temper_version::line("temper-box")`,
  `temper-box VERSION (COMMIT)` with its line feed, VERSION and COMMIT
  `temper`'s (`../release.md`, section 3). A candidate's is read by
  running it, `CANDIDATE --version`, with an empty environment, the null
  device on its stdin and stderr, and its stdout read to its end, at most
  256 bytes, within 10 seconds. It is that build's when its stdout is the
  wanted line, byte for byte, and its exit 0; anything else (another line,
  more bytes, a failure to start, a signal, the deadline) is another
  identity. A candidate that isn't a regular file executable by the user
  is none.
- **1. Beside `temper`'s own executable:** `DIR/temper-box`, DIR the
  directory of the path the capture read (`../cli/README.md`, 7.4), as a
  checkout's `target/` and a deployment's directory have it. Of its
  identity, it is taken; of another, passed over, and left as it is.
- **2. In the cache,** when COMMIT is clean, twelve lowercase hexadecimal
  digits: the directory `CACHE/bin/<COMMIT>/<os>-<arch>/`, CACHE
  resolved as the three directories are (`--cache` among `temper box`'s
  arguments, else `TEMPER_CACHE`, else `$XDG_CACHE_HOME/temper`, else
  `~/.cache/temper`; `temper_shell::directories`). `bin/` stands apart
  from the forge's and Claude Code's caches beside it.
  - Its files: `temper-box`, `0o755`; `temper-box.sha256`, the unpacked
    file's SHA-256 as 64 lowercase hexadecimal digits and a line feed;
    `temper-box.lock`; and while a fetch runs, `temper-box.gz.part` and
    `temper-box.part`. Each directory from `bin/` down is made `0o700`
    when missing.
  - **Under the lock:** the lock file, `0o600`, is opened and locked
    exclusively, waiting while another `temper` holds it, before the
    cache is read, and held until the file is published or refused, so
    that of two `temper box` at once one fetches and the other then finds
    it.
  - **Reused** when `temper-box`'s SHA-256 equals the one in
    `temper-box.sha256` and its identity is the wanted one. A file whose
    sum is missing or differs is removed, with its sum, and fetched
    again, as the forge's fetch treats a wrong file. A file whose sum
    matches and whose identity doesn't is the site's mistake: it is
    removed, with its sum, and refused, naming the line it printed.
- **3. Downloaded** (3), when COMMIT is clean and neither found one.
- **A `-dirty` or `unknown` COMMIT** reads no cache and never downloads:
  with none beside it, it is refused, saying which binary is missing and
  how to build it: `temper: no temper-box of this build (VERSION
  (COMMIT)) beside DIR/temper; a build from a checkout downloads none:
  build it with cargo build -p temper-box`.
- **No PATH:** a `temper-box` on PATH may be another commit's, so
  `temper` doesn't look there. Run as `temper-box`, it's whatever the
  shell finds.
- **`<os>-<arch>`** is `uname -s` and `uname -m` as the site names them
  (`../draft/deployment-binaries.md`, section 2): `Linux-x86_64`,
  `Linux-aarch64` or `Darwin-arm64`, the platform `temper` was built for,
  given to the machine by its host. Another platform finds one beside it
  or in the cache alone, and its download is refused, naming it.

## 3. The download

- **What's trusted:** fabrig.ai over HTTPS, with the machine's roots or
  `SSL_CERT_FILE`, read as the forge's fetch reads them (skein's
  `runtime/shell.md`, 6.2; `../protocol/setup/owned.md`, 6). The sums
  come from the same site, so they catch a damaged file, not a site that
  lies; writing the checksum into `temper` at release is Later
  (`../draft/cli.md`, section 10).
- **The site:** `https://fabrig.ai`, or the origin `TEMPER_INTERNAL_SITE`
  names, internal, for tests' fake site: `https://HOST:PORT` with HOST
  `127.0.0.1`, `[::1]` or `localhost`, and nothing after the port. A
  value naming another host, another scheme or a path refuses `temper
  box`, naming the variable. Its TLS is the same: a fake site's
  certificate is trusted by an `SSL_CERT_FILE` the test names.
- **What's fetched,** in this order, each `GET`, one request and one
  answer, `200` with the body:
  - `SITE/temper/v{MAJOR}/<COMMIT>/SHA256SUMS`, MAJOR the first number of
    VERSION (`v0` before 1.0), at most 64 KiB;
  - `SITE/temper/v{MAJOR}/<COMMIT>/<os>-<arch>/temper-box`, gzipped with
    no suffix (`../draft/deployment-binaries.md`, section 2), at most 128
    MiB as served and 512 MiB unpacked.
  Any other status refuses, naming the URL and the status. A redirect
  is followed only to the same origin, `https`, at most three; one to
  another host, port or scheme refuses, naming both URLs.
- **The sums,** as `sha256sum` writes them: each line 64 lowercase
  hexadecimal digits, then two spaces or a space and `*`, then a name
  running to the line feed. Exactly one line's name is
  `<os>-<arch>/temper-box`, and its digest is the served, gzipped
  file's. None, two, or that line malformed refuses, saying which; other
  lines are not read past their name. A body past 64 KiB refuses.
- **Staged, then published,** under the lock (2):
  1. Staging files a fetch left are removed first.
  2. The served body is streamed into `temper-box.gz.part`, private
     (`0o600`), hashed as it is written, within 128 MiB: past it, or
     past the answer's `Content-Length`, refuses.
  3. Its digest is checked against the sums' line: another refuses,
     naming the URL, the digest wanted and the one got.
  4. Only then is it unpacked into `temper-box.part`, private, hashed as
     written, within 512 MiB: past it, or a gzip stream that is
     malformed, truncated or followed by more than padding, refuses.
  5. `temper-box.part` is made `0o755` and synced; its identity is
     read (2), and another refuses, naming the line it printed.
  6. `temper-box.sha256` is written through `temper-box.sha256.part`,
     synced and renamed into place; then `temper-box.part` is renamed
     over `temper-box`, the directory synced, and `temper-box.gz.part`
     removed.
  A refusal removes every staging file; an interrupted fetch leaves only
  staging files, which the next removes (step 1). A `temper-box.sha256`
  without its file is a cache with no file, fetched again.
- **How far it has gone:** `fetched temper-box DONE of TOTAL bytes`, or
  with no size `fetched temper-box DONE bytes`, the forge's fetch's line
  (`../protocol/setup/owned.md`, 6; `temper_fetch::fetch::Fetched`),
  DONE the served bytes written and TOTAL the served file's
  `Content-Length`: one at its start, one at each whole percent it
  passes, at most one for a chunk, and one at its end; with no size, one
  at each MiB. A file found beside it or in the cache says nothing. A
  line the output will not take is dropped.
- **A failure** of the download, offline among them, refuses with its
  words, naming the URL and why, then a line for fetching it by hand,
  beside `temper`, where step 1 of 2 finds it:

  ```
  temper: temper-box: https://fabrig.ai/temper/v0/0123456789ab/Linux-x86_64/temper-box: cannot connect: ...
  Fetch it by hand: curl -fL https://fabrig.ai/temper/v0/0123456789ab/Linux-x86_64/temper-box | gunzip > '/home/ada/bin/temper-box' && chmod +x '/home/ada/bin/temper-box'
  ```

  The path is quoted for a POSIX shell, a `'` in it written `'\''`.

## 4. `temper internal fetch-box`

`temper internal fetch-box [--cache DIR]`, internal, spawned by `temper
box` in a terminal (`tui/box.md`), its own executable run again, so the
TUI draws while the fetch runs, and nothing of the fetch shares the
TUI's loop.

- **Its arguments:** `--cache DIR`, the cache `temper box` resolved from
  its own arguments and environment, given whole; without it, the cache
  resolves from its environment as 2 says. Its environment is
  `temper box`'s captured one, `SSL_CERT_FILE` and `TEMPER_INTERNAL_SITE`
  read from it.
- **Its stdout:** 3's progress lines, then, once found, `found PATH`,
  PATH the absolute path of the `temper-box` to run, its bytes to the
  line feed, and exit 0.
- **Its stderr:** a refusal's or a failure's words, 2's or 3's, the
  hand-fetch line among them, at most 8 KiB, and exit 1, with no `found`
  line.
- **Its stdin** is the null device. It reads no terminal and writes no
  other file than 2's.

## 5. Without a terminal

- **`temper box [ARGUMENT...]`,** its standard input or output not a
  terminal, or `--echo steps` among its arguments, finds `temper-box` as
  2 says in its own process, writing 3's lines and, when it fetches
  first, `fetching temper-box`, on its stderr, and then runs it in its
  own place (exec), `temper-box ARGUMENT...`, the arguments unchanged,
  with its environment and its descriptors: for scripts and units, and
  for the box's own echo on its terminal (`../protocol/setup/box.md`,
  2). A refusal or a failure prints its words on stderr, exit 1.
- **A `temper-box` refused its arguments** after the exec answers as
  `temper-box` does, exit 2: `temper` doesn't read the box's grammar.

## 6. Its code

- **The machine,** `temper_fetch::box_binary`, no_std: the order of 2,
  the identity line, the clean commit, the cache's and the site's paths,
  the sums' reading and the bounds, as its inputs and outputs, in the
  shape of `temper_fetch::fetch` (requests answered by events, `step`
  emitting exactly one next request), reusing that module's pieces
  where they fit (`Fetched`'s lines, `url`, `hex`). Its terminal answers
  are the path found and a refusal, whose words are 2's and 3's.

  ```rust
  // temper_fetch::box_binary
  /// `temper-box VERSION (COMMIT)\n`, the line a temper-box of this build prints.
  pub fn identity(version: &str, commit: &str) -> Box<[u8]>;
  /// Twelve lowercase hexadecimal digits: a commit that may download.
  pub fn clean(commit: &[u8]) -> bool;
  /// The sums' digest for NAME (`Linux-x86_64/temper-box`): exactly one line names it.
  pub fn sum(sums: &[u8], name: &[u8]) -> Result<[u8; 32], Sums>;
  pub enum Sums { Missing, Doubled, Malformed, TooLong }
  ```

- **The host,** `temper_fetch_shell::box_binary`, on skein-io, skein-http
  and skein-tls as the forge's fetch's host, with std's files, the lock
  as `claude_code`'s, and `flate2`'s gzip decoder, the workspace's, for
  step 4 of 3:

  ```rust
  // temper_fetch_shell::box_binary
  pub struct Find<'a> {
      /// The directory of temper's own executable.
      pub beside: &'a std::path::Path,
      /// The cache directory, resolved (2).
      pub cache: &'a std::path::Path,
      /// `https://fabrig.ai`, or TEMPER_INTERNAL_SITE's origin.
      pub site: &'a temper_fetch::Origin,
      pub trust: &'a skein_shell::Trust,
      pub version: &'a str,
      pub commit: &'a str,
  }
  /// The temper-box to run, its lines on `out` (3); Err's words are 2's and 3's.
  pub fn find(limits: &skein_io::Limits, find: &Find<'_>, out: &mut dyn std::io::Write)
      -> Result<std::path::PathBuf, Failed>;
  ```

  The platform is the host's, as its shell was built for. `Failed`'s
  words carry the hand-fetch line where the download failed.
- **The client's crate,** `temper-client-shell`, reads the cache, the
  site and the trust from its capture, runs `find` for `internal
  fetch-box` and for 5, and links `temper-fetch` and `temper-fetch-shell`
  (`../protocol/binaries.md`, 4). It holds the `TEMPER_INTERNAL_SITE`
  variable (`../cli/README.md`, 7.4).

## 7. Tests

- **The machine's choices, in step tests,** with no process or network
  (`../draft/cli.md`, section 9), each a row driving the machine with
  its events: one beside it of its identity, taken; one beside it of
  another, passed over; a cached one whose sum matches, taken, and one
  whose sum differs or is missing, removed and fetched; a cached one
  whose identity differs, refused; a download; a dirty and an unknown
  COMMIT, refused, with no cache read and no request; the sums with one
  line, none, two and a malformed one; each bound, passed by one byte;
  a redirect to the same origin and one elsewhere; and the progress
  lines' rule.
- **The host,** in its unit tests on a scratch directory: the lock taken
  before the cache is read, staging files removed, the publication's
  order, and a refusal leaving no staging file. The fetch over TLS is
  the forge's fetch's host's, tested there.
- **The process tier,** written and compiled, run after landing: a fake
  site, a loopback HTTPS server holding a gzipped `temper-box` and its
  sums under `TEMPER_INTERNAL_SITE` and an `SSL_CERT_FILE` naming its
  certificate, through `temper box` without a terminal: the download,
  its staging, lock and publication, an interrupted fetch, and the exec.
  A build whose COMMIT is dirty asserts the refusal instead, saying which
  it held on its stderr.

## 8. Open questions

None now; `../draft/cli.status.md` keeps what's open.
