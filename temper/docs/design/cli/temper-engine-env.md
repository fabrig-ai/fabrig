# temper-engine: environment

Generated from temper-engine's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_CHECKPOINT`

- **Set:** public
- **Sets**
- **Form:** DIRECTORY
- **Absent:** Set for every held command; an inherited value is replaced.
- The held checkpoint's absolute directory, set for checkpoint --hold's child.
- **Notes:** The checkpoint lock stays held until the child is closed.

## Internal
