# Findings on Home

This document holds findings, drawn, split from [`home.md`](home.md).

## 1. Findings, drawn

- **Findings, drawn** (`../../draft/home-for-a-person.md`, 3; the mockup
  `../web/home-for-a-person/home.html`, whose head's rules draw these):
  what Home draws of the findings that need the person, so that their
  weight reads before their words. Each part is drawn wherever Home's
  *Needs you* is, on a project's Home and on Home on All projects (`home.md`, 1).
  The TUI's launchpad (`../tui/pages/launchpad.md`) keeps its own lines.
  - *Severity, drawn:* a glyph of three bars, a `temper-severity` span
    with `temper-severity-low`, `temper-severity-medium` or
    `temper-severity-high`, holding three empty spans, 3 px wide and 5,
    8 and 11 px tall, bottom-aligned, gap 1.5 px: one filled for low, two
    for medium, three for high, the rest at .28 opacity, in `--gf-faint`
    for low, `--temper-amber` for medium and `--gf-signal` for high. The
    mockup's `data-severity` is a class here, since a view node holds no
    data attribute. Its name and its hint, so its label and its tooltip,
    are its word: "low severity", "medium severity", "high severity".
    It is the browser's alone: in the terminal it prints nothing, and a
    part that draws it says the severity's word there where its own words
    don't already (*A finding's row*; 1, *More*). One view function draws
    it, `findings::severity(b: &mut Builder, words: &Words, severity:
    w::Severity)`, public in `temper-client-view-work`'s
    `pages::findings`, for every part here and for the triager's groups
    after them (`../../draft/home-for-a-person.md`, 5).
  - *A chip of findings,* `temper-chip` with `temper-chip-low`,
    `temper-chip-medium` or `temper-chip-high`, takes its highest
    severity's ground: low `--gf-hover`; medium `--temper-amber-pale`, its
    ink `--temper-amber`, its border that amber at 35%; high
    `--gf-signal-pale`, its ink `--gf-signal-ink`, its border
    `--gf-signal-line`; medium and high in weight 500. It holds the glyph,
    then its words. `findings::chip(b: &mut Builder, words: &Words,
    severity: w::Severity, label: &[u8])`, beside `severity`.
  - *Counted,* `pages::findings::counted(store: &findings::Store, open:
    &[findings::Need<'_>]) -> findings::Counted`, `Counted { high: u32,
    medium: u32, low: u32, only: bool }`, of the domain's: over the open
    cards, how many findings of each severity they concern, each finding
    once by its `findings::Id` however many cards name it, and `only`
    where every open card concerns findings and one does at least. A
    finding's own card concerns its finding; a group's card
    (`needs-you.md`, 1, *A group of findings*) concerns its rows' findings;
    no other card concerns any. `Counted::total(self) -> u32` is the
    findings counted, `high + medium + low`. A card's own severity,
    `findings::Need::severity(self, store: &findings::Store) ->
    Option<w::Severity>`, is its finding's, its group's highest, and none
    for any other card; the cards' order reads it (`needs-you.md`, 1,
    *What the client keeps*). The needs sentence's "findings" (`home.md`, 1, *Home
    on any project*) is `only`, and where it holds the sentence and
    Needs you's head count the findings, `total`, not the cards: four
    cards, three groups of 3, 3 and 4 and a finding of the person's, say
    "11 findings need you." and "Needs you · 11".
  - *The head's counts,* `temper-home-sevs`, in Needs you's head after
    its heading and before "All needs" at the right, 13 px, muted, gap
    14 px between counts and 6 px between a count's glyph and words: a
    count for each severity with one finding at least, high first, its
    glyph then "2 high", "4 medium", "5 low"; none without a finding. On a
    phone the counts wrap under the heading. In the terminal they print
    after the heading, "2 high · 4 medium · 5 low".
  - *A finding's row,* in `temper-finding-rows`, a list with a rule above
    it, a row an item with a rule under it, padding 9 px 0, gap 12 px,
    14.5 px: its glyph; its title, a link opening the finding, the act
    the Home card's "Open" sent (`findings::Act::Open`); its kind, only
    where the rows' findings are of more than one kind; and its age, its
    span since it was filed, at the right in `temper-finding-when`, 12 px
    in the mono face, faint: "qa · 1 d", or "1 d". In the terminal:
    "TITLE · high · qa · 1 d". `findings::row(b, root, card, words,
    bindings, times, kind: bool)`, public beside `severity`, draws
    one item, in a list its caller opens, so that a group's card draws its
    rows with it (`../../draft/home-for-a-person.md`, 5). A group's row
    is drawn from its `w::FindingRow` with the same parts, its title text
    and no link (*A group's card*, below); one function or two, the
    view's choice, so long as both forms share these parts.
  - *A finding's own card,* a finding the person holds (`needs-you.md`,
    1, *A finding that is the person's*), a `temper-need-card` on Home in
    its place among the cards:
    - *its from line,* mono, `temper-need-from`: on Home on All projects
      the project's chip; then its chip of findings, its kind's word,
      "review"; then "in AREA", or "no area" where it has none;
    - *its row,* the one finding's, its kind left to the chip;
    - *its context,* in `temper-ask-context`, as Needs you's card says
      them: the last hand-off's why, quoted, with who handed it; a
      refusal; its waiting words while an act is out;
    - *its reason line,* `needs-you.md`, 1, *Why it reached you*'s for a
      finding, unless said once (`home.md`, 1, *One why*);
    - *its acts,* as on Needs you, quiet, in `temper-need-acts`: "Hand
      on", with its area's menu and its line of why; "Route to a goal",
      with the project's live goals' menu; "Backlog"; and "Drop with
      reason", each opening its form in place as there. The card's state
      is the domain's one `findings::Card`, so a form opened on Home is
      open on Needs you, and an act sent from either answers on both.
    - *On a phone,* its from line, its row, then "More", quiet, which
      opens Needs you at its card, as another card's "More" does.
  - *A group's card,* a question about a group of the triager's findings
    (`needs-you.md`, 1, *A group of findings*;
    `../../draft/home-for-a-person.md`, 5), a `temper-need-card` on Home
    in its place among the cards, as `home.html`'s first three draw it:
    - *its from line,* mono, `temper-need-from`: on Home on All projects
      the project's chip; then its chip of findings, "3 bugs", "3 CI
      faults", on its highest severity's ground, its count its rows' in
      digits and its noun, or "3 findings", "1 finding" with none; then
      its phrase, " · " and "the oldest 1 d", its oldest row's span (`../view.md`, 6),
      or its one row's span alone: "in temper · the oldest 1 d", or "the
      oldest 23 h" with no phrase; " · later" on a card marked later;
    - *its question,* in `temper-ask-question`, as a question's card has
      it;
    - *its rows,* in `temper-finding-rows`, a row a finding in its line's
      order (*A finding's row*), each kind said only where the rows hold
      more than one ("qa · 1 d" beside "review · 16 h"; "1 d" where all
      are flakes); a row's title is text, no link (`needs-you.md`, 1, *A
      group of findings*);
    - *its context,* in `temper-ask-context` and `temper-group-context`,
      14 px, muted, no margin, where it has one;
    - *its reason line,* a question's (`needs-you.md`, 1, *Why it reached
      you*), unless said once (`home.md`, 1, *One why*);
    - *its acts,* a question's: its choices, the recommended one first and
      primary with "recommended", the others quiet, then "Tell it something
      else", the link that opens *Needs you* at its card; a choice answers
      in place, with its held send and its Undo.
    - *On a phone,* as another question's card: its from line, its
      question, its recommended act full width, then "More"; its rows,
      context and reason line are left to *Needs you*.
    - *In the terminal,* its parts print as words: the from line "3 bugs ·
      in temper · the oldest 1 d", its question, a row a line "TITLE · high
      · qa · 1 d", its context, then its choices and "Tell it something
      else" as a question's.
    - *More:* as the fourth of four open cards, *More*'s line draws its
      glyph, its highest severity's, then its from line's words, "3 bugs
      in temper · the oldest 1 d"; in the terminal "One more: 3 bugs · high
      · in temper · the oldest 1 d".

## 2. Open questions

None.
