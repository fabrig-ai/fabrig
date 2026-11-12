# Design content review

Report, 2026-11-11. Read the root index, each directory README's preamble
and navigation, all 19 non-draft Open questions/Later sections, and the
connected-agent later section (`smith/docs/design/protocol/host.md`, 4).
Searched the non-draft design for repeated headings and bold terms,
matching passages and an answer to each deferred item; read candidate
sections rather than whole documents. Cross-tree answer searches covered
skein, jig and hearth. Drafts and status entries were excluded; temper
was not reviewed.

The design is unchanged: `smith/docs/design/**` is a test-read input.
Even a consolidation below is left to its designer with the tests that
read it. Nothing here decides an outstanding product or protocol choice
(`DESIGN.md`, 1).

## 1. Counts

Left: 7 findings: 1 partly answered question, 5 duplicated-question groups,
1 misplaced question. No confirmed contract overlap after excluding cited
summaries and the same subject's different layers. No moves done.

## 2. Findings left

### M1. Partly answered: Claude Code's headless setup

`smith/docs/design/benchmarks/README.md`, 14 still asks what permits
headless edits/commands and whether `CLAUDE_CONFIG_DIR` excludes the
person's files. `smith/docs/design/protocol/programs.md`, 4.2 and 5.1 now
specify a source-private directory, the filtered environment,
`--setting-sources user`, the command's tools and permission mode.
**Owner:** `smith/docs/design/benchmarks/README.md`, 14.
**Suggest:** replace the settled setup questions with citations; retain
only benchmark-specific confirmation still needed, including its named
model/effort on a subscription. An external program's future behaviour is
not established just by the design's present launch contract.

### M2. Duplicated: trace rotation and retention

`smith/docs/design/protocol/events.md`, 10 and
`smith/docs/design/shell/README.md`, 7 independently defer rotation and
retention. `smith/docs/design/protocol/agent.md`, 10 already points the
reader to events for those questions. **Owner:**
`smith/docs/design/protocol/events.md`, 10.
**Suggest:** keep one lifecycle question there; leave the shell its
separate product question of a default trace location, citing the common
rotation/retention question. The shell item mixes the two, so it cannot
simply be moved whole.

### M3. Duplicated: an environment per run

`smith/docs/design/domain/tools.md`, 8 and
`smith/docs/design/protocol/agent.md`, 10 both defer a typed bounded charter
overlay, merged once and admitted at the run entrance when a host needs it.
**Owner:** `smith/docs/design/domain/run.md`, 2, the charter's semantic
contract. **Suggest:** give the domain that one question, with tools and
agent protocol citing it and retaining only their own execution/encoding
consequences. No new charter field or precedence is chosen here.

### M4. Duplicated: conventions' home

`smith/docs/design/domain/run.md`, 13 and
`smith/docs/design/protocol/charter.md`, 10 both spell the choice between
the charter and agent configuration with a charter override. The latter
cites the former but still states the entire choice.
**Owner:** `smith/docs/design/domain/run.md`, 13.
**Suggest:** keep that question and make charter's item a reference only.
This is a possible unnumbered consolidation, left because smith's text is
read by tests.

### M5. Duplicated: producing streamed reasoning

`smith/docs/design/protocol/channel.md`, 12 and
`smith/docs/design/protocol/llm.md`, 12 both ask whether to request readable
reasoning from providers and produce it. The channel already carries its
kind; the undecided producer is the LLM component, not a second wire design.
**Owner:** `smith/docs/design/protocol/llm.md`, 12.
**Suggest:** one producer question there and a channel reference to it.
The corresponding host transport concern in jig is not a second design
of the provider's API.

### M6. Misplaced: failed-call usage's supposed upstream question

`smith/docs/design/benchmarks/README.md`, 14 says whether failed calls
carry usage is skein's LLM document's question. That document defines
usage and failures (`skein/docs/design/llm/README.md`, 2.3 and 2.5), but
its Open questions section (`skein/docs/design/llm/README.md`, 7) has no
such item. **Owner:** `skein/docs/design/llm/README.md`, 2.5.
**Suggest:** have the LLM designer own the proposed failure-vocabulary
extension, with the benchmark retaining its null-reporting consequence.
Moving the benchmark's entire wording upstream would carry downstream
result-schema policy into skein, so this is listed, not moved.

### M7. Duplicated: calls beyond a completion's count bound

`smith/docs/design/domain/session.md`, 11 and
`smith/docs/design/protocol/limits.md`, 13 ask whether excess calls should
be returned as not-run outcomes rather than fail the completion, as does
`skein/docs/design/llm/README.md`, 7.
**Owner:** `skein/docs/design/llm/README.md`, 7, for the producer's outcome
contract. **Suggest:** settle that contract once, then have smith cite it
for its session policy and sizing consequences. The texts use different
owners' bounds and outcomes; combining them or changing either protocol
is not a pure move.

## 3. Moves done

None. No smith design or status text, code, fixture or generated output
was edited, and no test-read input was changed.

## 4. Coverage limits

README bodies outside the root index, navigation and deferred/candidate
sections were only searched. The other bodies were read only around a
mechanically found candidate or answer. Repeated credential source names,
profile-value tables that cite their owner, domain/protocol views of the
host and charter, and the shell's composition of the inline agent were
not counted merely for sharing terminology. Genuine alternatives awaiting
measurements, external-provider evidence or a host's first need stay open.
This review does not establish implementation state or benchmark results.
