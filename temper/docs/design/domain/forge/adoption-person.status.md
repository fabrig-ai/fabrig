# adoption-person: status

2026-11-22. Where the tree's code lags or departs from [adoption-person.md](adoption-person.md).

- **Not yet:** 1, *Only under a box*: a link restored by a connector built
  without `--box` survives when its person remains the deployment's sole
  owner. `temper-engine-domain-forge/src/link.rs`, `owner`, clears only a
  different person; `person_of` blocks new links outside the box, but
  `forges.rs`, `link`, still returns the retained one and the root's
  `forge_person` uses it. No story covers that transition.
