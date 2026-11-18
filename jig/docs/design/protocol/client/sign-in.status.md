# sign-in: status

2026-11-18. Where the tree's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** *A sign-in approved by a browser*, *Its records*: client wire 36's approval records, an ask's port and a take's grant [in flight: change 8875].
- **Not yet:** *A sign-in approved by a browser*: each ask's kind and source, two waiting per source by the public name, a loopback return's port, its grant drawn and answered once, and the take with it [in flight: change 8881].


- **Departs:** the whole document, *Never taken, nothing made*: a take admitted before
  expiry remains reserved until the core answers, even past the approval's
  expiry. `jig-protocol-engine-clients/src/approval.rs`, `State::sweep`
  and `next_deadline`, exclude `Decision::Taking`; `approval_whose` and
  `approval_reply` do not check the expiry again. The implementation
  explicitly preserves the reservation as a printed link's does, rather
  than enforcing the section's unconditional expiry. Seen by reading;
  no runtime result establishes this boundary.
