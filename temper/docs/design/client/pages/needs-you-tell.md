# Tell temper

This document holds tell temper, split from [`needs-you.md`](needs-you.md).

## 1. Tell temper

- **Tell temper** (`../../draft/surfaces.md`, section 9; the whole story's
  `needs.html` and `team/needs.html`, their foot, and `phone-tell.html`): a
  person's words about what's off, filed as a finding
  (`../../domain/findings.md`) in the project in view and sent to its owner
  (`../../domain/areas.md`, section 4): the owner of the area the page they
  were on gives (*Where it goes*), else the project's triager, which reads
  the words and the page and hands it to its area's owner, or keeps it
  (`../../domain/areas.md`, section 5), else the project's owners. It
  answers nothing: a question is Ask temper's.
  - *Where:* a region named "Tell temper" at the foot of a project's Needs
    you, its filter naming the project, after Decided for you, on a wide
    page; Needs you across projects has no project in view and draws none.
    On a phone, Ask temper's page's second act, the link "Tell temper" under
    its composer, to `Tell { project }` at `/projects/<project>/tell`, a
    page of the phone's Ask tab ("Ask" current, `../view-navigation.md`, 1) whose top
    bar holds "‹ Back", a link to Ask temper's page, then "Tell temper",
    muted, small, and the night switch. The terminal's `/tell` is
    `../../draft/surfaces.md`, 11.4's (`../tui/pages/README.md`, 2,
    *`/tell`*).
  - *The page you were on:* `Pages::was_on(&self) -> Option<&tell::About>`,
    the last page the person opened in the session but Needs you, `Tell`,
    Ask temper's page and an Ask temper chat, the pages Tell temper is
    reached from, kept as it was left: `tell::About { address: Address,
    title: Option<Box<[u8]>> }`, `title` the task's title for a page of a
    task, `Task`, `Talk` or `Chat`, and the role's for Team with a role's
    card, `Team { role: Some(task), .. }`, as the domain held it when the
    page was left, else none. Kept as *What the client keeps* is, gone on
    reload or sign-out. It is Tell temper's page only where it is the
    project's, its address naming the project or, for a page of a task, the
    task being the project's; else Tell temper has no page.
  - *Where it goes:* `tell::goes(root: &Domain, project: u32, about:
    Option<&tell::About>) -> Option<tell::Goes>`, which the browser and the
    terminal share: the project's `findings` document
    (`../forge/records.md`), the one Needs you's finding cards read
    (`pages::findings`), given to one pure rule, `tell::route(document:
    &w::Findings, about: Option<&tell::About>) -> tell::Goes`, and none
    until that document is read:

    ```rust
    pub enum Goes {
        Area { name: Box<[u8]>, holder: team::Holder },
        Triager { holder: team::Holder, areas: bool },
        Owners { areas: bool },
    }
    ```

    1. the page names a task, `Task { task }`, `Talk { task, .. }`, `Chat {
       task }` or `Team { role: Some(task), .. }`, and that task holds one of
       the project's areas: `Area`, the first it holds in kind 14's order,
       its name and holder;
    2. else the project's settings name a triager and it stands, the
       document's `holder` being what the `triager` gives (`task N` task N,
       `party N` party N, `role N` role N, `charter N` a task): `Triager`,
       that holder;
    3. else `Owners`, role 0 (`../../domain/areas.md`, section 4).

    `areas` says whether the project has an area to hand it to. So a word
    said on a standing role's page, its card or its talk goes straight to
    the area the role owns, with no agent's run; any other goes to the
    triager, who judges where it goes from the words and the page, as
    `../../draft/standing-roles.md`, section 4 has a role route by
    judgement. Routing by path stays later (its section 13).
  - *Its parts,* in this order: on a wide page the head "Tell temper",
    at its right "feedback becomes a finding", muted; on a phone the
    headline "Something off?" and the lede "Say it in your words. temper
    files it as a finding and sends it to the area's owner, who answers on
    it."; the field, a textarea named "Something off?" on a wide page and
    "What you saw" on a phone, two rows and four; "About PROJECT · the page
    you were on, TITLE" where Tell temper has a page, else "About PROJECT",
    TITLE the page's: a page of a task its kept title, else "Task N"; any
    other page its name as the rail, the tab bar or the page's own heading
    says it ("Team", "How work runs", "Work"); the "Goes to" line
    (*Goes to*, below); "File it", primary, while the field holds words;
    then "Ask temper instead", quiet, on a phone "It's a question: ask
    temper instead", which opens Ask temper on the project with the words
    in its composer, unsent, as the Ask temper line's `Act::AskWith` does
    (`home.md`, 1). The field's words are kept in `Pages` while the region
    or the page shows, as a chat's composer keeps its own, so that a redraw
    keeps them and "File it" reads them.
  - *Goes to,* by `Goes`, NAME a holder as Team names it (`team.md`, 1,
    *An owner's name*), "you" where it is the party `me` names:
    - `Area`: "Goes to NAME, AREA's owner"; the person's, "Goes to you, as
      AREA's owner".
    - `Triager`: "Goes to NAME, PROJECT's triager"; the person's, "Goes to
      you, the triager"; with `areas`, then ", who hands it to its area's
      owner".
    - `Owners`: "Goes to PROJECT's owners"; with `areas`, then ", who hand
      it to its area's owner".
  - *Its act:* "File it" files the words as a finding of the project's,
    the one act and one request of `../root.md`, 1.6, through the engine's filing
    (`../../domain/findings.md`, section 4), its `about` the page Tell temper
    has, else none; filed, the field empties and the toast says "Filed for
    NAME.", NAME as *Goes to* names the holder of the `Tell`'s `goes`, "you"
    the person's ("Filed for you."), `Owners` "PROJECT's owners", and
    "Filed." where it kept none; refused, the words stay and the refusal
    says why under the field. Words past the finding's bound are cut
    nowhere: "File it" is disabled while they are past it, saying how far.
  - *The domain's act,* which "File it" sends and the terminal's `/tell`
    sends (`../tui/pages/README.md`, 2, *`/tell`*): `pages::Act::Tell {
    project: u32, words: Box<[u8]>, about: Option<tell::About> }`, its
    words trimmed of white space at both ends, `about` the page they were
    on: the browser's `was_on` where it is the project's, the terminal's its
    screen. Empty, it does nothing; past 4,096 bytes, the finding's
    body's bound, it sends nothing and keeps `tell::State::Long { over }`,
    the bytes past it; else one `FileFinding` of the project
    (`../forge/records.md`), sent through the forge's family as a finding
    card's acts send theirs: kind `feedback`, severity `medium`, its area
    the `Area`'s name that `tell::goes` gives at the act, else none, so
    that the triager holds it, or the project's owners while none stands
    (`../../domain/areas.md`, section 4); its title the words' first line,
    cut at a character's edge to at most 200 bytes; its body the words
    whole, then, where `about` is given and the whole stays within 4,096
    bytes, a blank line and `Page: PATH`, PATH the address as
    `address::print` writes it, then ` (TITLE)` where a title is kept, so
    that the triager and the owner read the page as well as the words:

    ```
    The courier link opens in English for German customers.

    Page: /projects/2/talk/52 (Mailer)
    ```

    One at a time: a `Tell` while one is sent does nothing.
    - *What it keeps:* `Pages::tell(&self) -> Option<&tell::Tell>`, the
      last one's until the next, `tell::Tell { serial: u64, project: u32,
      words: Box<[u8]>, goes: Option<tell::Goes>, state: tell::State }`,
      `serial` counting the session's `Tell`s from 1, so that a shell says
      each outcome once; `goes` what `tell::goes` gave at the act, none
      where the document was not read; `tell::State` one of `Sending`,
      `Filed { number: u32 }` once the row has committed,
      `Refused(w::FindingRefusal)`, `Lost` where the request ended with no
      outcome or *unknown*, and `Long { over: u32 }`.
    - *Its words:* a refusal's are a finding card's, the view's
      `shared::finding_refusal(w: &Words, refusal: &w::FindingRefusal)
      -> Box<[u8]>`, which the card's own words call too.
  - *Shown* on a project's Needs you, on a wide page, and on its `Tell`
    page, once `tell::goes` answers for the project, its `findings`
    document read; nothing stands in its place before
    (`../../draft/whole-story.execution.md`, section 2). The `Tell` page
    holds the project's document as a filtered Needs you does
    (`pages::findings`).

## 2. Open questions

- **Tell temper's area, chosen by the person** (`needs-you.md`, 1, *Tell temper*, *Where it
  goes*; the whole story's `needs.html` and `phone-tell.html`, which say
  where a word goes and offer no other place): a control on the card
  naming another of the project's areas, or the triager, before "File it".
  Default: none; a word filed with the wrong area is handed on by its owner
  (`../../domain/areas.md`, section 5).
- **Tell temper judged before filing** (`../../draft/surfaces.md`, section
  9, "its area found from the page they were on and the words"; the
  mockups' "Goes to" naming an area from words alone): an agent's run
  reading the words as they are typed, to name the area before "File it".
  Default: the page's role alone names an area before filing, and the
  triager judges the words once filed, one short run an item
  (`../../domain/areas.md`, section 6).
- **What became of a word, in the brief** (`../../draft/surfaces.md`, section
  9, "what becomes of it returns in the brief"; `phone-tell.html`'s "It
  comes back in your Home's brief"): the status agent's words naming the
  findings a person filed through Tell temper and what became of them
  (`../../domain/forge/attention.md`, 3). Default: not drawn; the
  card says nothing of it, and the finding's issue carries its owner's
  answer.
