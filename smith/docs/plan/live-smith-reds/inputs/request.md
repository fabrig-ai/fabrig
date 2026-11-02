# live-smith's standing reds: six live stories red after landing, none about credentials (11-02)

smith's live pair (`live-smith`, the root `.config/nextest.toml`) runs again after every landing,
its logins lent from the shelf. At main da9027dbf5ae its run was red with seven standing reds. One
is the API-key story, `smith::live an_anthropic_key_delivers_a_commit_in_place`, which needs an API
key no shelf holds: out of scope here. The other six are this plan's, each a standing red (it was red
before red-routing existed, so it starts no repair by itself):

| Test (binary, name) | Seen (a summary of the reds, approximate: the log is the record) | Log on hetzner8 |
|---|---|---|
| `smith::live a_run_refreshes_its_grant_at_the_issuer` | a forced refresh does not bump the login record's generation (smith/crates/smith/tests/live.rs ~34) | `FAB_HOME/after-landing/da9027dbf5ae-20261102T102506619199Z/live-smith-2.log` |
| `smith::live a_chat_crosses_from_chatgpt_to_claude_then_back` | exec fails on the Claude leg, diagnostics withheld (live.rs ~334/344) | `…/live-smith-1.log` |
| `smith::live claude_delivers_a_commit_in_place` | exec fails on the Claude story, diagnostics withheld | `…/live-smith-5.log` |
| `smith::live a_second_run_resumes_the_chat_from_its_files` | exec fails on resume, diagnostics withheld | `…/live-smith-3.log` |
| `smith-agent-process-world::live program::a_live_server_with_a_non_object_schema_lists_no_tools` | Claude Code's MCP init shape differs from the probe's (program.rs ~388): likely CLI drift | `…/live-smith-6.log` |
| `smith-agent-process-world::live program::a_live_thinking_turn_has_the_fakes_shape` | the thinking signature's shape differs (program.rs ~478): likely CLI drift | `…/live-smith-7.log` |

(`FAB_HOME` is fab's home on hetzner8: `~/src/fabrig/fab`.)

## What to do

For each: read its log, find the cause in the lowest world that shows it (smith's own fakes and
domain tests first: a probe shape that drifted belongs in the fake or the fixture it mirrors), and
repair it with a test there, red before, green after. Where a live test's expectation is the thing
that is wrong (the real CLI changed, smith's fake did not), bring the fake and the expectation to
the real shape and say which CLI version showed it. Where a cause is outside smith's code (a
machine's install), say so and leave the test as it is. Exec failures that withhold diagnostics:
first make the failure say why (a test that a failing exec's reason reaches the report), then the
cause.

## Acceptance

`live-smith`'s after-landing pair runs these six green on a main holding the fixes, or a test is
moved or ignored with a reason its design names. **A fix is checked by the after-landing pair, not
inside the implementer's run:** the live stories need logins lent to the pair, which an
implementer's slot does not have; run the fakes, domain and focused tests in the slot, and leave the
live run to after-landing.

## Out of scope

The API-key story; fab's after-landing code; the shelf's logins.
