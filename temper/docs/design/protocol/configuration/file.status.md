# file: status

2026-11-19. Where temper's code lags or departs from [file.md](file.md).

- **Not yet:** 5: the machine's facts take the host's name from `skein_shell::host_name()`, not `/proc/sys/kernel/hostname`; 10: the facts' test that reads it without `/proc`. [in flight: change 10337]
