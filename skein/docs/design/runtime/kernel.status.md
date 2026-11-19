# kernel: status

2026-11-19. Where skein's code lags or departs from [kernel.md](kernel.md).

Section 6.3 has no known gap in reading a requested `SIGHUP` as a
shutdown, with the sender on Linux and none on macOS.

Section 6.2 has no known gap in `Way::Null`: Linux and macOS install
`/dev/null` read and write through spawn file actions, and the simulator
gives the descriptor an immediate read end and accepts its writes.

