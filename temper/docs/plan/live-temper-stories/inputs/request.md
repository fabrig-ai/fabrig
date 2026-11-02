# live-temper: the two shelf-login stories fail under fab's after-landing pair (11-02)

Since plan live-temper (landed 11-02), fab's after-landing runs temper's two shelf-login live stories
(`temper::live a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` and
`a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`, `--run-ignored only`) with lent
logins: the ChatGPT record at `TEMPER_LIVE_ACCOUNT`, the Claude Code directory at
`TEMPER_LIVE_CLAUDE` (basename `claude`), a fresh 0700 `TEMPER_LIVE_STORE` and `TMPDIR` under the run.
The run at main c2922ded2 (hetzner8 `~/src/fabrig/fab/after-landing/c2922ded2b10-20261102T192738703354Z/live-temper.log`) failed both:

1. **ChatGPT story, 6.6 s:** `temper ctl client account import FILE --key connect-plan` (cwd
   `<store>/forge-land/first`, the client directory signed in: address, store, token present, all
   0700/0600) exits 1 with `temper ctl: record fileDIRECTORY could not be read`. Every directory on
   the store and record paths below `~/src/fabrig` is 0700 (`~/src/fabrig` and `~/src/fabrig/fab`
   are 0775, as for QA's hand walks, whose `account import W/account.json` from
   `~/src/fabrig/fab/walks/W` works). The message is two run together: `host/input.rs` ~150
   (`key_file_fail`) prefixes "record file" to any failure of `account import`, and
   `directory.rs` ~270-280 maps a dozen client-directory stages (Start, Exists, Checked, Private,
   Lock, Address, Token, Link, Store, Ready, Closing) to the one "DIRECTORY could not be read"; a
   private-mode refusal would read "is not private", so it is a failed open or read of another kind.
2. **Claude Code story, 56.6 s:** it runs, then fails `whole delegate set` (support/forge_land.rs
   ~1826-1827): `task list --project 2 --parent 2 --ended --json` exits 0 with `ended.rows.cut` true
   or `ended.rows.after` set: the ended delegates of the change are more than one page, and the
   story requires them whole on one page.

## What to do

1. Make the client directory's failure name its stage and the error (and separate "record file"
   from the reason), with a test; then find the cause of this import's failure (reproduce it in
   temper-client-shell-cli's tests with the layout above: a client directory under a nested 0700
   tree, a record in a 0700 directory of its own) and repair it there, red before, green after. If the
   cause is the environment the pair gives (a path, a name), say exactly what the story and ctl need,
   in the story's design text, so fab's pair can give it.
2. The delegate-set check: read the ended delegates whole by paging (`--after` as ctl's read
   prints it) instead of requiring one uncut page; or, if one page is the design, say why and raise
   the page so a landing's delegates fit. Test it in the story's own harness tests.

## Acceptance

fab's after-landing pair runs both stories green on a main holding the fixes (that run is the proof;
the stories need the lent logins, which an implementer's slot does not have: run the client and
harness tests in the slot). The owner's way (`scripts/proof-live.sh forge-land` / `forge-land-claude`)
keeps working.

## Out of scope

fab's after-landing code (its pair, the lending, the store layout) unless the cause is shown to be
there, in which case say so and stop; the other live stories, which stay ignored.
