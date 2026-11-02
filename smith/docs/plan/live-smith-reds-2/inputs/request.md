# live-smith: three live stories still red after landing (11-02)

smith's live pair (`live-smith`, the root `.config/nextest.toml`) runs after every landing with
logins lent from the shelf. Plan live-smith-reds (landed 11-02 11:49Z) recovered three of its six
standing reds and made the failures say why. At main 46e6e70d9 three stay red, each standing, so
none starts a repair by itself. Their logs are on hetzner8, in
`~/src/fabrig/fab/after-landing/46e6e70d9b17-20261102T170025232765Z/` (`live-smith-1..3.log`):

| Test (`smith::live`) | Panics at (smith/tests/end-to-end/live.rs) | What the log says |
|---|---|---|
| `a_chat_crosses_from_chatgpt_to_claude_then_back` | ~531 | step Exec, `failed=checkout file matches its prescribed content`; the run completed (3 turns: list, write); exit 0 |
| `a_run_refreshes_its_grant_at_the_issuer` | ~45 | "the issuer refresh was kept; generation before=7 after=7"; the run completed (7 turns) |
| `claude_delivers_a_commit_in_place` | ~569 | step Exec, `failed=init provides a required built in or smith tool`; backend ClaudeCode; the run completed (4 turns, two Bash calls) |

## What to do

For each: read its log and the story, find the cause in the lowest world that shows it (smith's
fakes and domain tests first), and repair it there with a test, red before and green after. Where
the story's expectation is what is wrong (a real model or CLI that legitimately differs from what
the story prescribes, e.g. Claude Code's init no longer listing a tool the story requires), bring
the expectation and smith's fake to the real shape, and say which CLI version showed it. Where the
story's check reads too strictly for a real model (a prescribed file content a model may word
differently), say what the design requires and test that. Where a cause is outside smith (a
machine or a login), say so and leave the test as it is.

## Acceptance

`live-smith`'s after-landing pair runs these three green on a main holding the fixes, or a test is
moved or ignored with a reason its design names. A fix is checked by the after-landing pair, not
inside the implementer's run: the live stories need logins lent to the pair, which an implementer's
slot does not have; run the fakes, domain and focused tests in the slot.

## Out of scope

The API-key story (manual, by the user's decision); fab's after-landing code; the shelf's logins.
