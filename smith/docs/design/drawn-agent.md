# A drawn agent in smith's worlds

Provisional, 2026-11-27. What the fake model says in smith's worlds when no script answers: the menu it copies its calls from, the slots that name what the run made, the refusals it targets and how each world provides them, and how its draws are weighted. The fake's mechanism is skein's (`skein/docs/design/testing/fakes/drawn-agent.md`): this document is the same subject seen from smith, and holds what is smith's. What the worlds count of it is [shapes.md](shapes.md). Written from `temper/docs/design/draft/misbehaving-agents.md`, 5.1.

## 1. In one page

- **The menu is the session's vocabulary** (section 2): a body for every tool a world grants, each carrying every field the product's schema has, and the world's translator offering and decoding every tool as `smith-protocol-llm` does.
- **Slots name what the run made** (section 3): `job` from a shell's `job N`, `read`'s offset from its continuation, and a path from the input, a listing or a search hit.
- **Each refusal it targets is listed** with its setup and the answer that shows it (section 4). Most need a body alone; the rest need prior state, timing or scarce capacity, which the noisy settings provide.
- **Each reference is classed** by the world as the answer arrives (section 5), and the class goes into the completion's shape.
- **Weights lean toward what a domain refuses** (section 6), above all a completion whose earlier call is refused at the entrance before a write, with reads after it.
- **Every world takes the same menu with its own grants** (section 7), and the noisy sweep asks that each targeted refusal fall (section 8).

## 2. The menu: a body for every tool

- **Every tool the session offers** under its grants (`domain/tools.md`, 3): `read`, `list` and `search` under inspect, `write`, `edit` and `remove` under modify, `shell` and `job` under shell. The world's translator (`smith-session-world`'s `translate`) offers each tool a grant gives, with the fields the product's schema offers it, and decodes each call as `smith-protocol-llm` does: the same call, or the same problem, for every body and each cut prefix of one, which `smith-protocol-llm-world`'s `the_session_fake_decodes_like_the_protocol` compares. A field the product decodes, the translator decodes: `read`'s `offset` and `window`, `list`'s `depth` and `glob`, `search`'s `glob`, `edit`'s `all`, `shell`'s `timeout_seconds`, `output_bytes` and `yield_seconds`, `job`'s `wait_seconds`, `output_bytes` and `stop`.
- **Bodies are supersets.** The fake draws a tool and a body apart (`testing.md`, 4), so each body carries a field for every tool: a path, an offset and a window, a depth, a glob, a pattern, a content, edits, a command, its timeout, output budget and yield, a job, its wait and `stop`. A call reads its own tool's fields from any body. A field a body leaves out takes the product's default.
- **Most values are valid; some refuse.** A body's values name the world's checkout (`README.md`, `src/lib.rs`, the guide in `docs`, the missing `notes.md`) or a slot (section 3). Some bodies carry one value that a tool refuses (section 4), so that a drawn call is refused for a reason the product gives, in the field it reads.
- **The invalid entries stay:** an unknown tool's name, a cut object and an empty one, which the fake copies as written.

## 3. Slots

The menu declares five slots (`skein/docs/design/testing/fakes/drawn-agent.md`, 2), each found where the product's renderer puts the value it names (`protocol/llm.md`, 5):

| Slot | Kind | Before | After | Never issued | Weight | Where a model sees it |
|---|---|---|---|---|---|---|
| `job` | number, 1 to the largest `u32` | `job ` | a space | `999999` | 100 | a running shell's `job N running`, a job's output header (`testing.md`, 4) |
| `offset` | number, 1 to the largest `u32` | `[next line ` | `]` | `999999` | 100 | a read's continuation |
| `listed` | text | a line feed | a tab | `never-issued.txt` | 100 | a listing's entry, `NAME`, its tab and its kind |
| `found` | text | a line feed | `:` | `never-issued.txt` | 100 | a search hit, `PATH:LINE: TEXT` |
| `named` | text | `` ` `` | `` ` `` | `never-issued.txt` | 100 | a path the input names, between backticks |

