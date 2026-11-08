# survey-answer: a read-only survey's answer reaches its lead whole

A goal's first run on a real deployment, at fabrig `7638040201`, asked a read-only survey through `sub_agent` (inspect, `gpt-6.1-sol`, a request for at most about 1,500 words). The survey ran 14 turns in 150 s and ended its turn, but the lead got exactly 1,024 UTF-8 bytes, stopped mid-sentence, then `[child-result cut=23 stop=end-turn]`. The lead said the answer came back cut short and read the files itself, which is the survey's whole cost spent twice. That run's evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing test first, below the live tier:** a sub-agent's answer of a few KB, through smith's delegation, reaches the run that asked it whole. A row of an existing delegation test if one fits.
2. **Find the bound that cut it at 1,024 bytes** and why it's so small on a deployment whose call room is 63,740 bytes. Look first at smith's `delegated_result_bytes` and the `feedback_bound` it's derived from (`smith/crates/smith-agent-service/src/profile.rs`), and at how jig's and temper's profiles feed them. Then size it from what the design holds: an answer is a run's result, so the room a call's result has. If the design doesn't say, say so and size it from the call's payload, with the start's checks naming it.
3. **A cut answer says how to read the rest,** if one can still be cut: a continuation, or the cut's words saying what was left and that the lead can ask again, narrower. `cut=23` alone tells it nothing.
4. **The design says so,** in smith's limits design and the configuration section that lists what follows a call's payload.

## Not in this plan

How a goal's close-out reads its siblings' reports: a design question being settled.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. Formats change in place before v0.1.0. smith names neither jig nor temper. `walk = false`.
