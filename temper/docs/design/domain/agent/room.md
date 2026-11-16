# A charter's room

2026-11-16. A run's first request carries its charter's text, its brief and
its host tools' declarations, and two bounds hold them: smith's host part,
in rendered bytes (1), and the charter allowance the engine fits a charter
to, in decoded ownership and in encoded bytes (2). This document says how
the engine fits a brief to what the rest leaves, what a refusal names when
the rest alone doesn't fit (3), and how a charter is checked before a run
meets it (4). The tools themselves are `tools.md`'s; this is written from
`../../draft/tools-by-role.md`, 6.

## 1. The host part's budget

smith refuses a run whose host part (its host tools, instructions, brief,
and finish and deliver schemas) passes its bound, the room of the agent's
smallest window (`charter.md`, 5), before any turn. The largest stock
charter's non-brief host part is charter 1's with a Change contract and
the control family's plan form of `change` (`tools.md`, 1); with the
authorised brief, its task section a whole spec
(`../../protocol/configuration/limits.md`), and 1,472 bytes for 64 stock
sections' titles and framing (16 title bytes and 7 framing bytes each), it
leaves a margin at `temper-box init`'s models
(`../../protocol/configuration/README.md`, section 9); `temper-service`'s
step tests hold each stock charter's figures. A gathered brief spends what
the non-brief host part leaves. So a new host tool is offered only where a
family and the charter gate it, never to every run, and a charter's room
is measured before a tool joins it.

## 2. The allowance and the brief's fit

smith counts a run's brief in its host part, beside the charter's offered
tools, its instructions and the finish and deliver schemas. Before sending
an assignment, the engine fits gathered sections to its charter bound for
both the encoded charter, including framing, and its decoded ownership.

