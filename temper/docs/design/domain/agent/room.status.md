# room: status

2026-11-19. Where the tree's code lags or departs from [room.md](room.md).

- **Not yet:** 2, "Text alone": `assignment::down` sends a field that is not UTF-8 as it is, so smith's encoding refuses the whole assignment, `Section.body not UTF-8` (`temper-engine-service-vocabulary/src/assignment.rs`, `down`, `messages`, `answered`).
- **Not yet:** 2, "Text alone": no fuzzy property holds that a resolution's assignments over random conflict sets with arbitrary bytes, first and resumed, always encode (`temper/tests/engine/domain/tests/suite/fuzzy/`).

