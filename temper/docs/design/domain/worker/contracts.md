# Contracts: a squash's head trailers from the checkout

The shapes the changes of one goal share, and no other area reads, while
they move a squashed member's head trailers from the landing's reads of
the forge into the prepare job's checkout
(`../../draft/design-economy.md`, 4.3). The lasting design is
`../landing-prepare.md`, 1.2, `../../protocol/tooling.md`, 4,
`jobs.md`, 2 and `../../protocol/git-operations.md`, 1.9. The goal's
last change deletes this document and its status.

## 1. The member messages' crate

- **`temper-protocol-member-message`,** a crate at
  `temper/crates/temper-protocol-member-message`, holds what
  `temper-engine-domain-forge`'s `member_message` module holds, moved
  whole, with the same names, signatures and meaning: `Member`, `Kind`,
  `commit_message`, `trailers`, `names_head` and `TRAILER_BYTES`
  (`../../protocol/tooling.md`, 4). It names skein's crates alone
  (`skein-lib`, and `skein-forge` for the head's `Commit`) and no
  engine or worker crate, so that the worker's checkout can name it.
  `temper-engine-domain-forge` keeps no `member_message` module and no
  re-export of it: each reader names the crate.
- **`pub fn with_head_trailers(squash: &[u8], head_message: &[u8]) ->
  Box<[u8]>`,** in that crate: given a member's squash message as
  `commit_message(member, Kind::Squash)` composed it for a member whose
  words keep no TRAILERS, its own TRAILERS none or `trailers(head_message)`,
  and that member's head commit's message, the message
  `commit_message` composes for the same member with `head_trailers`
  set to `trailers(head_message)`. So for any such member `m` and any
  message `h`, `with_head_trailers(&commit_message(&m, Kind::Squash), h)`
  equals `commit_message(&m', Kind::Squash)`, `m'` being `m` with
  `head_trailers: &trailers(h)`, whether `m.head_trailers` was empty
  or already `trailers(h)`. A `squash` it cannot read as a squash
  message (a title line, an optional description paragraph, and a last
  paragraph ending in temper's `Change`, `Pull-request` and `Head`
  lines) comes back unchanged.

## 2. The checkout's message read

- **`Op::Message { at: Place, commit: Commit }`** and **`Done::Message {
  message: Option<Box<[u8]>> }`** in `temper-worker-domain-checkout`'s
  `git` module (`../../protocol/git-operations.md`, 1.9): a local,
  read-only operation, its timeout class a local one's, `message` the
  commit's message bytes, or `None` where the output passed
  `stdout_bytes`.
