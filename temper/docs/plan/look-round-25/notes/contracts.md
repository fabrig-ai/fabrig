# look-round-25: the shapes that cross tasks

The run's binding calls. A task builds its half to these; where one says "names", the spec that owns it chooses the rest inside it.

## The engine's path (`temper-engine-domain-ways`, `path-stops`)

Design: `temper/docs/design/domain/forge.md`, 8.13, as this cut wrote it.

- `Step`, in this order: `Ask`, `Draft`, `Plan`, `Produce`, `ProduceMany`, `Check`, `Review`, `Repair`, `Approve`, `Conflict`, `Resolve`, `Merge`, `CatchUp`, `Prepare`, `Prove`, `Turn`, `Verify`, `Branch`. `Update` is gone: the queue's update is in the conflict's and the merge's cards' words.
- `Stop` gains, after `opens` and `card`: `needed: bool`, a stop made only when needed, drawn outlined (a repair, a conflict, a resolution, a catch-up); `over: Box<[u8]>`, at most `OVER` bytes, the small words over the stop's own, empty for none: a repair's "tests", "review" or "tests, review", a merge into a goal's branch's "into goal". `pub const OVER: usize = 16;` beside `WORDS`. `Stop::same` compares both.
- `Inputs` gains `merge_job: bool`, the project names a merge job (`temper_engine_domain_forge` landing's `merge_job(project)`), after `by`.
- `can(inspect: bool, modify: bool, shell: bool, agents: bool, settings: bool) -> &'static [u8]`: the families' words joined by "; ", then "; never merge"; with none of the four, "read temper's records and the forge; propose settings; never change anything" where `settings` (the charter's tools hold jig's settings family, as Ask temper's do), else "read the facts in its task; never merge".

## The wire (`temper-client-protocol-wire`, `path-stops`)

- `enum PathStep { ask, draft, plan, produce, produce_many, check, review, repair, approve, conflict, resolve, merge, catch_up, prepare, prove, turn, verify, branch }`.
- `record Stop { actor: Actor step: PathStep words: text 64 needed: bool over: text 16 opens: Opens card: Card }`.
- The service maps the engine's fields one to one (`temper-service/src/family/ways.rs`).

## The view `roles` (`roles-view`)

Design: `temper/docs/design/client/README.md`, 4.6 and 5.16; `temper/docs/design/domain/forge.md`, 8.13, *The project's roles*.

- Engine: `ways::View::Roles(Box<[(u32, Card)]>)`, each a charter's number and its role's card: `who` the line of when it runs ("For each chat"), `name` the role's name ("Chat"), `facts` "Model", "Can" and "Asked to", as an agent's stop's card has them: the ways crate exports `pub fn role_card(agent: &Agent, who: &[u8], name: &[u8]) -> Card`, which `agent_card` calls for a stop.
- Wire: `record RolesView { roles: list 16 CharterView }`; `View` gains `roles: RolesView` after `rule`.
- The client reads it as it reads `charter N` (the `View` read in the project's scope, again when the session's `settings` watch names a newer revision); a reply draws each role as `charter N` draws its card.

## The client's drawing (`temper-client-view`)

Design: `temper/docs/design/client/README.md`, 5.16, as this cut wrote it.

- `ways-misses` moves, unchanged in what they draw: `pages::ways::drawing(b: &mut Builder, path: &wire::BranchPath, open: Option<u32>, w: &Words, bindings: &mut Bindings, project: u32)`, today's `ask_page::path`; `pages::ways::legend(b: &mut Builder, w: &Words)`, the legend of today's `ask_page::foot`. Ask temper's page calls both.
- `path-marks`: `paths::Stop<'a> { mark: Mark, words: &'a [u8], needed: bool, over: &'a [u8] }`, `paths::OVER: u32 = 16`; its callers pass `needed: false, over: b""` until `how-drawing` passes the wire's `needed()` and `over()` in `pages::ways::drawing` and `views.rs`'s `path`.
- `how-drawing`: How work runs' path section calls `pages::ways::drawing` and `pages::ways::legend`; `pages::how::lanes`, `Lane` and `Column` go.

## skein's fade (`fade`)

Design: `skein/docs/design/ui.md`, 11.2, and `skein/docs/design/ui-web.md`, 8, as the `fade` designer writes them.

- `skein_ui::scene::FadeStop<'a> { pub offset: u16, pub styles: &'a [Style] }`, `offset` in thousandths of the way from `from` to `to`, 0 to 1000, in rising order.
- `Scene::fade(&mut self, from: Point, to: Point, stops: &[FadeStop<'_>]) -> bool`: paints the last mark's stroke, a line, a polyline or a path, with a linear fade from `from` to `to` in the scene's pixels, each stop coloured by its tokens; refused past the scene's limits, as `hint` and `act` are. A shell that paints no fade draws the mark by its own tokens.
- `path-marks` paints each tone's rise with one: across from `(0, 40)` to `(0, 106)`, down the page from `(offset + 206, 0)` to `(offset + 150, 0)`, three stops at 0, 500 and 1000.

## The forge's CI workflows (`workflows-read`, `ci-card`)

Design: `temper/docs/design/domain/forge.md`, as the `workflows` designer writes it, and 8.13, *CI's card*.

- The forge's repository record gains `workflows: Box<[Box<[u8]>]>`, the names of the repository's CI workflows on its landing branch, in the forge's order, each and their count bounded by the designer's limits.
- `ways::Repository` gains `workflows: Box<[Box<[u8]>]>` after `contexts`; the root fills it from the record (`ci-card`).

## `look.rs` and `words.rs`

- `how-drawing` renames the slots of `HOW_SCROLL` to `HOW_LINE` (tokens 648 to 656) `temper-reserved-648` to `temper-reserved-656` and deletes their constants; `roles-page` takes slots 648 onwards, by its spec, for Roles' names.
- `path-marks` appends its names at the end of `NAMES`; words keys: `path-marks` 5100 to 5119, `roles-page` 5120 to 5149.
