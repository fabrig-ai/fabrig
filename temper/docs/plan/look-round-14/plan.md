# look-round-14: what the render check found in `connect-flows` and page 07

The request: `inputs/request.md`. A look repair after landing (`temper/docs/design/draft/look-requests.md`, 4), from the render check at `e572bf86aa`. The real pages are to look like the requests' pages. These are misses, not new looks.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `look-repair` | `tasks/look-repair.md` | | Every Settings select draws its chevron, and the sheet, the connect pages and Models' add form read as the requests drew them |

Critical path: `look-repair`, about 90 minutes by its estimate.

## What was decided

- **One task, not two.** The request allows two. A stylesheet task and a view task could run side by side, but the account belongs in the landing's last commit (`look-requests.md`, 5), so one would then wait on the other. One writer also keeps the stylesheet and the view's captions together.
- **The chevron's rule is `#app#app .temper-settings select`.** Its doubled id outweighs every `#app >` scope in the stylesheet, and no selector there holds two ids today. It adds no `!important`, as the request asks. A table test holds the rule and that the doubled id stays unique.
- **The other cascade fixes narrow the rule that was too broad, or weight the one that lost:**
  - The kit's radiogroup rule excludes the source chooser.
  - The row's span rule is scoped to the row's description.
  - The no-match row's rule outweighs the list's `> li`.
- **The Forge and Account captions take one new rule,** `p.temper-connect-title:has(+ .temper-settings-text)`, the shape of the label rule the request's pages used. It adds no look token, and Copy public's new caption falls under it.
- **Copy public's caption is drawn in the browser alone.** It's a paragraph directly over the textarea, as Forge's and Account's are, and the terminal already gives the field's name.
- **`rows=4` becomes `min-height: 107px`.** `skein_ui` carries no `rows`, and the repair stays in temper. The box is the same, and the account says so.
- **Models' field name is hidden by the stylesheet,** as `.temper-settings-switch + label` does. It stays the accessible name, and `pages/models.rs` doesn't change.
- **A model way's path shares one `settings::way_path` with the forge's,** whose `elsewhere_path` becomes one call of it. `README.md`, 3.5, *The flows' paths*, already says this, so the code was what lagged.
- **The design moved first, in this cut:**
  - `client/README.md`, 3.5, 4.4 and 5.2;
  - `client/forge.md`, 5.1 (it already said "+ Connect a forge", which `README.md` had lagged);
  - `client/browser.md`, 2.5.
  Their three **Not yet** entries name `look-round-14`, and the task deletes them in its last commit, a stated departure from the implementer's text.

No stored format, protocol or public interface changes. No journey's recording goes stale, since none draws a Settings path, the sheet or Forges' action. No new browser test. temper/ui's render check follows landing.

## Left alone, as the request says

- The report's second table stands: the Models tab in the nav, the paste fallback, step 3's words, no counterfactual default.
- The Settings page's H1 and the project menu's open row are frame differences, named in the account.

## Overlaps

- **`account-email/names-by-email`** (integration) changes `temper-client-view/src/pages/connect_model.rs` and `src/tests/connect_model.rs`. This task must add a function and change one call in the first. It keeps off the second unless the settings table's fixture can't offer ways, an option its spec decides. Whichever merges second takes a small merge.
- **`model-catalogue-3`** has no temper files listed yet, but will add "Read again" and the rest beside Models. This task leaves `pages/models.rs` alone and changes only the add form's and headings' rules. Whichever lands second catches up, as the request says.
