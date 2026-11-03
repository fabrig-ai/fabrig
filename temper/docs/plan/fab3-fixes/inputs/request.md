# Request: the acceptance findings of forgejo-connect and withdrawn-forms

From agents/loop, 2026-11-03 ~03:27Z: Codex's acceptance review of three landed plans
(interrupted-calls accepted) refused forgejo-connect and withdrawn-forms on these findings. One
direct part, three independent tasks.

1. **A fresh token paste survives an unchanged earlier failure** (temper client/forge.md 5.4).
   `temper/crates/temper-client-domain/src/forge/tokens.rs` ~203 and ~224, called on watch updates
   at `forge/forges.rs` ~938: `stopped()` clears the shared draft whenever a row is Stopped, or
   NotConnected with ended set. After a failed connect, tokens pasted again are lost on the next
   forges watch delivery (another forge's update included); with no pending request, any stopped
   connected-mode Forgejo clears a new draft. Fix: track the submitted target and its observed
   stop/ended state, and clear only on that attempt's new stop transition. Tests: extend
   `temper/crates/temper-client-domain/src/tests/forgejo_tokens.rs` ~202: redeliver the stopped
   row after the fresh paste (~218) and assert the new value and its written counter survive; an
   unrelated stopped row while preparing a new forge leaves the draft; the clear-on-send, new-stop
   and sign-out assertions kept.
2. **temper's STATUS.md summary** (`temper/docs/design/STATUS.md` ~27-30) still says a connected
   Forgejo, the tokens' Connect and Replace lag; the detailed status files and the code have them.
   Delete that clause; keep any specific open gap (client/tools.status.md 11.3's deferred
   real-loop story), consistent with client/forge.status.md.
3. **smith-agent-shell's README prescribes removed forms**
   (`smith/crates/smith-agent-shell/README.md` ~17-29: provider codex, dialect 1, endpoint
   account, chatgpt.com; ~46: identity plain/claude-code). Startup rejects them. Replace the
   fenced configuration with a valid current OpenAI one (provider openai, dialect 3,
   api.openai.com address and server name, no endpoint account), drop identity from the optional
   fields, and add a test that parses the README's fenced JSON with the agent's config parser so
   it stays runnable.
