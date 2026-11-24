# Settled upgrade image

Made by `upgrade::the_settled_image_matches_the_story` in the one-process
world's focused suite: story A, its uncertain forge write, then a drained stop.
This is the current build's fixture, not the once-kept baseline.

Rewrite with `TEMPER_TEST_UPGRADE_IMAGE=1 cargo nextest run -p
temper-one-process-world --test suite -E
'test(=upgrade::the_settled_image_matches_the_story)'` (one shell line).
Run the same test without the variable to check deterministic bytes.
Never edit the image by hand. Rewriting replaces the whole directory.

- `state/`, `worker/`, `client/`, `setup/`: the four machine roots.
- `worker/kept/` holds the held mark of the one-process stop, which cancels without a drain and so leaves it as a crash does.
- `modes.txt`: every node's kind, mode, owner and machine path, including
  empty directories. Host `dot-git` components stand for machine `.git`.
- `configuration.json`: the story's startup configuration.
- `forge.image`: the fake peer's own lasting domain and service image.
- `outside.json`: issued synthetic tokens and users, stop wall in nanoseconds,
  mid-run task and attempt, and the kept terminal answer's bytes.
- `ledger.jsonl`: the story's mutation projection, in the fake forge's
  observation JSON-lines encoding (no ancestry judgment is stored).

The loader makes a cold referee store and preloads transcript/channel payloads;
source-map reads rebuild the committed prefix, cold parts and layouts before
recovery. No live referee state is kept.
