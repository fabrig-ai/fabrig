# boot: status

2026-11-10. Where the tree's code lags or departs from [boot.md](boot.md).

- **Not yet:** 1: the machine's layout, EFI boot and bootstrap (M3),
  with the BIOS path in M3b and containers and pool hosts in M5.
- **Not yet:** 3.1, 3.3: assembling the boot command line and writing it
  into entries or slot fragments. M2's boot test does not assemble its
  command line yet (hearth/docs/design/testing/real-builds.md, 6).
- **Not yet:** 3.2: the activator's second validation of the command line.
- **Not yet:** 4.1, 4.2, 4.3: installing the generated configuration before
  kernel postinst, building the initramfs and the output's `/boot`, and
  verifying the Incus module list by booting the output in M2.
- **Not yet:** 4.4: mounting the next root for a soft reboot.
