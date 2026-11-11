# Design content review

Report, 2026-11-11. Read the design root index, directory README preambles
and navigation, and every non-draft Open questions and Later section
(including KV's settled choices/later work and the shell's deferred
optimisations). Searched all non-draft design documents for repeated
headings, bold terms, matching passages and answers to each deferred item;
read the candidate sections, not the remaining documents whole. Status
entries and drafts were excluded. Cross-tree answer searches covered smith,
jig and hearth; temper's design was not reviewed here.

Nothing here chooses between differing contracts or removes an answered
question. The owners below settle those changes under `DESIGN.md`, 1.
A cited summary or another layer's application of a contract is not counted
as an overlap.

## 1. Counts

Left: 4 findings: 1 overlap, 1 answered question, 1 duplicated question,
1 misplaced-question group. Done: 3 duplicate-question consolidations by
hand; no numbered section or document moves.

## 2. Findings left

### S1. Overlap: the classic stream contract

`skein/docs/design/runtime/lib.md`, 7 and
`skein/docs/design/runtime/io/sockets.md`, 3 both state the demand's one
answer, withdrawal, End with an outstanding demand, and delimiter-maximum
rules in full. This is more than io's state table applying lib's contract:
several bullets retell the common obligations before adding io's details.
**Owner:** `skein/docs/design/runtime/lib.md`, 7.
**Suggest:** keep the common protocol there and retain in sockets only its
assertions, intake/delivery timing, kernel operations and entity-close
behaviour, with a citation. This needs an editorial merge, not a whole-section
move: sockets' section also owns its state machine and io-specific rules.

### S2. Answered question: conditional reads beyond the log

`skein/docs/design/forge/github.md`, 15 still defers validators beyond the
log, with the default "the log alone". But
`skein/docs/design/forge/connection.md`, 4.6 specifies a bounded validator
cache keyed by request and identity and says every read in both dialects
uses it. **Owner:** `skein/docs/design/forge/connection.md`, 4.6.
**Suggest:** the GitHub designer retire or narrow the question against that
contract; do not leave the two scopes as competing defaults. This item is
left unchanged; the separate DNS-question consolidation below changes no
conformance fact.

### S3. Duplicated question: an update's lifetime

`skein/docs/design/forge/README.md`, 10 and
`skein/docs/design/forge/client.md`, 11 both ask when an update's real git
work can outlast the client's lifetime and what a repeated update costs.
The former supplies the remote-work distinction; the latter adds drift,
CI and a late rebase's risk. **Owner:**
`skein/docs/design/forge/client.md`, 11.
**Suggest:** one measurement question there, with the dialect's facts
remaining in its own document. Combining the nonidentical risk accounts is
an owner's edit, not a relocation of either item unchanged.

### S4. Misplaced questions: downstream adoption in temper

`skein/docs/design/runtime/lib.md`, 14's "temper's side of 4 and 8" and
`skein/docs/design/ui/README.md`, 15's "temper's side of section 13" defer
consumer calendar/token and Limits-literal changes in named temper crates.
These do not leave lib's or the UI kit's contract undecided.
**Owner:** temper's design overview, `temper/docs/design/README.md`, for
routing the consumer work to its per-part designers; exact placement is
left to the sibling temper review. **Suggest:** that review give each
consumer obligation its home and leave skein only a citation if needed.
No temper question or text was moved across the review boundary.

## 3. Moves done

The map is `tools/citations/maps/content-review-skein.map`. These are
unnumbered items, handled by hand rather than `move.py`; each destination
already held the question and the source now cites it. No section's content
was merged or its number changed.

- `skein/docs/design/ui/README.md`, 16, "The text shell's own words" ->
  `skein/docs/design/ui/text.md`, 9, "Its own words in another language".
- `skein/docs/design/ui/README.md`, 16, "A server keeping the previous
  bundle's digest directory" -> `skein/docs/design/ui/web/README.md`, 13,
  "A server keeping the previous digest directory".
- `skein/docs/design/forge/github.md`, 15, "The blob host's address" ->
  `skein/docs/design/forge/connection.md`, 11, "Runtime DNS". GitHub now
  cites the existing component question; its dialect facts and test-read
  conformance material are unchanged.

## 4. Coverage limits

README bodies outside the root index, reading/navigation material,
deferred sections and candidate sections were only searched. Other design
bodies were read only around a candidate or an answer. In particular TLS's
close and the link's close were checked: TLS owns sending the alert, while
the link owns ordering TLS's output before io's close and bounding that
composition (`skein/docs/design/net/tls.md`, 3.1;
`skein/docs/design/net/link.md`, 3). They are not counted as an overlap.
A question with an explicit future alternative or an experiment still
needed was not deemed answered merely because it also states a default.
This report is a search-led inventory, not a line-by-line review of all
contract prose or a claim about what code has implemented.
