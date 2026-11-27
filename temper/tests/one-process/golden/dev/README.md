# Dev's build's deployment image

Made by build **13efa774615022299aea9ed57b2577df517494ad**, by
`upgrade::the_settled_image_matches_the_story` in `temper-one-process-world`'s
focused suite: story A, its uncertain forge write, then a drained stop.

Copied unchanged from that commit's tree at
`temper/tests/one-process/golden/settled/`, using `git archive`. Every file
except this README is byte for byte from that tree. **Nothing writes this
directory again.** The settled image's README rewrite instruction does not
apply here: this is a golden deployment, not the current story's output
(`temper/docs/design/protocol/floor.md`, 5).

In each loaded copy, the loader appends one commit with the two frame layout
rows written by dev.fabrig.ai's 2026-11-25 08:44Z open: `jig-store-core`'s
`TurnRecord` at id 6 and `smith-transcript`'s `Turn` at id 7. It fingerprints
the corresponding schema texts already in this store. The same commit
advances the copy's deployment header to its commit number. The kept bytes
predate that open and do not change.

`state/`, `worker/`, `client/` and `setup/` are the four machine roots.
`modes.txt` records private modes and empty directories that git cannot keep;
the floor's loader recreates them. Host `dot-git` components represent machine
`.git` components. `configuration.json` is the kept startup configuration.
The forge's `forge.image`, `outside.json` and `ledger.jsonl` are kept unchanged
too; the reader loads the machine roots alone, since restoration and history
reads need no forge write. All identities and credentials belong to the fakes.

`golden::dev_image_opens_and_serves_on_this_build` starts this build with its
production owner sets, reads the kept person, chat and ended tasks through
the shipped ctl and the kept client's token, paginates each ended task's full
activity, and repeats the start over the started copy without making another
generation. Source files stay byte for byte unchanged; shared logs keep their
source-extent prefix unchanged and may only append after it as task 6 resumes
(jig's `protocol/store/logs.md`, 7). The reader uses the floor's configuration
adapter and operator prepare step on the loaded copy, with no dev-specific
limit override; no kept file is rewritten.

The image has live tasks 1, 5 and 6 and ended tasks 2, 3 and 4. Their six
history logs (2, 4, 6, 8, a, c in hexadecimal) and five transcript logs
(3, 7, 9, b, d) all live in shard `0000000000000000`. It holds no refused
delegation. The production engine and worker runtime stay attached during
both starts; no forge mutation is needed to reach ready.
