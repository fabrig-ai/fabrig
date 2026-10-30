# Slot tools status

Automatic idle-target policy (`tools/slots/design.md`, 5.5): observation only.
`--automatic-targets` defaults to observation; `--automatic-targets live` refuses
before entering a sweep. All automatic candidates remain PROTECTED. Central
claim custody and terminal/no-resume evidence await supported producers; process,
source and host-wide borrower proofs remain UNKNOWN in the observation entrance.
Existing manual acknowledgement recovery and executable trimming remain separate.
No automatic deletion or timer activation is implemented by this change.
