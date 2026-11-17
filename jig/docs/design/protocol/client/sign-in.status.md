# sign-in: status

2026-11-17. Where the tree's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** the whole document, *The asked link*: the clients' component keeps no asked link, and `LinkAsk` has no route [in flight: change 6834].

- **Departs:** the whole document, *Never taken, nothing made*: a take admitted before
  expiry remains reserved until the core answers, even past the approval's
  expiry. `jig-protocol-engine-clients/src/approval.rs`, `State::sweep`
  and `next_deadline`, exclude `Decision::Taking`; `approval_whose` and
  `approval_reply` do not check the expiry again. The implementation
  explicitly preserves the reservation as a printed link's does, rather
  than enforcing the section's unconditional expiry. Seen by reading;
  no runtime result establishes this boundary.
