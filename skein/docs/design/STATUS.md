# Status

The status of each of skein's design documents and drafts is beside it,
as `<name>.status.md`: where the code lags or departs from a design
document (**Not yet**, **Departs**), and where the design lags or
departs from a draft (**Design lags**, **Design departs**, **Open**),
as `DESIGN.md`, 2, says. A gap goes in its document's status file,
never on this page.

To roll them up, from the repository root:

    python3 -I -B tools/citations/status.py skein

It prints the open entries by label, a line for each status file with
an open entry, and what's due: the files over 48 KB, largest first, the
drafts whose status records no gap, and the status files it can't read.

This page is fixed: `python3 -I -B tools/citations/status.py --page skein`
prints it, and the citation checker refuses any other text
(`DESIGN.md`, 4).
