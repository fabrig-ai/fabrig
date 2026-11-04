+++
id = "names-docs"
part = "names"
kind = "docs"
needs = ["citations-docs"]
files = ["hearth/docs/design/protocol/format.md", "hearth/docs/design/domain/render.md", "hearth/docs/design/domain/machine.md", "hearth/docs/design/domain/artifacts.md", "hearth/docs/design/domain/operating.md", "hearth/docs/design/domain/testing.md"]
effort = "medium"
minutes = 20
+++

# hearth's design names no private host, address or account

## Goal

The private names leave the design documents by the mapping in `notes/contracts.md`, 1.1, so the examples read as the fixtures do
once `names-hosts` and `names-account` land. What the examples mean stays.

Acceptance:

- These print nothing:

  ```sh
  git grep -n -E 'ekanayaka|194\.164\.162\.|/home/free|home-free|free@|Free User|[uU]sers/free' -- hearth/docs/design hearth/AGENTS.md
  git grep -n -E '`free`|"free"' -- hearth/docs/design
  ```

- A fence's columns stay aligned.
- `hearth/docs/design/protocol/format.md`'s fleet and `x1` examples name what `hearth/tests/config/examples/fleet.json` and `x1.json`
  name, as they did.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 1
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 1

## Owns and may change

- **Owns:** those names in the six documents under `files`. Writing `docs/design/` departs from the implementer's text, as
  `notes/common.md` says.
- **Not to touch:** anything else of those documents, and every digest in an example: nothing checks a document's digest, and the
  request moves goldens, not prose. `hearth/docs/design/draft/archive/factory.md` holds no hit; section 1.3's names stay.
- **Options:** the example in `protocol/format.md`, 4.2 where `"alice"` and `"bob"` already stand stays as it is. An example that would
  hold `alice` twice takes `bob`, and your report says so.

## Steps

1. Replace the hits. On this plan's base: `protocol/format.md` holds the domain, the hosts, an address, the account and
   `/home/free`, ten lines of the first search and a dozen of `free`; `domain/render.md`, 6.3 the addresses; `domain/machine.md`,
   `/home/free` in an fstab and `home-free` in a tree; `domain/artifacts.md`, the repository and `infra/`; `domain/operating.md`, a host;
   `domain/testing.md`, `home-free` in a table row.
2. Read the second search's neighbours by `git grep -n -w free -- hearth/docs/design`: the English word stays (`notes/contracts.md`, 1.2).
3. Commit (`hearth: the design's examples use reserved names`), run the checks in `notes/common.md`, push.

## Report

Lines changed per document; any place `bob` was taken; any name that looked private and isn't in the contract.
