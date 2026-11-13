# Talking with a role

## 1. Talking with a role

Illustrative (`tui/whole-story/role.html`, at 80 by 24, the role's
choices under the box):

```
╭─ temper · vsr-rs · talking with Porter ───────────────────── 127.0.0.1:8367 ─╮
│ Porter · a standing role · owns vsr-rs · this week 49 of 60                  │
│ Its Now, 06:58: two landings behind; porting the witness catch-up check, wa… │
╰──────────────────────────────────────────────────────────────────────────────╯

❯ Why two behind? I thought a port took twenty minutes.

✦ It does. The second waits on purpose: vsr's handoff fix touches the same
  three functions, and porting both together saves a port and a check.

❯ Fine. From now on, port likely() and unlikely() as plain conditions.

✦ That's a rule for every port, so it goes in my rules, not my Now. Add it
  to my rules?

  1. Yes, add it — it applies from my next run
  2. Just for this port
  3. Tell it something else
────────────────────────────────────────────────────────────────────────────────
❯
────────────────────────────────────────────────────────────────────────────────
❯ 1. Yes, add it  it applies from my next run
  2. Just for this port
  3. Tell it something else
```

- **What it is:** the root at `Talk { project, task }`
  (`../../pages/talk.md`): the role's chat page in its talk,
  `pages().chat()` with `Chat::talk()`, drawn as a chat is (`chat.md`,
  section 1) but for what this section says (`../../../draft/surfaces.md`,
  11.3).
- **Opening one:** `/talk` on the launchpad (`README.md`, section 2,
  *`/talk`*): the root goes to the role's `Talk`, followed as a chat
  chosen from `/chats` is (`../README.md`, 3, *Its pages*). The talk itself
  opens with the person's first words (jig's `domain/tasks/standing.md`,
  5.6); one of theirs still open, from a terminal closed without leaving,
  goes on.
