# look-round-20: To begin's Home, Ask's tabs, the switch's place and the night's shadows

The request: `inputs/request.md`. The render check after `look-round-19` found four misses in two look requests on `origin/look-requests`, landed by `temper/docs/design/draft/look-requests.md`, 3 and 5: `to-begin-two-steps` (`4332a26265`) and `night-mode` (`3c5333485a`). `notes/common.md` holds the rules every task follows.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `begin` | `tasks/begin-sections.md` | | Home draws neither Needs you nor Chats while To begin shows, in both shells; the gallery's `home-needs-under-begin` renamed `home-begin-with-needs`; the table `sections_are_hidden_while_to_begin_shows` | 80 |
| `frame` | `tasks/tabs-and-switch.md` | | Ask temper's three tabs on a phone while To begin shows, the composer over them; the switch at a phone chat bar's right and in the folded rail beside a Split; `temper.css` alone | 75 |
| `shadows` | `tasks/night-shadows.md` | | the night set's twelve `rgba(0, 0, 0, .4)` shadow colours removed, each shadow the day's at night | 30 |

**Critical path:** `begin-sections`, about 80 implementer-minutes. All three start at once; none needs another.

## What was decided

- **Needs you and Chats while To begin shows (task 1):** the client README, 3.5 already said they are left out "even with a need or a chat started from another page"; its *Needs you* list kept an older line, "While To begin shows: the card alone", which this cut removed. The view leaves both out in `main`, so the browser's phone block, which is `needs`', goes with them at every width, and the terminal follows, since both shells read one Home. A need while the card shows is reached through Needs you's own page once the card goes, or through a skip: the request's choice (the user, 2026-11-07). The gallery state that drew them is renamed `home-begin-with-needs`, since its old name says the opposite of what it now holds.
- **Tasks 2 and 3 are one task, `tabs-and-switch`:** both turn on the one phone rule that hides a chat layout's rail and tab bar whole (`temper.css`, about line 1793), so one writer owns it. Stylesheet alone: the markup already draws the three tabs on Ask temper's page while To begin shows (`frame::tabbar`, `home::beginning`) and the switch as the rail's last child.
- **The three tabs are told by their count,** `.temper-tabbar:not(:has(> a:nth-child(4)))`, the request's own idiom (its overlay's three-column rule, landed): while To begin doesn't show, the tab bar holds at least four links. No view style is added for it.
- **The switch on a phone's chat layout** stands at the right of the chat's own bar, since there the rail and its top bar go: the rail draws no box (`display: contents`) and none of its other parts, the switch fixed over the bar's right end, the bar padded for it. In the folded rail beside a Split, it stands centred at the folded rail's foot. The request places it "at the rail's foot, and at the top bar's right on a phone"; these are those places in the two layouts its pages didn't draw at that width (its own `03.html` hid it too, under the folded rail's rule).
- **Shadows (task 4):** the request's overlay sets no shadow's colour but three rings (To begin's card, the selected effort's none, the composer's focus), so every shadow keeps the day's colour at night, the five the render check named and the seven others alike (dialogs, the entry, cards, other menus): the night set loses its twelve `rgba(0, 0, 0, .4)` lines and each token holds its day value. `--gf-shadow-edge` and `--gf-home-edge`, a card's ring in the night's line, stay as `look-round-19` drew them, after the overlay's ring on To begin's card.
- **Tests stay cheap:** no new test function. Task 1's case is a row of the view's existing table (`empty_sections_...`, which becomes `sections_are_hidden_while_to_begin_shows`), and the test that pinned the old rule (`beginning_keeps_only_the_card_and_no_phone_block_or_rows`) is folded into it; the gallery's rows change. The stylesheet tasks take no test: their check is temper/ui's render check after landing. The browser tier keeps its twelve functions; no story is added.
- **Design written here:** the client `README.md`, 3.5 (*One layout for every chat*; *Needs you*'s old line removed; *Ask temper's page*, its composer and tabs on a phone), 5.3 (the night switch's places; the tab bar on a chat's layout) and 11 (the after-landing check's list); `browser.md`, 2.5 (*The project's frame*: the three tabs; *Day and night*: *Shadows keep their colour*, new, and *The switch*'s places), 5 (the after-landing check's list) and 7.2 (the two Home rows); `journeys.md`, 10. A **Not yet** entry per task in `README.status.md` and `browser.status.md`, each deleted by its task; the existing entry for task 1 now names it.

No stored format, protocol or public interface changes.

## Overlaps with drafts beside this one

- **`tui-launchpad`'s `view-shared`** (merged on its trunk) changed `temper-client-view`'s `pages/home.rs`, which `begin-sections` changes: unavoidable, since the request changes what Home draws. The task keeps the names it finds; fab's catch-up merges.
- **`tui-launchpad`'s `host-launchpad`, `joint`, `tui-chat`, `tui-foot` and `tui-launchpad`** edit `temper/docs/design/client/README.status.md`: this cut's entries sit right under its opening paragraph, away from theirs, and `begin-sections` and `tabs-and-switch` delete only their own.
- No other draft beside this one writes `temper.css`, the gallery, `browser.md` or `browser.status.md`. `tabs-and-switch` and `night-shadows` both change `temper.css`, in places far apart (`notes/common.md`).
