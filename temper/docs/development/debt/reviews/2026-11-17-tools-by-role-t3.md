# tools-by-role T3: the count

2026-11-17. Milestone T3 of `temper/docs/design/draft/tools-by-role.execution.md`: the figures `temper/docs/design/draft/tools-by-role.md`, 8 asks for, read on dev.fabrig.ai after T2's redeploy. T2 is goal 5617, on main at `298f6c145b`, and has run on dev since 11-17 19:48Z. The user read the figures from dev's turn records (`task turn TASK A.T --json`, its `decoded.used`) at 11-17 23:2xZ, in task 7172.

"Input" below means input, cache-read and cache-written tokens added together, the sum the draft's figures use. The turn records name no model, so each turn's model is its task's LLM. Each run's model is its baseline's, so the two sides count the same tokenizer's tokens.

## 1. In one page

- **A designer's first request** fell from 33,974 input tokens (goal 4426) to 25,808–26,536, between 21.9% and 24.0% less.
- **A coder's first request** fell from 14,684 input tokens (task 4299) to 11,481, 21.8% less.
- **A goal's request of up to 4 KB arrives whole.** The largest request since the redeploy, 3,540 bytes, reached its designer's first turn whole, with no `[cut` marker. Before T2, goal 4426's 2,608 bytes were cut after about 640.

## 2. The first requests

Each one is the run's first turn: the charter's text, the brief and the host tools' declarations.

| Run | Turn | Charter | Model | Input | Cache-read | Cache-written | Output | Total input | Against the baseline |
|---|---|---|---|---:|---:|---:|---:|---:|---:|
| Designer, goal 4426 (baseline) | 6336.1, 11-16 02:03Z | 5 | Opus 5.5, xhigh | 2 | 21,954 | 12,018 | 281 | 33,974 | — |
| Designer, goal 7112 | 10129.1, 11-17 20:04Z | 5 | Opus 5.5, xhigh | 2 | 13,286 | 13,248 | 288 | 26,536 | −7,438 (−21.9%) |
| Designer, goal 7225 | 10295.1, 11-17 21:49Z | 5 | Opus 5.5, xhigh | 2 | 13,286 | 12,520 | 208 | 25,808 | −8,166 (−24.0%) |
| Designer, goal 7166 | 10227.1, 11-17 21:20Z | 17 (the lean design, template 16) | Opus 5.5, xhigh | 2 | 13,286 | 12,523 | 236 | 25,811 | −8,163 (−24.0%) |
| Coder, task 4299 (baseline) | 6185.1, 11-16 00:20Z | 2 | gpt-6.1-sol, high | 8,028 | 6,656 | 0 | 56 | 14,684 | — |
| Coder, change 7145 | 10131.1, 11-17 20:21Z | 2, at revision 48 | gpt-6.1-sol, high | 11,481 | 0 | 0 | 46 | 11,481 | −3,203 (−21.8%) |

- **The like-for-like pair** is 4426 against 7112 and 7225, all three on charter 5. Goal 7166 runs charter 17, a different text, and lands within 3 tokens of 7225.
- **The cached prefix is the same, 13,286 tokens,** in all three designers after the redeploy. Before T2, 4426's was 21,954. That prefix is the part every goal's first run shares; what varies between runs is mostly the brief, written to the cache.
- **Change 7145** was the first coder's attempt to start after 19:48Z. Its request read nothing from the cache, so its whole input shows as input. That doesn't change the total.

## 3. A request of up to 4 KB

Goals started since 19:48Z, with their request's bytes as their designer's first turn received them:

| Goal | Request | In the first turn |
|---|---:|---|
| 7112 | 3,540 bytes | whole: no `[cut` marker, first and last lines both present |
| 7225 | 1,491 bytes | whole |
| 7166 | 582 bytes | whole |

Goal 7104 started at 19:06Z, before the redeploy, so it isn't counted.

The bound behind it: the standard profile's budget for the brief's task section is 4,096 bytes (`jig/crates/jig-protocol-config/src/profile/standard.rs`, `BUDGETS`). That budget is part of the 13,312-byte brief floor the settings check holds since T2 (`temper/docs/design/domain/agent/room.md`, 4.3). A charter that passes apply leaves room for a 4,096-byte request whole. The largest request seen here was 3,540 bytes. No request between 3,540 and 4,096 bytes was observed.

## 4. The offers

From the design (`temper/docs/design/domain/agent/declarations.md`, 1 and 3) and the draft's figures before T2 (`tools-by-role.md`, 1):

| Role | Before T2 | After T2 |
|---|---|---|
| Stock goal, charter 5 at policy 1's authority | 42 tools, 36,512 bytes | 22 tools, 14,336 bytes at most |
| Stock coder, charter 2, questions up | 36 tools, 25,570 bytes | 7 tools, 4,096 bytes at most |

- **22, not the draft's 21.** The draft counted 21 with the forge's reads as two tools. T2 made them three: `read_forge`, `read_issues` and `read_ci` (`temper/docs/design/domain/agent/tools.md`, *The forge's reads are three tools*).
- **Dev's goals are offered 27.** Dev's charter 17 names all ten families (tools 1,023), and dev's policy authority holds decide, propose and effect, which policy 1's leaves out. So this goal's run was offered the stock goal's 22 tools plus `decide`, `decide_escalation`, `withdraw`, `propose` and `effect_forge`. This is a property of dev's settings, not of the stock offer. The user confirmed it in task 7172.

## 5. Reading the figures

- **Most of the saving is in the shared prefix.** A designer's shared prefix fell by 8,668 tokens (21,954 to 13,286), more than the whole drop. The declarations are what T2 changed there. The records don't say whether the charter texts also changed between 11-16 and 11-17, so this count can't attribute every token to T2. The same is true of the coder's 3,203 tokens: its charter was at revision 48 at 20:21Z, and 4299's revision isn't in these figures.
- **The rest of a first request** is the charter's text, which belongs to its keeper; Claude Code's own six tools, outside the cap; and the brief. Those are where any further saving would come from, and none of them is this draft's.
- **The brief now gets room.** Goal 4426's designer spent a turn on `read_task` to read its cut request. The goals counted here were given their whole request in the first turn.
