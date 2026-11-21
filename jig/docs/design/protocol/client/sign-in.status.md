# sign-in: status

2026-11-21. Where the tree's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** the whole document, a service's credential, *Replaced*: its records on jig's client wire 43 [in flight: change 11704].
- **Not yet:** the whole document, a service's credential, *Replaced*: the component's draw at each `ServiceReissued`, its keep in the old digest's place and its busy answer to the service's sign-ins meanwhile [in flight: change 11706].

- **Departs:** the whole document, *Never taken, nothing made*: a take admitted before
  expiry remains reserved until the core answers, even past the approval's
  expiry. `jig-protocol-engine-clients/src/approval.rs`, `State::sweep`
  and `next_deadline`, exclude `Decision::Taking`; `approval_whose` and
  `approval_reply` do not check the expiry again. The implementation
  explicitly preserves the reservation as a printed link's does, rather
  than enforcing the section's unconditional expiry. Seen by reading;
  no runtime result establishes this boundary.
