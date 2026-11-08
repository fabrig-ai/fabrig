+++
id = "help"
part = "help"
kind = "docs"
needs = []
files = ["temper/docs/help/forge.md", "temper/docs/help/account.md", "temper/docs/help/repository.md", "temper/docs/help/settings.md", "temper/docs/help/deployment.md", "temper/docs/help/project.md", "temper/docs/help/chat.md"]
effort = "medium"
minutes = 50
+++

# The help pages say the rail, Work and Deployment

## Goal

The help pages that name the rail's entries or Settings' tabs say them as the
design now has them: Work in the rail, with Goals and Changes as its filters and
Chats reached from it; "How work runs" in the rail for the project's process and
its Repositories tab; "Deployment" at the rail's foot for Accounts, Forges,
Models, Workers, Members and Limits; "+ New draft or change". The code lands in
this plan's other tasks; the words are the design's, already on the trunk.

Departure from the implementer's text, and why: **this task owns `docs/help/`**
pages listed in its files, since the request puts help pages that a change makes
stale in the plan, and no code task writes help.

Acceptance:

1. Every place in the files below that names a rail entry, the switcher's
   "Settings for PROJECT", a Settings tab or "+ New chat" in the rail says the new
   word and the new way there, as `temper/docs/design/client/README.md`, 3.5 (*Work*,
   *Settings and Deployment*) and 5.3, have them. A page's other facts stay.
2. Kind 7's switches Goals and Changes are said as what they now do: they hide
   Work's Goals and Changes filters, and hide no rail entry (README.md, 3.5,
   *Work*, *Its filters*; 5.3).
3. No page grows past its bound; a page the change grows past it is shortened,
   losing no fact.
4. The help tests pass: `heavy cargo nextest run -p temper-docs`, and
   `heavy cargo nextest run -p temper-client-tools --lib -E 'test(/help/)'`; check
   each selected tests.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *Work* (its first paragraph and *Its
  filters*), *Settings and Deployment* (*Its head* and *Its tabs*), and 5.3, *The
  rail*, *Projects in the rail* and the phone's tab bar.
- The places found by the lead's survey, each a lead, not a list to stop at:
  `forge.md` around lines 65 and 88 ("Settings on the rail", "In Settings:
  Forges"); `account.md` 69-70 ("In Settings: Accounts"); `repository.md` 65 ("In
  Settings: Repositories"); `settings.md` 15-16 (Goals and Changes in the rail), 134
  ("On Settings, Models"), 143-146 (Ask temper in the rail: still true);
  `project.md` 25 (Goals, Changes and Away shown) and 39-43 (the switcher);
  `chat.md` 129-130 (still true: check); `deployment.md` 84 and 86 ("Settings,
  Models"). Search the help tree for "Settings", "Goals", "Changes", "Chats",
  "Builders" and "New chat" for any other.

## Owns and may change

Owns: the help pages in `files`.

Reader and neighbour files: another `temper/docs/help/*.md` page the search finds
naming the old navigation, the same kind of edit, named in the report.

Not this task's: `temper/docs/help/reference/*`, which is generated from code
(`settings-fields.md`'s lines on Goals, Changes and Away come from the setting's
own description; leave them). Any code, test or design file.

Pre-approved options:

- **If a page explains a path by clicks** ("Settings, then Forges"), say the new
  path ("Deployment, at the rail's foot, then Forges"), on a phone the switcher's
  "Deployment" (README.md, 5.3).
- **If a help test asserts a phrase you change:** stop and say so; that test is
  code, not this task's.

## Steps

1. Read the design sections in Inputs.
2. Find each place (Inputs), and rewrite each in the page's own manner, short.
3. Run acceptance 4's tests; read a failure's log before anything else.
4. Commit and push: the message says that the help follows the rail, Work and
   Deployment.

## Report

Follow the implementer's text. Also list each page and line changed, and each
place you left with its reason.
