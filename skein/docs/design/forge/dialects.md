# The forge kit: dialects and their call plans

## 1. Dialects and their call plans

### 1.1 What a dialect holds

A dialect renders the vocabulary on one forge's API. Forgejo's, at its
pin (1.3), holds:

- **Each request,** as a sized encoder: its bytes are counted before any
  is written, so a request's head and body are within the bounds the
  component gives.
- **Each answer,** as a decoder: a state machine over skein-json's
  tokens with a bounded stack, which keeps the few fields its answer
  needs and skips the rest as they pass (`forgejo.md`, 1.3).
- **The server's side of each document,** for the fake forge: the
  decoder of each request, and the encoder of each answer
  (../testing/fakes/forgejo.md, section 1).
- **The failures:** error bodies, the refusals whose message tells two
  failures of one status apart, and the map from a status to an `Error`
  (`forgejo.md`, 1.4).
- **The deliveries:** the headers, the events subscribed, each payload's
  decoder and encoder, the table to hints and the listener's path for a
  forge (`README.md`, section 6).
- **The setup's documents:** the admin's reads, the bot, its tokens,
  orgs, a repository's read by id, collaborator and hooks, and the
  repositories the admin can see (setup/forgejo.md, section 1).
- **Names, times and ids:** percent-encoding of a path's segments, RFC
  3339 times in both directions, and object ids in 40 or 64 hexadecimal
  digits.
- **How a credential renders on a request** (`README.md`, section 7).
- **The start's reads,** two call plans that the component runs when a
  forge starts and decides from, counted in no call's cost
  (connection.md, section 3):
  - **the identity read,** which says who the credential stands for:
    Forgejo's is `GET /api/v1/user`, for the user's `id`;
  - **the page cap read,** which gives the forge's page cap: Forgejo's
    is `GET /api/v1/settings/api`, for `max_response_items`, which must
    be above 0 (`forgejo.md`, 1.1).
- **The renewal,** for a renewed kind of credential: the plan that
  renews it, and how long before expiry it is due (`README.md`, section 7).
- **The pin** (1.3).
- **The call plans** (1.2).
- **Its limits,** `Limits`, which the component's owner gives:
  - `json` and `writer`: skein-json's tokenizer and sized writer;
  - `string_bytes`: the largest name, title, context, description, link
    or path kept;
  - `body_bytes`: the largest body kept whole;
  - `inner`: the most elements kept of a list inside one document;
  - `target_bytes`: the longest request target, its path and query.

  `check` refuses a zero, a `json.string` below `string_bytes` or
  `body_bytes`, and a depth below 8 (Forgejo's documents need about 8).
  The component's own limits (the connections, the bytes of a head or of
  a body read, the deadline) are connection-machines.md's, section 1.

A dialect holds no io, no clock and no connection.

GitHub's dialect, `skein_forge::github`, holds the same at its own pin,
with the App's documents (the JSON Web Token's bytes, the mint and the
installations' reads) where Forgejo's has the setup's admin reads
(github.md, section 2).

What holds for both dialects alike sits at the crate's root (1.4).

### 1.2 Call plans

A call plan is a dialect's small pure machine for one call. The
component runs any dialect's plans on its connections, and the only
dialect knowledge it keeps is in the plans it is handed.

