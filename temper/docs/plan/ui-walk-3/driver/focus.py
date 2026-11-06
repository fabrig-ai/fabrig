#!/usr/bin/env python3
"""Replay rename Save/Enter/Escape/empty-name focus and sign-out; see additions.py."""
import signal


def focused(tab):
    return tab.js("""(()=>{const e=document.activeElement,s=getComputedStyle(e),r=e.getBoundingClientRect();
      return {tag:e.tagName,name:e.getAttribute('aria-label')||e.labels?.[0]?.innerText||e.innerText,tabindex:e.getAttribute('tabindex'),outline:s.outlineStyle,
        width:s.outlineWidth,focusVisible:e.matches(':focus-visible'),height:r.height}})()""")


def key(tab, name, number):
    tab.call('Page.bringToFront')
    for kind in ['keyDown', 'keyUp']:
        tab.call('Input.dispatchKeyEvent', type=kind, key=name, code=name, windowsVirtualKeyCode=number)


def main():
    # Keep the older entry point while using the complete rename/sign-out walk.
    import sys
    from additions import main as additions_main
    from walk import ROOT
    sys.argv[1:1] = ['--section', 'rename', '--gallery', str(ROOT / 'target/ui-walk-3-gallery')]
    additions_main()


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    main()
