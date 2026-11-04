# Request: the temper drafts say what, not who runs it

From survey/agents-refs, 2026-11-04; the user approved the plan (step 11 of 12). fabrig names nothing of
whoever runs its development. Two execution files and `notes.md` carry a lab's run schedule, run
evidence and a mapping from its pipeline; `dashboards.md` uses that lab's status page as its
example. The lab already keeps its own copy of those lines. fabrig keeps each milestone's content
and acceptance, and the design choices with their reasons. Wording and deletion of documents only.
One or two direct tasks.

## 1. temper/docs/design/draft/reference-case.execution.md, configuring.execution.md
- Keep: what each milestone delivers, its order and dependencies, its acceptance, what is done.
- Remove: who runs a milestone and where: the pipeline's name, trunks, plans as run units ("a plan
  of its own"), sessions, hosts, paths on a machine, the citation of the pipeline's role file,
  "in place of ..." comparisons. A "done" cell says the commit or date.
- Where a paragraph is only the run schedule, delete it.

## 2. temper/docs/design/draft/notes.md
- Delete section 2 (run evidence) and section 3 (the mapping from the pipeline onto jig's and
  temper's terms). Fix every citation of those sections, and of later section numbers if they
  move, across the repository.
- Keep the choices N1-N24, each with a reason that stands without the pipeline: where a choice
  reads "unlike fab" or cites its run evidence, say the reason itself (what breaks, what it
  costs). Keep the N numbers: other documents cite them.

## 3. temper/docs/design/draft/dashboards.md
- Its example (~l.9, 10, 58, 197, 370) is a real status page: replace it with a neutral example
  project's status page, in the draft's own words, its test input and golden described as that
  example's data. Remove the status host's name. Fix ~l.387, which states a build tool is fabrig's.
- `tools/citations/allow.txt`'s dashboards entry stays only if the draft still needs it.

## 4. Two status lines
`temper/docs/design/client/browser.status.md` and `temper/docs/design/playground.status.md` each
hold one name the outward check counts (`tests/boundary/outward.txt`): say it without the name.

## Bounds
- Documents only; no code. `temper/docs/development/` is not touched here.
- `tools/citations/citations.py --all` at 0.
- These files' entries leave `tests/boundary/outward.txt`; the check stays green.
