# The forge kit

Provisional, 2026-10-20. What skein gives a service that works on a code
forge, on Forgejo and on GitHub (github.md; section 8). The kit
is:

- a neutral vocabulary of forge calls, rendered by a dialect;
- a component that runs those calls and receives the forge's webhooks;
- a client domain that keeps a bounded view of a forge and writes to it
  at most once;
- a setup machine that makes a forge ready for a service;
- the fake forge and the conformance tooling that a service's worlds and
  tests need.

This document is the kit as a whole and its first crate, `skein-forge`:
the vocabulary, the dialects, keys' markers, deliveries and credentials.
Its [vocabulary](vocabulary.md) and [dialects](dialects.md) have their
own pages. The dialects have [Forgejo's API](forgejo.md) and
[operations](forgejo-operations.md), and [GitHub's dialect](github.md),
[operations and merge rules](github-operations.md), and
[App and setup documents](github-app.md).
The others are [connection.md](connection.md) (the component), with
[calls](connection-calls.md), [webhooks](connection-webhooks.md), and
[entities, limits and machines](connection-machines.md);
[client.md](client.md) (the client), with its [write effects](client-effects.md);
setup/README.md (the setup), job-state.md (a job's state, read
when a wait on CI passes its bound) and ../testing/fakes/forge.md (the
fakes, the throwaway Forgejo and the conformance).

## 1. In one page

- **What a connector stands on, not the connector.** The kit is the
  layers a forge connector is built on: the documents, the call plans,
  the component, the client's keep and write lanes, the setup's machines
  and the fakes. It holds nothing a connector decides: what to watch,
  which effects exist and what each is for, when to write, and what
  people see (section 2.3). The kit names no service.
- **Shaped like skein's LLM crates.** `skein-forge` is to forges what
  `skein-llm` is to providers: a neutral vocabulary rendered by
  dialects. Each call is answered once, and nothing is retried.
  `skein-forge-connection` is to it what `skein-llm-connection` is to
  `skein-llm`: a component that an owner's protocol layer owns, and that
  connects, runs calls and keeps connections (../llm/README.md;
  ../llm/connection.md).
- **One call, one answer, and its cost.** Each `Read` or `Write` is
  answered by exactly one `Answer` or `Error`, made of one or a few HTTP
  exchanges. The answer carries the count of requests the call cost.
  Budgeting the forge's rate is the client's job (client.md,
  section 6).
- **Repositories by the forge's id.** A repository is named by the
  forge's number and the forge's own id, which survives a rename or a
  transfer. A repository is first found by its owner and name, with a
  locate.
- **The owner names its marks.** A creation's key travels in a marker
  whose word the owner gives (`<!-- <word>:key … -->`; temper's word is
  `temper`). The user agent is the owner's, and setup marks its bot with
  the same word.
- **Dialects hold the documents and the plans.** Forgejo's dialect holds
  both sides of each document, its deliveries, the setup's documents and
  its pin. Each call is run by a call plan of the dialect: a small pure
  machine that does no io (`dialects.md`, section 1).
- **Credentials have a kind, set by the dialect.** A token lasts until
  revoked (Forgejo's, or a GitHub fine-grained token). A GitHub App
  installation's token is minted from the App's key and lasts an hour.
  The component holds the current credential, renews it when the dialect
  says, and gives it to no one (section 7).
- **Webhooks for latency, polling for liveness.** Deliveries are
  verified as they stream and become hints. The client's keep polls
  whether or not they arrive.
- **The client keeps and writes, and decides nothing.** Its working set,
  request budget and write lanes are bounded. Each write carries a key
  that its owner made, so a write whose answer was lost is found after a
  restart, not made twice. What an effect is for is opaque bytes of the
  owner's.
- **Setup brings a forge to one kind of result: an access.** On any
  forge that result is the same:
  - the user it acts as;
  - its API and git credentials;
  - the repositories it may write;
  - a webhook secret and its hooks.

  Each forge has its own plan to reach it. Forgejo's is reconcile, with
  a site admin's token. GitHub's is the App manifest flow
  (setup/README.md). The kit supplies only the machines; the host is the service's.
- **The fake forge is shared, with a layer per dialect.** Its domain is
  one; Forgejo's protocol layer and GitHub's follow their dialects, and
  skein-fake-peers hosts them. Forgejo's conformance runs against a
  throwaway Forgejo and records the facts that section 9 lists;
  GitHub's runs against the fake, and opt-in against github.com
  (../testing/fakes/github.md, sections 1 and 2).
- **GitHub is a second dialect,** `skein_forge::github`, beside
  `forgejo` (github.md). It adds to the vocabulary validators, the
  rate's window, `Queued`, the protection's queue and bypass,
  `NotConnected`, `NotInstalled` and the App's credential (section 8).
  Forgejo's dialect ignores each addition or answers its neutral value.

## 2. The kit's crates

### 2.1 The crates

```
skein-forge                     the vocabulary (api, locate); what the dialects share (4.4); the
│                                 dialects: Forgejo's documents, both
│                                 sides, its call plans, its deliveries, the setup's documents, the
│                                 pin; GitHub's likewise (forge-github.md); keys' markers;
│                                 credentials' kinds
├── skein-lib, skein-json
skein-forge-client              the keep (working set), the request budget, the write lanes and their
│                                 recovery, keys
├── skein-lib, skein-forge (its vocabulary only)
skein-forge-connection          the component: calls, connections, each forge's start, repositories'
│                                 paths, users, credentials and their renewal; the webhooks' listener
├── skein-lib, skein-io, skein-http, skein-tls, skein-json, skein-forge
skein-forge-setup               the access and each forge's plan: Forgejo's reconcile machine;
│                                 GitHub's manifest flow
├── skein-lib, skein-io, skein-http, skein-json, skein-forge
crates/skein-fake-forge-domain     the fake forge's domain
crates/skein-fake-forge-http       what its two layers share: a connection's core and the git route
crates/skein-fake-forge-forgejo   Forgejo's API over it on skein's HTTP server; the setup's routes;
                                     signed deliveries; the tally of requests served
crates/skein-fake-forge-github    GitHub's API over it, at its pin (../testing/fakes/github.md, section 1)
crates/skein-fake-peers            hosts the forge peer, beside the LLM and OAuth peers
testing/skein-throwaway-forgejo    a real Forgejo and its runner, at the pin, for the conformance
tests/forge/machine, protocol,     the kit's worlds, one crate each (section 11)
  client, setup
tests/forge/conformance            a tool outside the suites: records the dialect's facts
tests/forge/github                 GitHub's conformance: on the fake, and opt-in live (../testing/fakes/github.md,
                                     section 2)
```

- **`skein-forge` is `no_std` (with `alloc`) and lib-only,** as the
  client is, whose vocabulary it carries. It has no io and no clock.
- **The client and the component do not depend on each other.** Both
  stand on the vocabulary. The owner carries calls to the component, and
  answers and hints back to the client (client.md, section 2;
  connection.md, section 2).
- **Each crate has its documents:** this page, [vocabulary.md](vocabulary.md)
  and [dialects.md](dialects.md) for `skein-forge`, with
  [forgejo.md](forgejo.md) and [forgejo-operations.md](forgejo-operations.md),
  and [github.md](github.md), [github-operations.md](github-operations.md)
  and [github-app.md](github-app.md) for the two dialects;
  [connection.md](connection.md), [connection-calls.md](connection-calls.md),
  [connection-webhooks.md](connection-webhooks.md) and
  [connection-machines.md](connection-machines.md) for the component;
  [client.md](client.md) and [client-effects.md](client-effects.md) for the
  client; setup/README.md for the setup; and ../testing/fakes/forge.md for
  the fakes, the forge peer, the throwaway and the conformance.

### 2.2 What the owner gives

The owner is the service code that owns a part: its protocol layer owns
the component, its forge connector (a domain) owns the client, and its
setup shell hosts reconcile. The kit reads no file and no configuration
(connection.md, section 3). The owner hands it values:

- **The word.** The marker's and the bot's (section 5).
- **The user agent,** on every request (`forgejo.md`, section 1.1).
- **The namespace and the identity.** The namespace is the owner's keys'
  namespace. The identity is the owner's bytes for an effect, opaque to
  the kit (client-effects.md, section 1).
- **For each forge:** its number, its dialect, its API's address and
  transport, the value of its credential, the forge user that credential
  must stand for, and its webhook secret when it has one.
- **The listener's address,** when webhooks are used.
- **The limits** of each crate it owns.

### 2.3 What stays with the service

- **The connector's top.** It is what the client's owner builds on the
  client:
  - what it watches;
  - which effects exist, and what each is for;
  - policy: gates, merges and landings;
  - adoption's checks;
  - what people see.

  It is not in skein: it is the service's.
- **The protocol layer's translation,** between its own boundary and the
  kit's calls, answers and hints; and the store's records of the
  client's rows (the `Stored` rows that the client hands its owner to
  keep).
- **The configuration and its files.** The kit reads none.
- **The setup's host:** its command, its files, the forge's lock, its
  exit status. Owned Forgejo's preparation (`app.ini`, migrations, the
  runner's registration, upgrades' dumps) is the service's.
- **The service's git component,** and that component's fakes.
- **The service's worlds' stories,** and its cost ceilings. They run on
  the kit's fakes.

## 3. The vocabulary

Moved to [`vocabulary.md`](vocabulary.md), section 1.

## 4. Dialects and their call plans

Moved to [`dialects.md`](dialects.md), section 1.

## 5. Keys' markers

What the kit writes inside what it creates: keys' markers, and nothing
else. No record, outcome or page of the owner's is written into the
forge. Labels are written only by the calls that name them
(`CreateIssue`'s, `Labels`, `CreateLabel`); the
kit decides nothing by one, and whether its owner reads them back is
the owner's.

- **A creation's key,** at the head of an issue's body, a comment or a
  review: `<!-- `, the word, `:key `, the key in lowercase hexadecimal,
  then ` -->` and a line feed, then the text. It is an HTML comment,
  which Forgejo's rendering hides. A pull request has no marker: its key
  is its branches.
- **The word is the owner's.** It is 1 to 16 bytes of lowercase ASCII
  letters, digits and `-`, checked at start. The same word marks the
  setup's bot, `<word>:bot` (setup/forgejo.md, section 1).
- **Services that share a forge** each have their own word, their own
  bot and their own keys' namespace. A body whose marker carries
  another word has no key.
- **The key is the owner's bytes.** The client builds it with
  `effect_key(namespace, identity, max)` (client-effects.md, section 1):
  - version 1, one byte;
  - the namespace's length, as two bytes big-endian;
  - the namespace, which is the owner's;
  - the identity, which is the owner's opaque bytes for the effect.

  The key's first byte is its version; the marker has none of its own.
  Owners that share a repository tell their creations apart by
  namespace.
- **Decoded** from a body whose first bytes are `<!-- `, the word and
  `:key `, then an even count of lowercase hexadecimal digits, then
  ` -->` and a line feed; any other body has no key.
- **The marker code only encodes and decodes,** and never reads the key.
  Whether to believe a key is the client's call: its version, its
  namespace, and whether its author is a configured writer
  (client-effects.md, section 1). A person who copies a marker into their
  own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`. The marker is
  twice the key's length plus the word's length plus 15 bytes: with a
  6-byte word, twice the key plus 21. The body sent is the marker and
  the text, which `body_bytes` holds (`forgejo.md`, 1.3).
- **One code serves every use:** the writes' encoders, and the decoders
  of items, comments, reviews and deliveries.

## 6. Deliveries

Forgejo's deliveries, made hints: they cut polling's latency, and
decide nothing. Polling stands with no webhook (client.md,
section 4), and finds whatever a delivery lost, refused or never sent
told of. The listener, its connections and its caps are
connection-webhooks.md, section 1. This section is what the dialect holds.

- **The listener's path.** A forge's deliveries arrive at
  `/hooks/<number>`, the forge's number in decimal with no sign and no
  leading zero, within 16 bits. A path of another shape names no forge.
- **Signed, and verified as it streams.** Forgejo signs each delivery's
  body with its hook's secret: HMAC-SHA256, in lowercase hexadecimal, in
  `X-Forgejo-Signature`. The component computes it as the body arrives,
  with skein lib's `sha256::Hmac` keyed by the forge's secret, and
  decodes the body at the same time into the few fields its hint reads;
  nothing of it is believed until the whole body is read and the
  signature holds, compared with `sha256::equal`, in constant time
  (../runtime/lib-bytes.md, section 1.2).
- **Its verdicts:**

  | A delivery | Answer | Hint |
  |---|---|---|
  | with no signature | `403` from its head, its body unread | none |
  | with a wrong signature | `403` once its body is read | none |
  | whose body length passes its cap | `413` | none |
  | verified, and its body malformed | `400` | none |
  | verified, and an event of no row below (a tag's included) | `204` | none |
  | verified, and mapped | `204`, queued first | one |

  An unknown path, a forge with no secret and a method other than `POST`
  are the listener's (connection-webhooks.md, section 1).
- **Answered first.** Once the body is read and verified, the response
  (`204`) is queued, and only then does the hint go up. The component
  never makes Forgejo wait on its owner's work. skein's HTTP server lets
  the component answer when it chooses (http.md, section 5.4).
- **The events a hook subscribes to** are every event whose delivery the
  table maps to a hint, and no other: 22 in all. The component reads
  each delivery by `X-Forgejo-Event`, the header it arrives under, which
  several events share; each hint's repository is the forge's number and
  the payload's `repository.id` (`run.repository.id` for a job's), so no
  path is needed (`forgejo.md`, 1.1). A repository id past 32 bits makes no hint.

  | Events subscribed | `X-Forgejo-Event` | `Hint` |
  |---|---|---|
  | `issues`, `issue_assign`, `issue_label`, `issue_milestone` | `issues` | `Change::Item(number)`, with the key of the issue's body |
  | `pull_request`, `pull_request_assign`, `pull_request_label`, `pull_request_milestone`, `pull_request_sync`, `pull_request_review_request` | `pull_request` | `Change::Item(number)`, with no key: a pull request's key is its branches (section 5) |
  | `issue_comment`, `pull_request_comment` | `issue_comment` | `Change::Item(number)`, with the key of the comment's body |
  | `pull_request_review_approved`, `pull_request_review_rejected`, `pull_request_review_comment` | `pull_request_approved`, `pull_request_rejected`, `pull_request_comment` | `Change::Item(number)`, with the key of the review's body |
  | `push` | `push` | of a branch, `Change::BranchMoved { branch, head, actor }`, the commit it moved to and the pusher's id; of a branch deleted (its `after` all zeros), `Change::Branch(branch)` |
  | `create`, `delete` | `create`, `delete` | of a branch, `Change::Branch(branch)` |
  | a job's ends: `workflow_job_success`, `workflow_job_failure`, `workflow_job_cancelled`, `workflow_job_skipped` | the same | `Change::Commit(commit)`, the run's `commit_sha` |
  | none | anything else; a tag's `push`, `create` or `delete` | nothing |

  A body not kept whole (`forgejo.md`, 1.3) gives no key. The decoder yields a
  `Delivery`: `Issue`, `Pull`, `Comment`, `Review`, `Push`, `Ref` or
  `Job`, a job's end being `Success`, `Failure`, `Cancelled` or
  `Skipped`. Each row's events, its header and the fields its hint reads
  are recorded on the pinned release (`forgejo.md`, section 2). A row that went wrong
  would cost a poll's wait, never a decision: a hint only brings a read
  sooner (client.md, section 4).

  **Who caused it** is what lets the owner drop its own writes' echoes,
  by key (a creation's, for an item) and by pusher (for a push); the
  client reads on every hint it holds a resource for, and uses the
  pusher for drift alone (client.md, section 4). A job's delivery
  has no top-level repository or sender: its `run.repository.id` and
  `run.commit_sha` stand for them, and its hint names the commit and no
  actor. A hint for a repository the owner has not adopted goes up all
  the same; the owner drops it.
- **CI is hinted as each job ends.** A job's delivery is sent for each
  of its attempts, so a job run again is hinted again when it ends. The
  run's deliveries (`workflow_run_*`), sent once its last job ends, and
  the deprecated `action_run_*` are not subscribed.
- **What has no delivery** is found by polling alone (client.md,
  section 4): a commit status, which Forgejo never delivers, so CI that
  a system other than Actions reports and the verdicts the owner posts
  as statuses.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: `forgejo.md`, section 2) is read again by the
  owner's fresh reads, within its `settle` of the move; the client runs
  no window for it (client.md, sections 4 and 5). The component
  hints at once.
- **Bounded.** Push payloads list commits, which the decoder skips. The
  listener's connections, head, body and deadline are capped
  (connection-webhooks.md, section 1, and connection-machines.md, section 1).
- **Not sent again.** Forgejo does not resend a failed delivery: one
  lost while the owner is down, refused for its size or its signature,
  or never sent is found by polling. A hint that arrives before the
  owner's domain is ready is dropped, and the keep's first listing finds
  what it told of.
- **Hooks are set up by reconcile, or by the person,** at the listener's
  URL for the forge, with the forge's secret, the events above, JSON and
  active (setup/forgejo.md, section 1.5). A repository with no hook polls.

## 7. Credentials

- **A credential has a kind, and each dialect names the kinds it
  takes:**
  - **a token,** which lasts until revoked: Forgejo's, or GitHub's
    fine-grained token;
  - **an App** (GitHub's: `Credential::App { id, key }`): the App's id,
    and its private key, an RSA key of 2048 bits as PKCS#1 DER. The
    component signs a JSON Web Token with it (RS256, through `ring` as
    `skein_tls::sign_rs256` has it), mints from that each installation's
    token, which expires within the hour, and chooses the installation
    by a repository's owner (connection.md, section 5). The
    dialect gives the token's bytes and the mint (github-app.md,
    section 1); GitHub's tokens render as `Authorization: Bearer <t>`.
- **The owner gives the value** at start, for each forge (section 2.2).
  The component reads no file.
- **The dialect says how a credential renders on a request.** Forgejo's
  is `Authorization: token <t>` (`forgejo.md`, 1.1). The request's head, that field
  among them, fits skein-http's `request` limit: the component bounds
  the credential's size at start (connection-machines.md, section 1).
- **For a renewed kind,** the dialect also gives the plan that renews
  the credential, and how long before expiry it is due. The component
  runs it (connection.md, section 5).
- **A credential's value reaches no record, request line, fact or
  hint.** It is not logged, granted or stored.
- **A credential that is expired or revoked** reads as `Forbidden` on
  every call (section 10). An App's key that GitHub refuses (a `401` on
  the JSON Web Token) is the same.

## 8. GitHub

GitHub's dialect is `skein_forge::github`, a module of `skein-forge`
beside `forgejo`, at the pin `X-GitHub-Api-Version: 2026-03-10`.
github.md holds its documents, plans, failures, deliveries, the
App's documents and the facts it relies on; ../testing/fakes/github.md, sections 1
and 2, its fake layer and its conformance. Here is what it adds to the
kit.

- **A provider is documents plus call plans.** Nothing above the
  Forgejo dialect and its plans assumes Forgejo. GitHub's calls are REST
  at the pin and three GraphQL documents (the viewer, the queue's
  enqueue, a pull's merge commit), each call's cost counted by the
  component as Forgejo's is (`dialects.md`, 1.2).
- **The vocabulary gains:**
  - validators on reads and their not-modified answer, free of the rate
    (`vocabulary.md`, 1.3);
  - the rate's window, `Rate`, which the budget takes in place of its
    configured one (`vocabulary.md`, 1.3);
  - pauses, as `RateLimited { after }` (`vocabulary.md`, 1.4);
  - `Queued`, a merge put in a merge queue (`vocabulary.md`, 1.2, 1.3), and `Pull`'s
    `queued`;
  - the protection's queue, signatures, linear history, code owners and
    `Bypass` (`vocabulary.md`, 1.6);
  - `NotConnected` and `NotInstalled`, the component's (`vocabulary.md`, 1.4);
  - the App's credential, `Credential::App` (section 7).

  Each addition is optional for a dialect: Forgejo's carries no rate and
  no validator, answers no queue and `Bypass::Never`, and never
  `Queued`. Reads batched into one query are not added.
- **Credentials:** the App (section 7); git over HTTPS with an
  installation's token narrowed to contents.
- **Deliveries:** `X-Hub-Signature-256`, `X-GitHub-Event` and
  `X-GitHub-Delivery`, one URL for the App, each naming its
  installation. A pushed delivery and one read from the App's delivery
  log go through one decoder (github.md, section 9).
- **CI:** check runs and Actions' jobs, whose logs come as a redirect to
  a download host the component reaches (github.md, section 10).
- **Setup:** the App manifest flow (setup/README.md). There is no owned
  mode.
- **The fake:** a GitHub protocol layer over the same domain
  (../testing/fakes/github.md, section 1).

## 9. Verification

The dialect's step tests, in `skein-forge`, with no io:

- **Documents.** Each decoder and encoder is tested:
  - against the conformance's transcripts;
  - against answers generated from the seed;
  - against mutations of both.
- **Encoders.** Golden bytes for each encoder; bounds at the limit and
  one past.
- **Decoders.** A fuzz target per decoder; the bounds at the limit and
  one past (`string_bytes`, `body_bytes`, `inner`, the stack's depth); a
  string cut at every byte; the server's encoders met by the client's
  decoders, and the client's encoders by the server's decoders.
- **Plans.** Each plan against canned answers, for every call of `forgejo-operations.md`, 1:
  - the page size, the smallest of three (`forgejo.md`, 1.1);
  - names percent-encoded, a slash in a branch's name `%2F`;
  - times read and written, 2554 included;
  - each failure of `forgejo.md`, 1.4, in the status and the message that tell it;
  - the first failure ending a call;
  - a request that was never seen and one that may have been, for a
    write and for `OpenPull`'s reads;
  - the tables filled from the answers heard, and a row dropped.
- **The marker.** Encoded and decoded with several words, a word of 1
  and of 16 bytes, a body whose marker carries another word, an odd
  count of digits, an upper-case digit, a missing line feed, and a key
  at the limit of `op_bytes`.
- **Deliveries.** Each event of section 6 decoded to its hint; a
  signature right, wrong and missing; a branch's deletion; a tag; a job
  that ends; a repository id past 32 bits.
- **Transcripts** are format-faithful and content-irrelevant: volatile
  fields (times, ids, hashes) are masked, and no credential or secret
  reaches a file (../testing/fakes/throwaway.md, section 2).

The component's machine world, the protocol world, the client's world,
the setup's world and the conformance are in connection.md,
client.md, setup/README.md and ../testing/fakes/forge.md. The worlds are under
`tests/forge/`, one crate each (`machine`, `protocol`, `client`,
`setup`); the conformance, `tests/forge/conformance`, is a tool outside
the suites (../testing/README.md, section 6).

## 10. Open questions

- **A real Forgejo's refused reads, unexplained** (`vocabulary.md`, 1.4, `forgejo.md`, 1.3): on a deployment,
  an agent's `PullFor` was refused `InvalidAnswer`. The answers captured there
  show a comparison's `removed` (`forgejo.md`, 1.3), and no cause for that: none is a by-branch
  answer, and none was read with CI red. What would settle it is that answer,
  with its request path and the head and base asked, and a section's reads at a
  red head. A comparison whose `commits` come shorter than its `total_commits`
  is refused the same way (`skein-forge/src/forgejo/plans/commits.rs`); whether
  Forgejo answers one on a long range is unknown. Default: as they stand, until
  an answer shows a fault.
- **Dismissed reviews:** whether one counts as no verdict, or as the
  comment it was. Default: the comment it was.
- **Contents on demand.** A pull request's files with `sides` cost two
  reads per file, since `File` carries contents. Default: a reader that
  needs paths alone, as an overlap, asks without `sides` (`vocabulary.md`, 1.2); one that
  needs contents asks them for the whole page, or reads one path by
  `File` at each commit; a comparison reads names alone (`vocabulary.md`, 1.2).
- **A file too large for the answer** fails its call as `TooLarge`.
  Default: so; a side cut with a mark would need `File` to carry one.
- **Bodies past `body_bytes`** are carried empty. Default: so, the
  component counting them; a skein-json read that keeps a string's first
  bytes and digests the rest would let it keep their heads and keys.
- **An edit within a creation's second** reads as `Original`, so a
  person who edits the owner's keyed comment within that second is not
  heard. Default: `Original` when the times are equal.
- **The merge style:** the repository's `default_merge_style`. A style
  an owner names per repository would need `Write::Merge` to carry it.
  Default: the repository's.
- **An update's real time** (`forgejo.md`, 1.4): aborting its call does not stop the
  forge's git work. How often that work outlasts the client's `lifetime`,
  and what another copy costs, await measurement (client.md, 11).
  Default: the lifetime stays, and recovery uses the idempotent class.
- **An expired or revoked credential** reads as `Forbidden` on every
  call until the next start, which it stops (connection.md,
  sections 3 and 5). Default: so; the owner could tell it apart, so a
  person is told once.
- **What the dialects still write apart.** Their field walkers, over
  different masks and `Want`s: `dialects.md`, 1.4 takes what depends on no forge's
  documents, and joining the walkers is a redesign. And
  `skein-forge-setup`'s `host/limits.rs` builds the one `Limits` in two
  functions, `documents` and `github`. Default: the walkers stay apart;
  the setup's two functions can become one, which nothing asks for yet.
- **The conformance on macOS.** A bottle is the pin's version and another
  build (`dialects.md`, 1.3), so what the conformance checks is Linux's binaries, and a
  bottle answers only to temper's one story
  (`temper/docs/design/testing/tiers.md`, 1.8). Default: so, until the
  conformance joins the macOS workflow with skein's suites
  (`temper/docs/design/draft/macos.md`, section 6).
- **GitHub's further gaps.** Settled: its dialect is a module of
  `skein-forge`, and section 8 lists what it adds. Open:
  github.md, section 15.
- **What the call plans' states leave** (`dialects.md`, 1.2): a plan's cells are arms of one
  `match`, where ../../foundation/programming-model.md, section 5.4 has one handler each.
  Default: as they stand, until a debt item names them. The `if let`s
  `forgejo/client` and `forgejo/server` keep on their own enums go with `D015`
  of `skein/docs/development/debt/`.
- **A document kind no plan names** (`dialects.md`, 1.2): the exchange's decoder picks the kind,
  so a plan's match over a decoded document ends in `_` under a named `#[expect]`,
  where ../../foundation/programming-model.md, section 10.3 has no `_` on an own enum, and a new
  `d::Document` kind goes silent there. Decided under the user's standing word,
  for the user's later review (`README.status.md`). Default: so.
