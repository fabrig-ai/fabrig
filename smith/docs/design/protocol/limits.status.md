# limits: status

2026-11-07. Where smith's code lags or departs from [limits.md](limits.md).

Sections 2.1, 2.2, 3.9 and 4 have no known gap in endpoint limits,
`charter_models`, saved-window fallback quantities or entrance checks
against the endpoint rather than a model declaration.

Section 3.4 has no known gap in a sub-agent's answer allowance following
the declared `tool_payload`, including the advertised cap, explicit
overflow marker and usage line. The profile checks cover complete
delegated feedback, its rendering and transcript receiving bounds at
payloads 256, 10,662 and 63,740. The larger charter and outcome feedback
bounds still determine the measured profile figures below.

Section 3.10 has no known gap in charter and outcome allowances following
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
- **Departs:** 2.4: a host's input admits up to 1,048,576 rather than
  262,144 bytes (`smith-domain-run/src/host.rs`, `HostInput::CAPACITY`),
  matching the channel's `host_ask_input` and transcript's `call_input`
  ceilings; the deployment's receiving limit still applies.
- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-service/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).