- **Its steps.** `begin` takes the call and returns a step. `heard`
  takes what the plan heard and returns the next. A step is one of:
  - `Send`: an exchange, to be made on the call's connection. It holds
    the request and a range: the head or the tail bytes of a raw answer
    to keep (a job's log is kept from its end, a file from its start);
  - `Done`: the call's `Answer` or `Error`.
- **What a plan hears is the dialect's type.** Forgejo's plans hear,
  for an exchange:
  - `Head`, with the status and the few header fields they read
    (`Date`, `Retry-After`, `X-HasMore`, `Content-Range` and
    `Connection: close`);
  - `Row`, for each element of a listing that the exchange's decoder
    keeps;
  - `Done`, with the body: a document the decoder read, raw bytes (and
    whether more follow), an empty body, or a malformed one;
  - or `Failed`: the exchange failed with the server never having seen
    the request (`Unseen`), or possibly having seen it (`Seen`).
- **It is handed its bounds as plain values,** not the client's
  `Limits`: the page size (`forgejo.md`, 1.1), `rows`, `answer_bytes` and `body_bytes`.
  The component takes them from the client's limits and its own, and
  checks them at start (connection-machines.md, section 1).
- **It does no io and keeps no connection.** The component makes each
  exchange, and tells the plan what it heard.
- **It resolves a repository's path and a user's login** in two tables
  that the component keeps and hands it: it reads a row, and fills one
  from the answers it hears (connection-calls.md, section 1). A row of
  a repository holds its `owner` and `name` (its path), the object
  format, the merge style and the `html_url`. The tables are lib's `Lru`
  (../runtime/lib-containers.md, 1): bounded, and they evict the row used longest ago.
- **It ends with the call's result,** its `Done`, and counts nothing:
  the call's cost is the component's, one for each request whose first
  byte it wrote (connection-calls.md, section 1.1). A plan ends on the
  first failure, except where `forgejo-operations.md`, 1 says otherwise.
- **It maps a failure to an `Error`.** `classify` maps a status, a
  refusal's message and `Retry-After` to an `Error`, and says whether
  the path or the row is dropped (`forgejo.md`, 1.4).
- **It knows its dialect and its call,** and nothing of the budget or of
  the other calls in flight.

### 1.3 The pin

The dialect is pinned to one release of the forge, the one whose
behaviour the conformance checked.

- **Constants:** `VERSION`, Forgejo's release (17.0.1), and
  `RUNNER_VERSION`, forgejo-runner's (13.2.1).
- **Checks:** `names_forgejo` is true for the first line of
  `forgejo --version` when it names `VERSION`, ended by nothing, a `+`
  or a space; `names_runner` is true for the first line of
  `forgejo-runner --version` when it is `forgejo-runner version v`
  and `RUNNER_VERSION`, and nothing more.
- **Readers:** the throwaway Forgejo, the conformance, and a service's
  owned Forgejo check their binaries against the pin (../testing/fakes/throwaway.md,
  section 1).
- **Digests on Linux:** the SHA-256 of each release's Linux binary, per
  architecture, as lowercase hex:

  | Constant | Of | Bytes | SHA-256 |
  |---|---|---|---|
  | `FORGEJO_SHA256_LINUX_AMD64` | `forgejo-17.0.1-linux-amd64` | 122,057,320 | `51a8cdf4a1252e79f4b3855d7a7891af6c6635105420768c460e2f0690385d6f` |
  | `FORGEJO_SHA256_LINUX_ARM64` | `forgejo-17.0.1-linux-arm64` | 113,764,224 | `1e99403746406e8199efc903ddd1940d5728a60e50241f7aa6c9f2b907bd8ec9` |
  | `RUNNER_SHA256_LINUX_AMD64` | `forgejo-runner-13.2.1-linux-amd64` | 22,306,978 | `bb369fe1ecb50fdc983c0b1d993f19b0920df6e5864511892b3cdbf16d858b34` |
  | `RUNNER_SHA256_LINUX_ARM64` | `forgejo-runner-13.2.1-linux-arm64` | 20,840,610 | `eeddb7902f79abe488972f4189fbe3ea02762ee827d1b406b469420bf0d62dd9` |

  The sizes are not constants: they are what the digests were taken
  over, and what a fetch's bound must admit.
- **Digests on macOS:** neither project publishes a darwin build, so
  the pins are Homebrew's bottles for macOS 15 (`arm64_sequoia`), on
  arm64 alone, two digests for each binary: the archive's, which is its
  address, and the binary's inside it, which a file already in place is
  held to. A bottle is the same version as Linux's release and another
  build:

  | Constant | Of | Bytes | SHA-256 |
  |---|---|---|---|
  | `FORGEJO_BOTTLE_SHA256_MACOS_ARM64` | the bottle of `forgejo` 17.0.1 | 43,841,143 | `19db7c548064dfd4097264a7b4c4cf3b228bfd6190bc33a3c6e8a9b26c3d0e06` |
  | `FORGEJO_SHA256_MACOS_ARM64` | its `forgejo/17.0.1/bin/forgejo` | 111,660,034 | `4b6ed0e7e6ef034d298872125f69332e2aad49367435f66c58ee5eb2c5680cf6` |
  | `RUNNER_BOTTLE_SHA256_MACOS_ARM64` | the bottle of `forgejo-runner` 13.2.1 | 7,960,512 | `0a8cb32f024121e50ebca1840ed2f5650d65d3eb3e183679c0f8f0bc14d6fb48` |
  | `RUNNER_SHA256_MACOS_ARM64` | its `forgejo-runner/13.2.1/bin/forgejo-runner` | 21,661,778 | `2829f0125068f0ad92f908cb08c4d3c2f458cc22eec15fd675c332823cbece4b` |

  Each binary's load commands name macOS's own libraries alone
  (`libSystem`, `libresolv`, `CoreFoundation`, `Security`).
- **Sources:** each binary's release file, named by the constants and
  the architecture:
  - Forgejo's on codeberg.org,
    `https://codeberg.org/forgejo/forgejo/releases/download/v17.0.1/forgejo-17.0.1-linux-{amd64,arm64}`;
  - forgejo-runner's on code.forgejo.org,
    `https://code.forgejo.org/forgejo/runner/releases/download/v13.2.1/forgejo-runner-13.2.1-linux-{amd64,arm64}`.
    Codeberg serves no runner release (404).

  Each answers a single `200` with the file, and no redirect. A Linux
  fetcher follows none, and takes any other status as the source
  failing.
- **The bottles' sources,** fixed by the archive's digest, never looked
  up: `https://ghcr.io/v2/homebrew/core/forgejo/blobs/sha256:DIGEST` and
  `https://ghcr.io/v2/homebrew/core/forgejo-runner/blobs/sha256:DIGEST`.
  Asked with `Authorization: Bearer QQ==`, the anonymous value Homebrew
  sends, each answers a `307` to its storage's `https` address, which
  answers `200` with the archive and is sent no token.
- **TLS:** with the machine's roots. The digest, not the certificate, is
  what makes the file the pinned one.
- **An architecture without a digest** is refused: a fetcher names the
  architecture and fetches nothing. The pins are Linux on amd64 and
  arm64, and macOS on arm64.
- **Readers of the digests:** a service that fetches the pinned binaries
  (temper's `forge fetch`) checks each file it streams, and each file it
  finds already in place, against the digest of its own platform and
  architecture. skein-forge does no io for this: it holds the constants
  and a `digest(binary, platform, arch, form)` that answers the pinned
  digest, or none where there is no such pin: `Platform` is `Linux` or
  `Macos`, `Form` is `Binary` or `Archive`, and Linux has no `Archive`.
- **Moving the pin** moves the digests with it: a new `VERSION` or
  `RUNNER_VERSION` comes with the new files' digests, all of them, taken
  as above, in the same change, and only to versions with bottles.

### 1.4 What the dialects share

What does not depend on a forge's documents sits at `skein_forge`'s
root, and both dialects stand on it:

- `time`: RFC 3339 and the HTTP date, on lib's `Civil` (../runtime/lib.md, 8);
- `path`: percent-encoding and a target's reading;
- `text`: `Body`, a text kept whole or only counted;
- `decode`: `Malformed`;
- `limits`: `Limits`, `Problem` and `check`, which takes the depth a
  dialect needs (8 for Forgejo, 10 for GitHub);
- `plan`: an exchange's `Failure` and the `Error` each reads as (`forgejo.md`, 1.4);
- `users`: the logins' table (1.2).

A dialect keeps its public name for each by re-export, and names no item
of the other dialect. Its documents, ids, plans, `Bounds` and repository
rows stay its own. The dialects stay modules of one crate (`README.md`, section 10).
