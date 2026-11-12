# channel: status

2026-11-12. Where skein's code lags or departs from [channel.md](channel.md).

- **Not yet:** 5.2 and 8.3: the machine's *version* refusal names both ranges in its text, and the owner's terminal for a refusal by this side carries the text it sent [in flight: change 1754].

- **Departs:** 8.3: four cells of `skein-channel/src/machine.rs` break "its
  owner hears closed once, with why" or "closing stops now", because the
  request of 2026-11-05 held the machine's behaviour still while its flags
  became states; `skein-channel/src/tests.rs` pins each, and no debt item
  holds them. A credential past `Limits::credential`: `open` goes closed with
  no event, and a later close is ignored, so the owner hears nothing
  (`oversized_open_credential_silently_closes_and_owner_close_is_ignored`).
  With a stream each way, a framing error after `Finish` went down: the
  `Refuse` goes and drains and no terminal follows, until the owner closes
  and hears `Owner`, never the framing error
  (`two_pipes_framing_error_after_finish_waits_for_owner_close`). Over one
  stream, a failure and then a grant of the output right: the right is
  neither used nor released and no terminal follows, and a close is ignored
  (`one_stream_output_failure_then_grant_has_no_terminal`). With a stream each
  way, once the read stream has failed or ended inside a frame: the queue
  drains as this section has it, but the owner's close is ignored until it
  has, so a queue that never drains never closes
  (`two_pipes_truncation_drains_ignores_close_and_refusal_failure_reports_in_order`).
