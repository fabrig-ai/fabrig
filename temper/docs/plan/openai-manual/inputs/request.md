# Request: live-skein's two OpenAI stories go manual too

From the director, 2026-11-03 ~11:00Z (unattended; the user takes the director's recommendation),
on after-landing's red-10. Since withdrawn-forms (9b4a756cd7) skein's OpenAI live stories call
the public Responses route, and the Codex login lent to live-skein gets `401 Missing scopes:
api.responses.write`. No shelf login serves that route in a form skein reads (agents/creds,
05:35Z); the user's sign-in stays on their list for later. So, like the three Anthropic stories
(red-10's repair, 05:57Z), the two positive OpenAI stories become manual. The design is
`skein/docs/design/llm.md` 5 and its status file's section-5 entry.

One task, `skein/tests/llm/tests/live.rs`:

- give `openai_streamed_text_and_history` (~505) and `openai_tool_round_trip` (~514) each
  `#[ignore = "live: needs a login that holds api.responses.write; run by the owner, how is decided later (skein/docs/design/llm.status.md, 5)"]`,
  in the form of the Anthropic ones beside them; leave `openai_unauthorized` running (it needs
  no valid login) and `live.rs::input` and its readers as they are;
- `skein/docs/design/llm.status.md`, section 5: the two OpenAI stories join the Anthropic ones
  as manual, decide how to run them later, with the reason (the lent Codex login lacks the
  public route's scope);
- `skein/crates/skein-llm/README.md`'s live section: these two are ignored too, run alone with
  `--run-ignored only -E 'test(=NAME)'`.

Check: `heavy cargo nextest run --profile live-skein -p skein-llm-world --test live` lists only
the non-ignored stories (no provider call needed to check the listing: `--list` is enough).
