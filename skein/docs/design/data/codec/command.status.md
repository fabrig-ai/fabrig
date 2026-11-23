# command: status

2026-11-23. Where the tree's code lags or departs from [command.md](command.md).

- **Not yet:**
  - 8: the patch command reads and writes its files on disk directly; no `Tree`, `current_in` or `Tree::landed`, and `Landed::read` stands; 9: no witness of a tree on disk against one in memory [in flight: change 13887]
  - 9: the run's tables (`patch_run`) are over scratch directories in skein-codegen's suite [in flight: change 13888]
  - 9: the check's findings, the registered families and the maps refusals are in skein-codegen's suite, each maps row through the binary [in flight: change 13889]
  - 9: the generated module, wire floors, bridges and their goldens are over scratch directories in skein-codegen's suite [in flight: change 13890]
  - 9: the story runs over scratch directories, in a real-loop binary [in flight: change 13891]
