# ui-pgup: the plan

PgUp must scroll a chat while its agent writes and after its turn ends, with the focus in the composer, and the view must not snap back. The request asks for a reproduction before any change, so the first task changes no product code.

## Parts and tasks so far

- **`reproduce`:** task `reproduce`. A driver in this plan's `driver/` runs each step of the request on a fake box at 1440 and 420, on the code before any fix, and `reproduced.md` reports each step's offsets, holds or fails, and causes.

The fix tasks and the proof follow in the next batch.

## Critical path, as it stands

`reproduce`, then the fixes, then the proof on the same driver.

## Decided

- **The driver is tracked in the plan's directory,** as `temper/docs/plan/bundle-loading/driver/` is, so that the proof repeats the reproduction's steps and the two reports compare. It leaves with the plan's directory. It is no tier and no test.
- **No browser story,** as the request says: the driver's report is the proof.
