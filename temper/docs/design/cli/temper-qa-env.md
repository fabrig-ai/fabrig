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
- **Absent:** Triage and pass commands are refused.
- The running triage or pass session the agent's commands record for.
- **Notes:** Set by the loop for the agent it starts.

### `TEMPER_INTERNAL_QA_AREA`

- **Set:** internal
- **Sets**
- **Form:** AREA
- **Absent:** Set by the loop from request.area, or owner when absent.
- The area under which the deployment starts a request's goal.
- **Notes:** Given to request.command and request.state; never read by the tool.
