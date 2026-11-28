# claude-api: the reasons behind it

This holds the reasons for `claude-api.md`, moved from the shared notes file.

The reasons behind `claude-api.md`, from the user, 2026-11-11.

- **What they asked for:** "add support for using claude via API token, that would be via smith agent natively (no need to call out to native claude binary), since that's allowed by Anthropic terms of use (please check)", and "Initially it's sufficient to expose this via "temper ctl" (basically you can add an account in temper that uses a Claude API token)."
- **The terms, checked:** Anthropic's pages, read that day, allow a customer's own API key in software built on Claude, and forbid routing a subscription's credentials through it (`claude-api.md`, section 2).
- **One provider, not two:** a second, key-only Claude provider beside Claude Code's was weighed first, as configuration alone. It fails on the catalogue: jig refuses two models of one name, whatever their provider, and charters name a model by its name, so a second provider can't carry its own `claude-opus-5-5`. Making a model's identity its provider and name would be work that one provider never needs. One provider whose source picks the route keeps every piece.
- **In two steps:** "let's build B in two steps as you recommend": step 1 in v0.1.0, and "we'll evaluate the second one later, if it fits in v0.1.0 or not."
- **The draft's readings,** not the user's words: a subscription limited everywhere holds its account's runs, as jig's order already says, rather than spending the key after it, until step 2 weighs that; Claude's cache prices replace the stock's cautious ones, since Anthropic's page gives them and a key pays them; the pages draw the key where they draw OpenAI's, with no new look.
