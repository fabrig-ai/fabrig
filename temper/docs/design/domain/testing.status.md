# testing: status

2026-11-02. Where temper's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2.4: GitHub's real-loop stories 3, 4, 5, 7 and 8; 1, 2 and 6
  run (`tests/real-loop/tests/suite/github_land.rs`, `github.rs`). 2.4, 2.5,
  7: the twins' documents and the one-process world's composition in
  `temper-playground`, which does not exist; `tests/playground`. 2.6:
  `a_default_box_with_a_sign_in_registration_starts_without_dns` is ignored:
  a start without DNS fails while the sign-in's endpoints resolve at start.
  2.6, 5.3: the cluster's 4.6 process story is ignored: a retry placed on a
  stopping worker's still-linked channel is journaled as an extra refused
  attempt, until jig takes a draining worker out of placement. 2.7: a live
  story ending at its task's first terminal failure. 4.3: the shell subset;
  skein's fake machine runs no `sh` and no `rg`. 4.3, 7:
  `temper-fake-git-hosted` in `crates/`; it is in `testing/`. 6.4: the
  settings stories that need temper's kinds of settings (`root.md`'s
  status).
- **Departs:** 5.3: the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
