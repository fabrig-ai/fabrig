# fab3-fixes: the plan

From `inputs/request.md`: the three findings of the acceptance review that refused
forgejo-connect and withdrawn-forms (agents/loop, 2026-11-03). One direct part, three independent
tasks, all released in the first push; every task names `notes/common.md`.

## The design

No design text changes. `temper/docs/design/client/forge.md`, 5.4 already says what the tokens
fix builds ("empties the three fields as it leaves; a stop empties them again": the sent
connect's own stop); `client/forge.status.md` lists no gap for it, so none is added or closed.
`STATUS.md`'s summary is the one design file edited, by `status-summary`.

## Parts and tasks

1. **fixes** (`temper/docs/design/client/forge.md, 5.4`), slice 1.
   - `tasks/tokens-stop.md`, high, ~60 minutes, needs nothing: `temper-client-domain`'s
     `forge/tokens.rs` keeps on `Pending` what the sent connect has seen of its forge's row
     (`Seen::Open { stopped, ended }` or `Settled`) and empties the draft only on that row's new
     stop; with no connect sent, no row empties it. Tests in `tests/forgejo_tokens.rs`; the
     readers' tests in `temper-client-view` and `temper-client-world` run.
   - `tasks/status-summary.md`, medium, ~15 minutes, needs nothing: `temper/docs/design/STATUS.md`
     drops the connected Forgejo's closed lag and names `client/tools.status.md` 11.3, the real
     loop's story, in its place.
   - `tasks/agent-readme.md`, medium, ~40 minutes, needs nothing:
     `smith/crates/smith-agent-shell/README.md` shows OpenAI's public route (provider `openai`,
     dialect 3, `api.openai.com`), drops `identity`; a unit test decodes its fenced JSON with
     `config::decode`; the README joins `.config/gate/read-by-tests.txt` and `smith/AGENTS.md`'s
     list.

Critical path: `tokens-stop` alone (~60 minutes); the other two run beside it.

## What was decided, and why

- **What the sent connect remembers.** `tokens::Pending` stays the one record of a sent connect
  (it already outlives its answer); it gains a private `Seen`, taken from the checked row at send
  (a new forge's starts not stopped, no `ended`, and follows the number `forges.rs`'s `bind_new`
  gives it). A new stop is the row stopped (Stopped, or NotConnected with `ended`, today's test)
  where the connect last saw it not stopped, or with other `ended` words. That stop, the row
  Connected, or an answer that starts no connect settles it, so a later stop of the same forge
  empties nothing: the request's "only on that attempt's new stop transition". The shape follows
  `forge/replace.rs`'s row drafts, which already keep `stopped` and `ended` and clear on their
  change.
- **The stop that cannot be told apart.** A second connect whose stop has the same connection and
  the same words as the first, with nothing delivered between, is not seen as new: the draft was
  emptied at send and at the answer already, and telling them apart would need a request id on
  the row, a wire change no finding asks for. The spec says so.
- **`forges.rs` stays out.** The fix is inside `tokens.rs`; `forges.rs` keeps its call, so the
  task avoids the file `first-run-projects` and `reference-m3` are changing.
- **The README's test uses `decode`, not `parse`.** `parse` and `Decoded::resolve` resolve the
  endpoint's address and load the machine's trust: a name lookup of `api.openai.com`, a probe.
  `decode` checks every field, the endpoints' dialect and provider, the models and the derived
  limits, which is what keeps the README runnable.
- **No answer waits on the user.** No `[[question]]`.

## Overlaps with the drafts beside this one

- `tokens-stop` edits `temper/crates/temper-client-domain/src/forge/tokens.rs`, inside
  `reference-m3/kinds-ctl-reads`'s glob `temper/crates/temper-client-domain/src/forge/**`; that
  task's work is the kinds' reads, not the tokens form, so a textual overlap is unlikely. Its test
  file `tests/forgejo_tokens.rs` is in no other draft's list. `first-run-projects/welcome-ready`
  and `setup-project-goes` change `forge/forges.rs` and other client-domain tests, which this plan
  does not touch.
- `status-summary` and `agent-readme` touch no file another draft lists.

## Seen beside the findings, not planned

- `temper-client-domain`'s `forge/admin.rs` `stopped` (~376) has the shape finding 1 fixed: with
  no admin connect sent, any stopped admin row empties the admin form's token, and while its
  connect's forge stays stopped, every delivery empties a token pasted again. The request names
  only the bot tokens; this is for the user or a later draft.
