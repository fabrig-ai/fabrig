# limits: status

2026-11-24. Where smith's code lags or departs from [limits.md](limits.md).

`limits-derivation.md`, sections 1.6 and 1.8's `host_limits` and the agent's profile derivation
are `smith-agent-profile`'s at main's `fdc26f71c6`, re-exported at the
service's and local service's old paths (smith's `../shell/README.md`, 2.6).
The move leaves the bounds and the gaps below unchanged.

`limits.md`, sections 2.1 and 2.2, and `limits-derivation.md`, section 1.9, and `limits.md`, section 4 have no known gap in endpoint limits,
`charter_models`, saved-window fallback quantities or entrance checks
against the endpoint rather than a model declaration.

Section 6's `sub_agent` row is covered at a declared room of five by
ten program-world rows, asking two to six calls with program and
credential children. Requests enter together; admitted children answer
and excess calls return `refused reason=too-many`. The reported
deployment abort on such a burst was no limit's but a file answer's
routing (programs-claude-code.status.md, 1.1); no production limit changed.

`limits-derivation.md`, sections 1.4 and 1.10, and `limits.md`, sections 4 and 6 have no known gap in a sub-agent's answer
allowance: `answer_bytes` equals `outcome_bytes`, the larger of 4,096 and
`tool_payload`. A UTF-8 answer is cut at a character's boundary; its
marker counts the omitted bytes and asks for a new child with a narrower
brief. The feedback bound measures that marker at its maxima beside the
answer and usage line. The profile checks cover complete delegated
feedback, its rendering and transcript receiving bounds at payloads 256,
10,662 and 63,740. The larger charter and outcome feedback bounds still
determine the measured profile figures below.

`limits-derivation.md`, section 1.10 has no known gap in charter and outcome allowances following
`tool_payload`. The profile table pins T at 16,803,810 for payload 10,662
and output 8,192, and at 18,431,842 for 63,740 and 48,000. The 131,072 and
98,500 row is refused at `turn-in-transcript`, with `session.said_text`
1,245,440 past `transcript.said_text` 1,048,576; it establishes no T for
that pair.

Section 11's fake-program table carries inputs through 100,000 bytes,
including 65,536, whole; it does not establish the requested 131,072-byte
case. Its declared payload stays below the transcript feedback ceiling.
The live story for 65,536 and 131,072 is written for after landing; no
runtime outcome is supplied here.

- **Not yet:** 2.4: a host's answer stops at 65,536 rather than 262,144
  bytes (`smith-domain-run/src/host.rs`, `HostAnswer::CAPACITY`).
- **Not yet:** 7, 9: `orphan_exit`, and the agent's `no_progress`, derived
  and listed among its effective limits. [in flight: change 16344]
- **Departs:** 2.4: a host's input admits up to 1,048,576 rather than
  262,144 bytes (`smith-domain-run/src/host.rs`, `HostInput::CAPACITY`),
  matching the channel's `host_ask_input` and transcript's `call_input`
  ceilings; the deployment's receiving limit still applies.
- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-profile/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).

