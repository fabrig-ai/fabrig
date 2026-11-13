# simulator: status

2026-11-13. Where skein's code lags or departs from [simulator.md](simulator.md).

Section 3.2's descriptor window sizes and ordered window-change signals
are present, including `ENOTTY` on descriptors without a window.

- **Not yet:** 3.1, 3.3, 4: the records of a tree's metadata but
  `HardLink`, and of the filesystem (../runtime/kernel.md, 6.4 and 6.5),
  their disk faults, and what a power loss keeps of them. 3.4: cgroups,
  contained spawn and their faults.
- **Not yet:** 3.1, 3.3: `HardLink` through the machine seam, and what a
  power loss keeps of a second name [in flight: change 2656].