- **The head,** the region `tui-top`, framed as the launchpad's header
  (`launchpad.md`, 3): in its border "temper" (`tui-title`), " · "
  (`tui-faint`), the project's name (`tui-name`), " · " and "talking with
  NAME" (`tui-hint`, NAME `tui-name`), NAME the role's (`Talking::name`),
  or "role TN" before its task is read; and at the border's end
  `HOST:PORT` (`tui-address`), left out first where the border lacks the
  room. Inside it, two rows, each cut at a word with "…" to the frame's
  width:
  - *its line:* NAME (`tui-name`), then in `tui-hint` " · a standing
    role"; " · owns A, B", the areas it holds (`team::areas`;
    `../../pages/talk.md`, 3), in the areas' order, joined by ", ", while it
    holds any; " · PROJECT's triager" while it is the project's triager;
    and " · this week S of B" (`Talking::spend`, as Team's row says it,
    unscaled), or " · this period S of B" where the project's period is no
    week (`team::weekly`), while its budget is read. A part not read yet is
    left out.
  - *its Now,* dim: "Its Now, HH:MM: " and its Now's first line
    (`tui-hint`), HH:MM the note's `changed` as Team's kicker says the time
    (`../../pages/team.md`, 1, *A role's card*); left out while none is
    read. Read again as the role writes it, so a Now written in the talk
    shows there.
- **The conversation,** as a chat's (`chat.md`, 1, *The conversation*), of
  the person's open talk alone: the thread's lines after its opening
  (`Talking::from`), or every line read while the talk is open and its
  opening is not among them; while no talk of theirs is open, none of the
  role's earlier activity, only words being sent. Under the head's frame,
  a blank row, as on the launchpad. The rule above the box has no title,
  as the launchpad's: the head names the talk.
- **The role's choices,** as a design session's (`design-session.md`,
  section 1, *A reply's choices*), on every talk: a reply ending in a
  numbered list whose last item is "Tell it something else" stands as a
  prompt under the box, its first choice active, taken by Enter or its
  number, set aside by its last choice, Esc or other words. So the role's
  "Add it to my rules?", asked as charter 13 has it
  (`../../../domain/agent/standing.md`, section 7).
- **A proposal in place** (`../../../draft/surfaces.md`, 11.3: "a proposal an
  owner accepts in place"): while a card of the talk (`Chat::cards`) is a
  proposal the person may accept, its document offering them
  `ActKind::Accept`, the box is empty, no other list stands and the person
  has not set it aside, a prompt under the box, as the chat's question
  stands (`chat.md`, section 2): its head the card's line
  (`shared::proposal_line`), then "1. Accept it" and "2. Not now".
  - *Accept it:* the root's `Act::Decide { object, revision, act:
    Accept }` then `Act::Confirm` for the confirmation it raises, two acts
    as `/goal`'s way chosen sends (`design-session.md`, section 1), so that
    the role takes the rule at its next run (`../../pages/talk.md`, 1, *A
    proposal accepted in place*); a refusal is drawn under the box in the
    view's words, and the prompt goes once the card leaves the talk.
  - *Not now,* Esc or words typed: `Tui` keeps the card's object as set
    aside, and the prompt stands again only for another card; the card
    stays a card of the conversation (`chat.md`, 1, *a card*) and a need
    in Needs you.
  - The role's choices win over it while they stand, and it over nothing
    else: a list a command opens stands in its place.
- **The box,** "Message": its placeholder, while the person's talk is
  open, "Reply, or Esc to leave. NAME writes its Now when you go."; while
  none is, "Talk to NAME, or Esc to go back.". Enter sends its words as
  the talk's (`../../pages/talk.md`, 1, *Its words are talk*); a refusal,
  such as "talking with PARTY" while another person talks, is drawn under
  the box in the view's words, as a chat's send's.
- **Leaving:** Esc on the empty box while no list stands, or `/exit` sent
  whole, goes to the launchpad, `Went(Home { project })`, the talk's
  project, which leaves the talk (`../../pages/talk.md`, 1, *Leaving the
  page leaves the talk*); with a talk of the person's open, the notice
  (`README.md`, section 1, *The notice*) says "Left NAME after 9 min ·
  0.60. It writes its Now.", its length from the talk's `opened` to the
  leaving, in whole minutes, at least 1, and its cost (`Talking::cost`) as
  the quiet line says it, ", COST" left out while unknown; with none open,
  no notice. `/new`, or a chat chosen from `/chats`, leaves the talk too,
  with no notice. `/quit` and Ctrl+C close the TUI, and the talk lapses
  (`../../pages/talk.md`, 4).
- **The quiet line** (`README.md`, section 1, *The quiet line*):
  - *at its left,* the project's name, " · ", NAME (`tui-hint`), then the
    link's state only while it is not Live, as a design session's;
  - *at its right,* while the person's talk is open, "this session 9 min"
    and, while its cost is known, " · 0.60" (`tui-hint`), its length as
    leaving words it and its cost as Team's sessions say a cost
    (`../../pages/team.md`, 1, *Its sessions*); then the project's count of
    what needs the person, as a chat's.

  Illustrative: ` vsr-rs · Porter          this session 9 min · 0.60 · 2 need you`.
- **Its commands** (`README.md`, section 2, *Offered*): `/new`, `/chats`,
  `/open`, `/tell`, `/theme`, `/help` and `/quit`, and `/exit` whole to
  leave; none of a chat's own. `/open` opens the role's card on Team,
  `Team { project, role: Some(TASK) }` (`../../pages/team.md`, 1), where its
  sessions, earlier talks among them, are read.
- **Where it departs from its mockup** (`tui/whole-story/role.html`;
  `../../../draft/surfaces.md`, 13), each for a reason:
  - the head's line and its Now stand inside the frame, as the
    launchpad's brief does, where the mockup draws an empty frame and the
    two lines under it: the frame is the screen's one top region, and the
    head stays while the conversation scrolls;
  - "this period S of B" where the project's period is no week, the
    mockup's "this week" being a weekly period's;
  - the quiet line's left names no model and effort: a role's runs take
    its charter's models (`../../../domain/agent/standing.md`, section 2),
    which no picker of the person's changes;
  - the role asks "Add it to my rules?", its own words, where the mockup
    has "Add it to Porter's rules?"; and its choices stand under the box
    as well as in its reply, as a design session's do;
  - "Yes, add it" is the role's proposal, and accepting it in place is
    the proposal's own prompt after it, where the mockup's one answer adds
    the rule: an owner decides a proposal of settings, which the role
    never accepts (`../../../domain/agent/standing.md`, section 5).
