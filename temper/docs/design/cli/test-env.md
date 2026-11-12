# temper: test environment

Generated from the tree's test table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Test

### `TEMPER_TEST_BOX_PARENT_TEST`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** Run the parent side of the story.
- Run the box parent-death story's child with this scratch directory; the process test sets it.

### `TEMPER_TEST_BUNDLE`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The browser world refuses to run without a bundle.
- Locate the built browser bundle; the browser tier sets it.
- **Notes:** Relative paths are resolved from temper's tree; the directory must hold index.html.

### `TEMPER_TEST_E4_CAPTURE`

- **Set:** test
- **Reads**
- **Form:** SOCKET
- **Absent:** Start the capture listener and re-execute the story as a child.
- Connect the Forgejo-elsewhere story to its private capture witness; the story's parent sets it.
- **Notes:** An abstract Unix socket name, not credential bytes.

### `TEMPER_TEST_E4_LIFE`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** Run the complete story, including capture and both engine lives.
- Run one engine life in the Forgejo-elsewhere story's scratch directory; the story sets it for its child.

### `TEMPER_TEST_E4_RESTART`

- **Set:** test
- **Reads**
- **Form:** 1
- **Absent:** Run the first engine life.
- Select the restarted engine life in the Forgejo-elsewhere child; the story sets it.
- **Notes:** Any nonempty value selects the restart.

### `TEMPER_TEST_FORGEJO`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** The tests and tool that need pinned Forgejo refuse to run.
- Locate pinned Forgejo for owned-forge support, real checkout delivery and the git transcripts tool; CI or a developer sets it.

### `TEMPER_TEST_FORGEJO_RUNNER`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** The owned-forge tests that need the runner refuse to run.
- Locate the pinned Forgejo runner for owned-forge support; CI or a developer sets it.

### `TEMPER_TEST_FORGE_PARENT_HELPER`

- **Set:** test
- **Reads**
- **Form:** 1
- **Absent:** The helper test does nothing.
- Run the startup suite's inherited-listener helper; its fake Forgejo script sets it.
- **Notes:** Any nonempty value selects the helper.

### `TEMPER_TEST_LIVE_ACCOUNT`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** The landing story signs in interactively; rung-b0 requires a held account.
- Name a held ChatGPT account record for a live story; a developer sets it for proof-live.sh.
- **Notes:** A regular 0600 file, moved into a fresh store by account import; a kept store cannot import it again.

### `TEMPER_TEST_LIVE_ANTHROPIC_KEY`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** The key landing story skips, saying why.
- Name the Anthropic key file for forge-land-claude-key; the after-landing tier or a developer for proof-live.sh sets it.
- **Notes:** A nonempty regular 0600 file of the test's user, not a link; passed to account add-key by path, never read, printed or copied by the story.

### `TEMPER_TEST_LIVE_CLAUDE`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The Claude landing story refuses to run.
- Name a held Claude Code login for forge-land-claude; a developer sets it for proof-live.sh.
- **Notes:** A nonempty 0700 directory, not a link, whose basename is a valid source name; only for forge-land-claude.

### `TEMPER_TEST_LIVE_GITHUB_FIXTURES`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The live GitHub story refuses to run.
- Locate the live GitHub story's fixtures; proof-live-github.sh sets it.
- **Notes:** The directory holds app.json with the App and owner installation's public fixture.

### `TEMPER_TEST_LIVE_GITHUB_KEEP`

- **Set:** test
- **Reads**
- **Form:** 1
- **Absent:** Remove the scratch state on exit.
- Keep the live GitHub story's scratch state on exit; a developer sets it for proof-live-github.sh.
- **Notes:** Only 1 keeps the state.

### `TEMPER_TEST_LIVE_GITHUB_OWNER`

- **Set:** test
- **Reads**
- **Form:** OWNER
- **Absent:** The live GitHub story refuses to run.
- Select the live GitHub story's owner; proof-live-github.sh sets it.
- **Notes:** The story accepts only the probe owner, fabrig-ai.

### `TEMPER_TEST_LIVE_KEEP`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** Use the live-evidence directory under the host's scratch area, or beside the task directory for rung-b0.
- Select where proof-live.sh retains redacted notices; a developer may set it.
- **Notes:** The retained directory and notice are private; the transient client bearer is redacted.

