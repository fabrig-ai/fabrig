#!/usr/bin/env python3
"""Rewalk keyboard project creation and the rename form after the look fix."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
from walk import Walk, prepare, write_index


def focused(tab):
    return tab.js("""(()=>{const e=document.activeElement,s=getComputedStyle(e),r=e.getBoundingClientRect();
      return {tag:e.tagName,name:e.getAttribute('aria-label')||e.labels?.[0]?.innerText||e.innerText,tabindex:e.getAttribute('tabindex'),outline:s.outlineStyle,
        width:s.outlineWidth,focusVisible:e.matches(':focus-visible'),height:r.height}})()""")


def key(tab, name, number):
    for kind in ['keyDown', 'keyUp']:
        tab.call('Input.dispatchKeyEvent', type=kind, key=name, code=name, windowsVirtualKeyCode=number)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--width', type=int, choices=[1440, 420], required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    out = args.out.resolve()
    if out.exists():
        raise SystemExit('Use a fresh output folder.')
    prepare(out)
    shutil.copytree(args.bundle.resolve(), out / 'build/bundle')
    walk = Walk(out, args.width, 1)
    try:
        walk.first(False, False)
        walk.c.click('Hello', 'button')
        walk.c.fill('+ New project', 'Second walk project')
        key(walk.c, 'Enter', 13)
        walk.c.wait('Second walk project')
        measurements = {'created': focused(walk.c)}
        if measurements['created']['outline'] != 'none':
            raise RuntimeError('A1: project creation still outlines its page target')
        walk.m.click('Default', 'button')
        walk.m.click('+ New project', 'button')
        walk.snapshot(7, 'project-added', 'Home', 'A1 fixed: heading has no outline; mock creation is a toast.', verdict='unreachable')
        key(walk.c, 'Tab', 9)
        measurements['control'] = focused(walk.c)
        if not measurements['control']['focusVisible'] or measurements['control']['outline'] == 'none':
            raise RuntimeError('A1: a keyboard control lost its focus ring')
        walk.c.click('Second walk project', 'button')
        walk.c.click('Rename Second walk project', 'button')
        walk.m.click('Rename Default', 'button')
        measurements['rename'] = focused(walk.c)
        measurements['mock_rename'] = focused(walk.m)
        walk.snapshot(7, 'project-rename', 'Project switcher', 'A2: rename form sits below the rows with a visible Cancel; mock replaces the selected row.')
        key(walk.c, 'Escape', 27)
        key(walk.m, 'Escape', 27)
        measurements['escape_left_cancel'] = any(c['name'] == 'Cancel' for c in walk.c.controls())
        measurements['escape'] = focused(walk.c)
        walk.snapshot(7, 'rename-escape-focus', 'Project switcher', 'A3: Escape closes the editor but focus goes to the new-project field instead of the switcher.')
        if not any(c['name'] == 'Rename Second walk project' for c in walk.c.controls()):
            walk.c.click('Second walk project', 'button')
        walk.c.click('Rename Second walk project', 'button')
        walk.c.click('Cancel', 'button')
        measurements['cancel'] = focused(walk.c)
        walk.snapshot(7, 'rename-cancel-focus', 'Project switcher', 'A3: Cancel focuses main instead of the switcher; A1 removes the page target outline.')
        (out / 'focus-metrics.json').write_text(json.dumps(measurements, indent=2))
        print(json.dumps(measurements), flush=True)
    finally:
        walk.stop()
        write_index(out)
        shutil.rmtree(out / 'boxes', ignore_errors=True)
        shutil.rmtree(out / 'build', ignore_errors=True)


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    main()
