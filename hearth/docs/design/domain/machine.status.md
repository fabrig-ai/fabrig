# machine: status

2026-11-05. Where hearth's code lags or departs from [machine.md](machine.md).

- **Not yet:** 3, 4, 8: the machine's layout, EFI boot and bootstrap (M3),
  with the BIOS path in M3b and containers and pool hosts in M5.
- **Not yet:** 6.1, 6.3: assembling the boot command line and writing it
  into entries or slot fragments. M2's boot test does not assemble its
  command line yet (hearth/docs/design/domain/testing.md, 6.6).
- **Not yet:** 6.2: the activator's second validation of the command line.
- **Not yet:** 7.1, 7.2, 7.3: installing the generated configuration before
  kernel postinst, building the initramfs and the output's `/boot`, and
  verifying the Incus module list by booting the output in M2.
- **Not yet:** 7.4: mounting the next root for a soft reboot.
- **Not yet:** 10: the disk and boot domains, their worlds, and the firmware,
  GRUB and boot models and tool fakes.