### `TEMPER_TEST_LIVE_NOTICE`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** Print notices to stderr only.
- Append live notices to the private notice file; proof-live.sh creates it and sets the path.
- **Notes:** The transient notice may contain the full sign-in link; the retained copy is redacted.

### `TEMPER_TEST_LIVE_STORE`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The launcher chooses its live-store directory, or a fresh task-adjacent directory for rung-b0; the stories require the launcher's value.
- Select the live stories' persistent deployment stores; a developer or proof-live.sh sets it.
- **Notes:** Landing stories use a subdirectory per story; rung-b0 uses the directory itself. A held account needs a fresh store.

### `TEMPER_TEST_MANAGED_CLIENT`

- **Set:** test
- **Reads**
- **Form:** sign-in|adopt
- **Absent:** Do not run a fake client role.
- Select the managed sitting's fake client role; the sitting or proof-live-github-managed.py sets it.
- **Notes:** An unknown role is refused.

### `TEMPER_TEST_MANAGED_EVIDENCE`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** The managed relay refuses to run.
- Select the managed relay's redacted evidence file; the sitting or proof-live-github-managed.py sets it.
- **Notes:** The relay creates a new 0600 file.

### `TEMPER_TEST_MANAGED_MODE`

- **Set:** test
- **Reads**
- **Form:** fake|live
- **Absent:** The managed relay refuses to run.
- Select the managed relay's transport; the sitting or proof-live-github-managed.py sets it.
- **Notes:** Fake mode requires an upstream socket address; live mode uses the live transport.

### `TEMPER_TEST_MANAGED_PERSON`

- **Set:** test
- **Reads**
- **Form:** LOGIN
- **Absent:** The live launcher and managed relay refuse to run.
- Name the person making the GitHub App in the managed sitting; a developer supplies it to the launcher, which sets it for the relay.
- **Notes:** The fake sitting supplies its own person; the relay requires a nonempty login.

### `TEMPER_TEST_MANAGED_PREFIX`

- **Set:** test
- **Reads**
- **Form:** PREFIX
- **Absent:** The managed relay refuses to run.
- Limit the managed relay's branch namespace to this sitting; the sitting or proof-live-github-managed.py sets it.

### `TEMPER_TEST_MANAGED_SCRATCH`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The managed relay refuses to run.
- Locate the managed relay's scratch directory; the sitting or proof-live-github-managed.py sets it.
- **Notes:** The relay writes relay.jsonl and relay-ready.json here.

### `TEMPER_TEST_MANAGED_UPSTREAM`

- **Set:** test
- **Reads**
- **Form:** IP:PORT
- **Absent:** Fake mode refuses to run; live mode needs no upstream.
- Locate the managed relay's fake upstream; the sitting or proof-live-github-managed.py sets it.
- **Notes:** Parsed as a socket address in fake mode.

### `TEMPER_TEST_PLAYGROUND`

- **Set:** test
- **Reads**
- **Form:** DIR
- **Absent:** The playground browser story refuses to run.
- Locate the built playground for the browser story; the browser tier sets it.
- **Notes:** The story canonicalizes the path before serving it.

### `TEMPER_TEST_RATE`

- **Set:** test
- **Reads**
- **Form:** 1
- **Absent:** The suite prints nothing.
- Print the installation's HTTP routes and costs from the forge protocol's rate suite; a developer sets it.
- **Notes:** Any nonempty value enables printing.

### `TEMPER_TEST_RECORD_RELEASE`

- **Set:** test
- **Reads**
- **Form:** FILE
- **Absent:** Compare the release card tree against its checked recording.
- Write the confirm tests' release card tree recording to this path; a developer sets it.

### `TEMPER_TEST_SITTING_WAIT_SECS`

- **Set:** test
- **Reads**
- **Form:** SECONDS
- **Absent:** Do not delay.
- Delay the fake managed sitting before reading the adopted-card confirmation; a developer sets it for proof-live-github-managed.py.
- **Notes:** A nonnegative integer; ignored in live mode.

### `TEMPER_TEST_WRAPPER`

- **Set:** test
- **Reads**
- **Form:** COMMAND
- **Absent:** Run the build commands directly.
- Wrap builds in temper's proof scripts and real-loop fixture regeneration; a developer sets it.
- **Notes:** One command name, not shell words.
