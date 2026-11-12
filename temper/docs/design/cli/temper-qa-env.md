# temper-qa: environment

Generated from temper-qa's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

## Internal

### `TEMPER_INTERNAL_QA_HOME`

- **Set:** internal
- **Reads**
- **Form:** DIR
- **Absent:** Required by every command but the front's.
- The deployment's QA home, holding qa.json and ledger.json.

### `TEMPER_INTERNAL_QA_SESSION`

- **Set:** internal
- **Reads**
- **Form:** N
- **Absent:** Triage commands are refused.
- The running triage session the agent's commands record for.
- **Notes:** Set by the loop for the agent it starts.