- **The allowance.** The bound is the smaller of the configured
  `hub.charter_bytes` and the charter allowance of the worker that
  receives it (`temper-engine-service-vocabulary`'s `charter_bound`), so
  the engine never sends a run a charter its worker would refuse: a worker
  refuses at decoding a charter whose ownership passes its smith
  `run_bytes`, before its own fit. Where the process has a local agent,
  that allowance is the agent's `run_bytes`, as a task read's room is its
  `host_reply_text` (`../routing.md`, 2). Where it doesn't know its
  worker's, it takes smith's at its floor, 65,536, which the stock agent's
  `tool_payload` of 10,662 does not raise (smith's `protocol/limits.md`,
  3.10): the smaller of the two defaults, and how the engine learns a
  remote worker's is section 5's. The cap stays 65,536 (the user,
  2026-11-16): a reasonable size for what a run is sent at once.
- **What stays whole.** Instructions, tools, contracts, models and run
  identity stay whole.
- **The worker fits again.** The worker service also fits the brief after
  the workspace is prepared and before encoding the start, using the
  receiving run's limits and measured schema costs. It fits both rendered
  host room and the decoded charter ownership left after smith charges the
  prepared workspace against `run_bytes`, including resolved host paths and
  conflict paths. Smith supplies that remaining allowance through
  `charter_room`; temper keeps the incoming charter allowance too.
- **One fitter.** `temper-protocol-brief` owns the fitter, which the
  engine's translation, `temper-engine-smith`, and the worker service each
  link; the translation's typed start takes explicit receiving limits and
  fits the brief too.
- **How a brief is cut.** A brief already within each entrance's room is
  kept whole. Otherwise, fitting cuts bodies from the last section in the
  core's order, at UTF-8 boundaries, keeping each cut section's title and
  ending its body with its own line: `[cut N bytes; whole: READ]`. `N`
  counts original body bytes removed, excluding the added separator and
  cut line. `READ` is `read_job/read_checks` for `CI`,
  `read_pull/read_pull_files` for `Pull request`, `read_reviews/read_remarks`
  for `Reviews`, `read_docs` for `Home`, `recall` for `Notes`, `the task's
  plan` for `Plan`, and `the task's activity` for the other core sections;
  other titles use `read_docs or the task's activity`.
  - When a title and its minimum cut line cannot fit, whole trailing
    sections are omitted. One line records them, in their original order:
    `[N sections omitted: TITLE, TITLE; whole: the task's activity]`. It
    follows the last retained body, or stands in an untitled section if
    none remains, only when it fits without cutting more retained text. If
    even that line cannot fit, it is left out, down to an empty brief at
    zero room.
  - Fitting also keeps the charter within its admitted ownership
    allowance. No run is refused for its brief's size.
- **The rest alone past the bound.** If the non-brief host part or fixed
  declarations plus workspace alone exceed their receiving bound, smith's
  declaration refusal still applies. Where the charter without its brief
  passes the allowance at the engine, the start is refused there, naming
  what didn't fit (3).

## 3. A refusal names what didn't fit

An assignment the engine can't encode is refused before it is sent, and
the refusal says what didn't fit, never a bare "assignment cannot encode".

- **One typed reason** for jig's encoding and temper's: jig's `Unencoded`
  (jig's `protocol/channel/records.md`, 2), the field that didn't fit, the
  codec's reason where a codec refused it, its figure, the configured
  bound and the wire's ceiling, each where it is known. temper's fitting
  returns the same record, since temper's translation already names jig's
  engine-workers vocabulary for the event it builds; no temper type of its
  own carries it.
- **temper's fitting's fields.** `fitted_charter` and `assignment::down`
  (`temper-engine-service-vocabulary`'s `assignment.rs`) return
  `Result<_, Unencoded>`:

  | field | when | figure | bound | ceiling |
  |---|---|---|---|---|
  | `charter.fixed_owned` | the charter without its brief, in decoded ownership, passes the allowance (2) | its ownership | the allowance | jig's `assignment_charter` ceiling |
  | `charter.fixed_encoded` | the same, encoded | its encoded bytes | the allowance | the same |
  | `charter.owned`, `charter.encoded` | the charter with its fitted brief still passes it | its bytes | the allowance | the same |
  | `charter.ownership` | ownership's sum passes a `u64` | none | none | none |
  | a smith charter path (`HostTool.input`) | smith's encoding refuses the charter, its codec's reason beside it | none | none | none |
  | a worker wire path | the workspace's items don't encode, its codec's reason beside it | none | none | none |

  The ownership is checked before the encoding, as the fitting checks
  them, so the first that fails is named. Another refusal of smith's
  encoding names its own field in the same form (`charter.endpoints`).
- **Its words,** rendered by `Unencoded::words` in step code, with
  thousands' commas: `assignment cannot encode: charter.fixed_owned 82,884,
  bound 65,536, wire ceiling 67,108,864` (jig's
  `protocol/channel/records.md`, 2 fixes the form).
- **Where they go.**
  - The start's answer: `Workers::assignment`
    (`temper-engine-service-component`'s `workers.rs`) answers the invalid
    start, cause other, with these words as its `why`, as jig's component
    answers its own (jig's `protocol/channel/records.md`, 2).
  - The engine's log: the root keeps a refused start's diagnostic and the
    shell prints it as a run-refusal line (`temper-engine-shell`'s
    `lines.rs`), whole, since the words are far within its bound.
  - The held task: the hub link ends the attempt as a permanent failure
    with these words as its reason, within 128 bytes, cut with a marker
    (jig's `domain/hosts/hub.md`, 3), so a task held out of tries names
    the field and its figures.

## 4. Checked before a run meets it

### 4.1 The entrances

At start, and at temper's three entrances of a change of settings that
touches a charter (the engine's `ChangeSettings` and `ProposeSettings`
routes and a run's `propose_settings`), temper checks each charter it is
given twice: its rendered room (4.2) and its fixed part (4.3).

- **Overlays.** An item setting an overlay is judged on the charter it
  makes over the charter's current default (jig's
  `domain/engine/overlays.md`, 1).
- **Start.** The start's check runs over the defaults, both checks, and
  again once the stored settings are restored, the rendered room alone. It
  judges each overlaid charter as its runs read it, so that a release whose
  new default leaves an overlaid charter too little room stops the start,
  naming the charter.
- **A restored charter's fixed part isn't checked at start.** A stored
  charter is changed only through the running engine's entrances, so a
  start stopped for it would leave no way to change it; its runs are
  refused at their start instead, naming what didn't fit (3).
- **What isn't checked again.** A stored proposal's acceptance, a restore
  and a reset carry values that passed an entrance or the start, and the
  bounds do not move while the engine runs.
- **A refusal.** A party's direct change or proposal is refused as a
  judged charter item, naming its item index; a run's proposal receives a
  host error with the refusal's words and its run continues. On restart, a
  stored current charter that fails the rendered check ends startup with
  `temper-box serve: restart refused: ` and the same diagnostic
  (2026-11-04).

### 4.2 The rendered room

temper sums each charter's offered tools, its instructions with every one
of its parts joined to them, the most any of its runs can select, and the
fixed parts, and refuses one that leaves its brief less than 2,048 bytes,
naming the charter and both figures (`../../protocol/configuration/limits.md`).
The bound is the engine's size profile's, the one its workers are
configured from (51,717 in the small one, for a window of 177,523 tokens;
in `standard`, the room of luna's window, `charter.md`, 5). An engine
without that figure, with no local agent, skips this check. A run smith
refuses for its host part anyway is held with smith's reason, the bound
and the figure, never a bare "invalid".

### 4.3 The fixed part

A charter is refused where its fixed part, on some wake, passes the
allowance (2), in decoded ownership or in encoded bytes: the comparison
`fitted_charter` makes before it fits a brief (3), made at apply.

- **On each wake.** jig joins a charter's text and the parts whose `on`
  meets a run's reasons (jig's `domain/tasks/messages.md`, 6). The check
  judges each of the fourteen reasons alone, `first` to `words` (jig's
  `Reasons`), on the instructions a run woken by it reads: the charter's
  text, then each part whose `on` names it, each joined by a blank line. A
  charter that resumes is judged with its parts too, as a standing session
  reads them. A charter with no parts reads the same on every wake.
- **What it counts,** on that wake, beside those instructions:
  - its main model's name and effort, and each alternative model's name,
    effort and fixed cost;
  - the declarations it is offered: the tools declared whatever its
    families, and those of each family its `tools` names, at the widest
    authority (`tools.md`); an asking charter's own tools and, where it
    names settings, `propose_settings`'s asking form;
  - the frame: the rest of smith's charter, at its largest: the largest
    Change contract and its delivery, the conventions and the reserved
    message's mark, whatever the charter.
- **Without a local agent** the check runs, at the allowance of the
  smaller of `hub.charter_bytes` and 65,536 (2), rather than being
  skipped.
- **Its refusal** names the charter, the wake, the measure and both
  figures, and says what to trim (4.5). Where several wakes pass, it names
  the one whose fixed part is largest in the measure that failed, the
  first in `Reasons`' order among equals; ownership is checked first.
- **A run woken by several reasons at once** joins every part any of them
  meets, which can pass where each alone doesn't; such a run is refused at
  its start, naming the field (3), and section 5 keeps it open.
- **The brief's floor** joins this check with `../../draft/tools-by-role.md`'s
  second milestone: a fixed part that leaves the brief less than its floor
  is refused the same way. Until then a fixed part that fits leaves a run
  whatever brief is left.

### 4.4 The costs, as facts

The declarations live in `temper-engine-smith`, which depends on
`temper-engine-domain`, so the domain can't measure them itself
(`skein/docs/foundation/programming-model.md`, 4). The
service layer, which holds both, measures them at startup with
`temper-engine-smith`'s `room::host_part` and gives the domain one table,
`temper-engine-domain`'s `room::HostPart`: each cost in smith's three
measures, and both bounds.

```rust
/// One cost in smith's three measures.
pub struct Cost {
    /// In smith's rendered host part, against `HostPart::bound`.
    pub rendered: u64,
    /// In the charter's decoded ownership (`Charter::owned_bytes`), against `allowance`.
    pub owned: u64,
    /// In the encoded charter (`encode_charter`'s bytes), against `allowance`.
    pub encoded: u64,
}

pub struct HostPart {
    /// Smith's host part in the receiving profile; none without a local agent.
    pub bound: Option<u64>,
    /// The charter allowance (2).
    pub allowance: u64,
    /// Rendered: smith's own text and the largest finish and deliver schemas.
    /// Owned and encoded: the frame (4.3), its main model's name and effort empty.
    pub fixed: Cost,
    /// One alternative model beyond its name's and effort's bytes.
    pub model: Cost,
    /// Declarations offered whatever the charter's families.
    pub always: Cost,
    /// Each family's own declarations, in `ToolFamilies::JIG`'s bit order,
    /// delegate first and ask last.
    pub families: [Cost; 12],
    /// An asking charter's own declarations, and `propose_settings`'s asking form.
    pub asking: Cost,
    pub asking_settings: Cost,
    /// Smith's families, rendered, paid where the policy turns them on.
    pub inspect: u64,
    pub modify: u64,
    pub shell: u64,
    pub agents: u64,
}
```

- **The costs add.** A charter's fixed part in each measure is `fixed`,
  plus its instructions' bytes on the wake, plus its main model's name and
  effort, plus each alternative model's `model` and its name and effort,
  plus `always` and the cost of each family its `tools` names, or, for an
  asking charter, `asking` and, with settings, `asking_settings`. The
  rendered room's sum keeps its own rules (4.2): every part joined, smith's
  closing blank line and the alternative-model offer's words.
- **Today's offer fills the slots** (`tools.md`): `always` holds every
  host tool declared to a charter that isn't asking whatever its families,
  `message` and `read_task` among them at the widest authority; the
  delegate family's slot holds `delegate`, `goal`, `change` in its plan
  form, `finding`, `red` and `follow`; the settings family's holds
  `settings` and `propose_settings`; the other families' slots are zero.
  When the offer follows the charter's families
  (`../../draft/tools-by-role.md`, 2), the service fills each family's slot
  and `always` keeps `read_docs` alone; the table's shape doesn't change.
- **What the domain can't read is taken at its largest.** A model's
  encoded cost includes its endpoint's name, which the domain doesn't read:
  `fixed` and `model` are measured at the longest name of the encoding's
  endpoint table, so the sum is never under smith's measure.
- **Held equal to smith's measure** by `temper-engine-smith`'s step tests,
  as the rendered table is today: for each stock charter on each wake,
  under a Change contract, the sum equals `owned_bytes` and the encoded
  length of `temper_engine_smith::charter`'s charter without its brief, its
  models on the longest endpoint name.
- **Without a local agent** the service still measures the owned and
  encoded costs, which need no agent's limits; `bound` is none, the
  rendered costs are zero, and the rendered check is skipped (4.2).

### 4.5 The refusals' words

- **The rendered room:** `charter N leaves its brief R bytes of its host
  part's B; it needs at least 2,048`, with thousands' commas in the
  figures and room zero when the non-brief part consumes the bound.
- **The fixed part:** `charter N on a W wake is F bytes M before its brief,
  past its allowance of A; trim its text, parts or tool families`, where
  `W` is the wake as a part's `on` spells it (`first`,
  `question_answered`) and `M` is `owned` or `encoded`, with thousands'
  commas in the figures: `charter 5 on a first wake is 66,012 bytes owned
  before its brief, past its allowance of 65,536; trim its text, parts or
  tool families`.

## 5. Open questions

- **A charter where the process has no local agent.** The engine does not
  know its remote worker's charter allowance, so it fits a charter to the
  smaller of `hub.charter_bytes` and smith's 65,536 (2), and a remote
  worker that takes more, the reference's 127,480 among them, receives its
  brief cut at 65,536. The worker's hello on jig's link, which names its
  reply bound (`tools.md`), could carry the allowance beside it, or the
  engine's document could name it in `hosts.workers`, and the request
  `landing-behind-main` of 2026-11-08 asks it be sized as that bound is.
  Default: the smaller of the two defaults.
- **A run woken by several reasons at once** (4.3). The check judges each
  reason alone; judging every part joined would refuse charters whose runs
  never join them all. Whether jig's wakes ever join parts a single reason
  doesn't, and so whether the check should judge their union, is open.
  Default: each reason alone, and a start past the allowance refused
  naming its field.