- **Bodies use them** as a take's values are used: `"offset":"{{#offset}}"`, `"job":"{{#job}}"`, `"path":"{{listed}}"`. Some bodies name a path by a slot and others by a literal, so a drawn call names a file it was shown, stale or not, one it never saw, or one no world issues.
- **The input names its files.** The world's openers give their sessions a task that names the checkout's files between backticks, as a person writes them, so `named` has candidates from the first turn.
- **A value is the product's text,** never a copy fixed for the world: a listing's entry is relative to the listed directory and a hit's path to the searched one, as the product renders them, so a drawn call may name a path its tool resolves elsewhere. That is a model's mistake too.
- **An entry with no line feed before it,** a listing's or a hit list's first where the renderer starts the text with it, is no candidate. Others of the same answer are.
- **The world never issues `never-issued.txt`:** no seeded file and no menu body makes it.

## 4. The refusals it targets

Each refusal below is drawn by the noisy sweep (`domain/session.md`, 10), and its answer is the word that shapes.md, 3 gives it. The text that shows it is the product's (`smith-protocol-llm`'s rendering, `protocol/llm.md`, 5).

| Answer | Refusal | Setup | Shown by |
|---|---|---|---|
| `outside` | a path past the view | a body: `../outside.txt` | `path is outside the workspace: P` |
| `too_long` | a path past the path bound | a body: a path longer than the world's 256-byte bound | `path is too long` |
| `protected` | a write to a protected path | a body: `.git/config` for `write`, `edit` and `remove` | `git directory is protected` |
| `nul_byte` | a NUL byte | a body: a command and a pattern holding `\u0000` | `input contains a NUL byte` |
| `malformed` | a field's value the decoder refuses | a body: a path holding `\u0000`, an `offset` of 0 | `invalid field value "F"` |
| `not_found` | a path not there | a body: `notes.md` before a write makes it, a slot's never-issued path | `path not found: P` |
| `not_file`, `not_directory` | the wrong kind of path | a body: `docs` for `read`, `src/lib.rs` for `list` | `path is not a file`, `path is not a directory: P` |
| `not_empty` | a directory with files | a body: `docs` for `remove` | `directory is not empty` |
| `edit_no_match`, `edit_ambiguous`, `edit_unchanged` | an edit that can't apply | a body each: an `old` the file lacks, one it holds twice, an edit leaving the file as it was | `edit 1 failed: ...` |
| `exit_nonzero` | a command that fails | a body: `false` | `exit code 1` |
| `unknown_tool` | a tool the session doesn't offer | the invalid entry `delete_repository` | `unknown tool` |
| `unknown_job` | a job its session doesn't hold | the `job` slot: a consumed, foreign or never-issued number | `invalid job N: ...` |
| `not_read` | a write before a read | prior state: a write to a seeded file its session hasn't read | `read the file whole before changing it` |
| `stale` | a version changed since its read | prior state: a read, then a menu command whose program rewrites that file (skein's fake checkout's `Program`'s changes), then a write | `file changed or is gone since it was read` |
| `timed_out` | an expired deadline | timing: the noisy settings' drawn tool deadlines against their drawn operation times | `operation timed out` |
| `busy` | the tools' calls all taken | capacity: the noisy settings draw the tools' concurrent calls in some worlds below what a batch and its held jobs' calls ask | `tool is busy` |
| `jobs_full` | every job cell held | capacity: the noisy settings' drawn cells (`domain/session.md`, 10) | `invalid yield_seconds: at most N jobs may run at once` |
| `read_only` | a write to a read-only directory | world: some noisy worlds mount a second directory, `ref`, read-only, and bodies name `/ref/...` | `path is read-only: P` |
| `too_large` | a body past the payload bound | world: some noisy worlds draw the tool payload bound, and the bounds the product derives from it, low enough that one body's content passes it | `write content too large: N bytes, bound B` |

- **Not targeted:** `not_granted`, since a world offers only the tools its grants give, as the product does; `linked`, since no world's checkout holds a symbolic link; `interrupted`, since no world that draws an agent restarts a worker; `call_too_large` and `cut_off`, the provider's bounds, which the fake's own faults draw (`skein/docs/design/testing/fakes/llm.md`, 2.8).
- **A drawn bound is a tiny limit,** as the worlds keep (`testing.md`, 6), drawn for each seed after every other draw, so that it reaches a refusal through the product's own check with no seed added.

## 5. Each reference's class

The world classes each reference a call names as the call's answer arrives, against what it knows of the session's workspace and jobs, and records it beside the call's answer. A class is the world's knowledge then: an earlier call of the same answer may change what it names (`domain/session.md`, 10).

- **A job:** `domain/session.md`, 10's classes.
- **A path,** the call's `path` as the tools resolve it:
  - `live`: a file or directory the session's workspace holds;
  - `stale`: one it held earlier in the case and holds no longer;
  - `foreign`: a place outside the session's workspace;
  - `unissued`: one it never held, the slots' never-issued path among them;
  - `unknown`: a call refused before it names a path.
- **An offset,** a read's `offset` against the file its call names:
  - `live`: a line the file holds;
  - `unissued`: past its last line;
  - `unknown`: a file the workspace doesn't hold, or a call refused before it names an offset.
- **Only a field a call carries** is a reference: a read with no offset names none. The fake's capture takes each slot's class by the call's id where it captures (`Domain::classify`).

## 6. Weights

- **The levers are the menu's.** The fake draws a tool from those offered, a body from the menu and an invalid entry by its `malformed` chance, each alike (`skein/docs/design/testing/fakes/llm.md`, 2), and knows no tool. So the world weights its draws by the share of bodies that carry a refusing value, and by the noisy settings' ranges of `malformed` and of calls per answer.
- **Toward the escaped shape first:** a completion in which a call is refused at the tools' entrance, before a write, with a read after it (`domain/session.md`, 10). Entrance refusals are a body's NUL path or zero offset, a `job` slot's stale or never-issued number, a yielding shell with every cell held, and the invalid entries. Enough bodies carry one that the noisy sweep reaches the escaped shape in more of its cases than before, with its seeds unchanged.
- **Within the budget.** A weight that costs more instructions than the layer's allowance is held, with its figures, for the user (`tools/tiers/budgets.md`, 8): an entrance refusal runs no operation, so weighting toward refusals adds little.

## 7. The worlds and their grants

- **The session world:** the noisy settings grant all three families, and an eighth of worlds `shell` and `job` alone (`domain/session.md`, 10); a story grants what it needs.
- **The agent world:** its translator offers what its grants give, inspect always, modify and shell where its workspace is writable, with the session world's fields, and decodes through the session world's translator.
- **The host world:** the scripted-process world grants nothing and plays no model; the inline world grants what its start gives, through the agent world's provider.
- **The local world:** grants by its workspace; its fuzzy chats have none, so their completions call no tool.
- **Each counts its sweeps' shapes** in the same words (shapes.md, 4).

## 8. The sweep's reach

- **The noisy sweep asks,** at the gate's seeds, that each answer of section 4 fall in at least one case, beside what it asks already (`domain/session.md`, 10).
- **And that each slot is used both ways:** each slot drawn from the conversation and drawn as its never-issued value in some case, and paths classed `live`, `foreign` and `unissued`, offsets `live` and `unissued`.
- **It prints every class and answer** it counted, nought included, and the shapes (shapes.md, 4).
- **A refusal no weight reaches** at the gate's seeds within the layer's allowance is held, with the figures, for the user: never more seeds, never a check dropped (`temper/docs/design/draft/misbehaving-agents.execution.md`, 4).

## 9. Why

- **Every tool, every field.** A model calls every tool its grants give with any field the schema offers. A translator that decoded fewer fields than the product, or refused one the product takes, drew calls no deployment's model makes and missed the ones it does.
- **Refusals by body first.** The fake picks bodies blindly, so a body that carries one refusing value reaches its refusal through the product's own check, with nothing else to set up. Prior state, timing and capacity come from the world, as the job cells did.
- **A menu command for a stale version,** not a second session on the same checkout: each opener has its own root, and a program's changes are what the fake checkout already scripts.
- **A read-only directory and a drawn payload bound in some worlds,** not in all: the namespace changes with a second directory, and a payload bound below a read's window changes every read, so the rest of the sweep keeps its draws.
- **The world classes, the fake fills.** As for jobs (`skein/docs/design/testing/fakes/drawn-agent.md`, 6), only the world knows what a path or an offset names.

## 10. Later

- **A symbolic link in a world's checkout,** for `linked`, once skein's fake checkout makes one.
- **A path from a failure's text** (`path not found: P`), a slot of its own, if a deployment's shapes show models taking it.
