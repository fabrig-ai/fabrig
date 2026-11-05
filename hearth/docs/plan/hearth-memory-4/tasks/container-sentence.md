+++
id = "container-sentence"
part = "sentence"
kind = "docs"
needs = []
files = ["hearth/docs/design/protocol/format.md"]
effort = "medium"
minutes = 5
+++

# One sentence of format.md, 3.5

## Goal

`hearth/docs/design/protocol/format.md`, 3.5 still names a container of the example fleet. The user has decided: the sentence is written around the target it
becomes, `chat-prod`, and the open question about it goes. It is unrelated to the rest of this plan, so it is one commit of its own.

Acceptance:

- `git grep -n 'east-chat' -- hearth` prints nothing, and `git grep -n "container name in 3.5" -- hearth/docs/design` prints nothing.
- The diff is the two edits of `## Steps` and nothing else: no section is renumbered, and the id base (`"ids": 327680`) stays as it is.
- `python3 -I -B tools/citations/citations.py --all` stays at 0, and `heavy cargo nextest run -p fabrig-boundary --test outward` passes.

## Inputs

- `hearth/docs/design/protocol/format.md`, 3.5 ("The same environment only") and 13
- `DESIGN.md`, 4: a surviving section keeps its number

## Owns and may change

- **Owns:** `hearth/docs/design/protocol/format.md`, the two edits below only. This departs from the implementer's text, which leaves `docs/design/` alone: these edits are the
  user's word, and this design sentence is the whole task.
- **`hearth/docs/design/protocol/format.status.md` doesn't change:** it says "No gap", and this is still true.
- **No test reads these lines:** `.config/gate/read-by-tests.txt` doesn't list the file, and the Rust tests that name `format.md` quote error messages and other sections.

## Steps

1. **3.5, "The same environment only".** The last sentence of the bullet's paragraph reads:

   ```
   host key in the lab, cannot be written. The container `east-chat`
   becomes the target `chat-prod`, with `hosts/chat/chat-prod.json`:
   ```

   It becomes:

   ```
   host key in the lab, cannot be written. The container is the target
   `chat-prod`, with `hosts/chat/chat-prod.json`:
   ```

   The JSON block under it stays.

2. **13, Open questions.** Remove the whole bullet that begins `**A bash fleet's container name in 3.5:**`, all six of its lines, down to `says nothing of a deployment.`. The bullets
   around it stay.

3. Run the citations check and `outward`. Commit, with a message that says this is the user's word to neutralize the name, and push.

## Report

The first line, the commit, the citations and `outward` results.
