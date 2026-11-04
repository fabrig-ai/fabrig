# Request: the withdrawn compatibility forms removed

From status-gaps (its item 5, question `withdrawn-forms`), answered unattended on 11-03: the
director's decision (the user asleep; the recommended option taken; central log
`/var/www/status/user/decisions.md`). The design is `skein/docs/design/llm.md` 4.6 and 4.7 and
smith's designs; the gaps are skein's llm.status.md 4.6 and 4.7, smith's STATUS.md and its
agent, channel, hosts and llm status files.

## The question, as asked

Removing the withdrawn forms (skein llm.status.md 4.6 and 4.7; smith STATUS.md and its agent, channel, hosts and llm status files) reaches further than the status files say, into things that are the user's:
1. smith's product still runs on them, and no status file records it: smith-local-shell's presets hold a codex endpoint (dialect codex, the default account codex) and Anthropic's with identity claude-code; its ChatGPT sign-in reads the chatgpt_account_id claim and smith-local-service sends the withdrawn grant. smith's shell.md already names only openai and anthropic presets and refuses identity and dialect codex. Removing them ends smith's Codex (ChatGPT) and Claude Code identity routes for its users. The sign-in-wait draft is changing smith-local-shell/src/credentials/product.rs now.
2. smith's channel wire: the withdrawn grant (channel-v5.schema, tag 2) and Cap::AccountId (tag 20) leave it, and smith's protocol/README.md 8 makes any change a reader notices a new version: channel v6 (jig names smith_channel::v5 in five files), or keep v5 and refuse those values.
3. Stored data: charters and transcripts at dialect 1 and skein-llm's replay tag 1 stop resuming (local chats on the codex preset); skein-llm's Codex recordings (openai/fixtures) cannot be captured again, since no probe of a real service is allowed; temper-init's shipped agent.json template and deployments' agent.json carry "identity", which then is refused.
4. jig's rule for an endpoint's dialect, which its design leaves open: accept only 2 and 3 (moving its fixtures that write 0 and 4), refuse 1 alone, or refuse nothing and let the charter's decode fail.
Options: (a) recommended: remove them all as the designs say, in a later cut of five tasks in order (temper's configs and literals; jig's fixtures and its refusal of any dialect but 2 and 3; smith's product, after sign-in-wait lands; smith's agent, protocol and channel at v6; skein's routes), with smith's shell.status.md first recording the product's presets as a gap; the Codex recordings deleted, dialect-1 stores started afresh, temper's template without "identity". (b) remove only what smith's product does not use (the endpoint's account, the id-and-bearer grant's channel arm at v6) and keep the Codex and Claude Code routes, the designs and status files then saying they stay. (c) defer the item until smith's product has its replacement routes.


## The answer: (a)

Remove them all as the designs say, in five tasks in this order:

1. temper's configs and literals (temper-init's shipped agent.json template and deployments'
   agent.json without "identity");
2. jig's fixtures, and jig's refusal of any endpoint dialect but 2 and 3 (its fixtures that write
   0 and 4 moved);
3. smith's product (smith-local-shell's codex and claude-code presets, the ChatGPT sign-in's
   chatgpt_account_id read, smith-local-service's withdrawn grant), **after sign-in-wait lands on
   main**: it changes smith-local-shell/src/credentials/product.rs now;
4. smith's agent, protocol and channel at v6 (the withdrawn grant, tag 2, and Cap::AccountId, tag
   20, leave; jig's five `smith_channel::v5` names move);
5. skein's routes (skein-llm's codex dialect, the account-id grant, the identity profile,
   Anthropic's bearer route); skein-llm's Codex recordings (openai/fixtures) deleted, never
   captured again (no probe of a real service).

First, smith's shell.status.md records the product's presets as a gap. Stored dialect-1 charters
and transcripts and skein-llm's replay tag 1 start afresh: no migration. Each task updates its
tree's status files.
