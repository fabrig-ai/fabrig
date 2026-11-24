# smith: environment

## Public

### `SMITH_CHECK_RESULTS`

- **Set:** public
- **Sets:** for a directory's checks, when its start names a results directory.
- **Form:** a directory path on the agent's file system, at most 4,096 bytes.
- **Absent:** unset when the start names none, even if configured in the command environment.
- The run creates the directory if missing before executing the check. The host
  keeps it as long as the workspace tree; smith never reads its contents.

See `smith/docs/design/domain/run-delivery.md`, section 1.1.
