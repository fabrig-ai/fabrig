#!/usr/bin/env python3
"""Replay only fix-look/fix-flow's touched states, on fresh fake boxes.

Build as README.md says. Use --out outside the checkout, --bundle and
--gallery. Sections: sheets, proposals, draft, rename, settings, position, all.
Setup clicks are prerequisites, not additional walked states. The real login,
funded brief and A4 answer sequence are unchanged and are not repeated here.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import time

from additions import (Addition, SHEETS, close_chat, focus_sample, has, index,
                       key, save, sheet_live, sheet_look, sheet_metrics)
from walk import ROOT, prepare


def settings(w):
    w.first(False, False)
    w.nav('Settings')
    w.mock_nav('Settings')
    w.c.wait('Chats can write drafts')
    status = w.c.js("""[...document.querySelectorAll('select')].filter(e=>
      e.checkVisibility({checkVisibilityCSS:true})).map(e=>({
      name:e.getAttribute('aria-label')||e.labels?.[0]?.innerText,
      value:e.selectedOptions[0]?.text,disabled:e.disabled}))""")
    save(w.out / f'status-{w.width}.json', status)
    if not any(s['value'] == 'On' and s['disabled'] for s in status):
        raise RuntimeError('N2: disabled Home brief does not say On')
    w.snap('settings-how-work-runs', 'Settings',
           'N2 fixed: disabled Home brief says On; D11 keeps the controls disabled.',
           'differs')
    for tab in [w.c, w.m]:
        tab.js("[...document.querySelectorAll('select')].find(e=>"
               "e.labels?.[0]?.innerText.includes('The brief on Home')||"
               "e.getAttribute('aria-label')==='The brief on Home')"
               "?.scrollIntoView({block:'center'})")
        shot = tab.shot
        tab.shot = lambda path, reset=True, shot=shot: shot(path, reset=False)
    w.snap('settings-status', 'Settings',
           'N2 fixed: Home brief is visibly On and disabled; D11 describes current behaviour.',
           'differs')


def rename(w):
    w.first(False, False)
    name = 'Hello'
    w.c.click(name, 'button')
    w.c.click('Rename ' + name, 'button')
    w.m.click('Default', 'button')
    w.m.click('Rename Default', 'button')
    result = {'form': w.c.js("""[...document.querySelectorAll('input,button,form')]
      .filter(e=>e.checkVisibility({checkVisibilityCSS:true})).map(e=>({
      tag:e.tagName,name:e.getAttribute('aria-label')||e.innerText,
      cls:e.className,height:e.getBoundingClientRect().height,
      minHeight:getComputedStyle(e).minHeight}))""")}
    w.snap('rename-form', 'Project switcher',
           'A2 fixed: selected-row field, primary Save; no Cancel or Rename act.')
    for action, value in [('save', 'Saved project'), ('enter', 'Entered project'),
                          ('escape', 'Discarded project'), ('empty', '')]:
        if action != 'save':
            if not has(w.c, 'Rename ' + name):
                w.c.click(name, 'button')
            w.c.click('Rename ' + name, 'button')
        w.c.fill('Project name' if has(w.c, 'Project name') else 'Rename', value)
        if action in ['save', 'empty']:
            w.c.click('Save', 'button')
        else:
            key(w.c, 'Enter' if action == 'enter' else 'Escape',
                13 if action == 'enter' else 27)
        if action in ['save', 'enter']:
            name = value
        immediate = focus_sample(w.c)
        time.sleep(.4)
        sample = focus_sample(w.c)
        sample['immediate'] = immediate
        if sample['focused']['tag'] != 'SUMMARY':
            raise RuntimeError('A3: rename focus did not return to summary')
        if any(t['outline'] != 'none' for t in sample['targets']):
            raise RuntimeError('A3: page target has an outline')
        w.words('Rename — ' + action,
                'Save and Enter save; Escape cancels; empty Save retains the name. '
                'Focus returns to the summary; next Tab retains a real control ring. '
                'Reference: client/README.md, 5.3 and 11.')
        w.snap('rename-' + action, 'Project switcher', 'A3 fixed: focus returns to summary.')
        key(w.c, 'Tab', 9)
        sample['next_tab'] = focus_sample(w.c)
        if sample['next_tab']['focused']['width'] != '2px':
            raise RuntimeError('A3: next real control lost its focus ring')
        w.snap('rename-' + action + '-tab', 'Project switcher',
               'A3: next Tab retains the real control focus ring.')
        result[action] = sample
    save(w.out / f'focus-metrics-{w.width}.json', result)


def draft(w):
    w.first(False, False)
    w.settings(False)
    w.send('Let us design Away mode.')
    w.c.wait('Hello from the local fake model.')
    w.m.click('Chats', 'link')
    w.m.click('+ New chat', 'link')
    w.mock_send('Let us design Away mode.')
    w.c.click('+ Draft', 'button')
    w.m.click('+ Draft', 'button')
    w.c.wait('docs/design/draft/away-mode.md')
    w.mock_send('Write this up as a draft beside us.')
    if w.width == 420:
        w.m.click('Draft', 'button')
    w.snap('draft-first-turn', 'Chat and draft', 'N3 recheck: first document offers Start goal; D7 retained rail.', 'differs')
    w.c.click('Chat', 'button')
    w.m.click('Chat', 'button')
    w.send('Add what happens to a running goal.', False)
    w.mock_send('Add what happens to a running goal.')
    w.c.wait('Edited')
    w.working_notes()
    w.c.click('Draft', 'button')
    w.m.click('Draft', 'button')
    w.snap('draft', 'Draft', 'N3 not reproduced: restored document still offers Start goal; white composer is a decision.', 'differs')
    w.c.click('Start goal…', 'button')
    w.m.click('Start goal…', 'button')
    w.snap('start-goal-sheet', 'Start goal', 'N3 not reproduced: goal sheet opens after the working-note round trip; available ways/budget follow decisions.', 'differs')


def proposals(w):
    w.stop()
    for number, (page, act) in enumerate(
            [(p, a) for p in ['Needs you', 'Home', 'Chat'] for a in ['accept', 'reject']], 1):
        if getattr(w, 'proposal_page', None) and page != w.proposal_page:
            continue
        if getattr(w, 'proposal_act', None) and act != w.proposal_act:
            continue
        child = Addition(w.out, w.width, w.box.number * 100 + number, 'proposals')
        child.prefix = page.lower().replace(' ', '-') + '-' + act
        try:
            child.first(False, False)
            child.nav('Settings')
            child.c.click('Start', 'link')
            child.c.fill('Describe how you work, or what to change',
                         'Show drafts and goals for this project.')
            child.c.click('Send', 'button')
            child.c.wait('I proposed showing drafts and goals.')
            if page != 'Chat':
                child.nav(page)
            child.box.login_ctl()
            save(child.out / f'proposal-{child.box.number}-before.json',
                 json.loads(child.box.ctl('inbox', '--json')))
            row = next(row for row in SHEETS if row[0] == act)
            case, gallery, title, says, keep, does = row
            opener = does
            if act == 'reject':
                if not has(child.c, 'Reject'):
                    child.words('Configure proposal acts',
                                'The configuring card offers Accept and Change something. '
                                'Reject is reached through Needs you. Reference: '
                                'client/README.md, 5.13, the settings proposal.')
                    child.snap('reject-unavailable', 'Configure',
                               'Configure offers Accept and Change something, with no direct Reject; '
                               'current written contract. Rejection is verified through Needs you/Home.',
                               'unreachable')
                    continue
                child.c.click('Reject', 'button')
                field = next(c['name'] for c in child.c.controls() if c['tag'] == 'TEXTAREA')
                child.c.fill(field, 'Keep the current project settings.')
                opener = 'Reject proposal' if has(child.c, 'Reject proposal') else 'Reject'
            if getattr(w, 'primary_only', False):
                child.words(title, says + ' Keep first; primary act second. Reference: client/README.md, 3.6.')
                child.c.click(opener, 'button')
                child.snap('primary-sheet', 'Configure', 'Direct primary press, without Escape/keep first.')
                child.c.click(does, 'button')
                time.sleep(.7)
                child.snap('primary-result', 'Configure', 'Direct primary result retained.')
            else:
                sheet_live(child, case, title, says, keep, does, opener)
            text = child.c.text()
            if 'No visible object with that name' in text:
                observations = child.out / 'observations.json'
                seen = json.loads(observations.read_text()) if observations.exists() else []
                seen.append({'id': 'B1', 'page': page, 'act': act, 'width': child.width,
                             'box': child.box.number, 'primary_only': getattr(w, 'primary_only', False),
                             'sheet': sheet_metrics(child.c), 'text': text})
                save(observations, seen)
                child.snap('accept-refused', 'Configure',
                           'B1: Accept leaves its sheet open with No visible object with that name; need remains 1.',
                           'differs')
            else:
                child.c.wait('Rejected' if act == 'reject' else 'Accepted')
                if sheet_metrics(child.c) or 'Malformed operation' in child.c.text():
                    raise RuntimeError('M3: proposal decision did not complete')
            save(child.out / f'proposal-{child.box.number}-after.json',
                 json.loads(child.box.ctl('inbox', '--json')))
        finally:
            child.stop()


def sheets(w):
    # The five reachable sheets exercise Escape, keep and the primary act.
    # Task variants retain the named gallery fallback and its explicit limit.
    w.only_cases = None
    close_chat(w)


def position(w):
    sheet_look(w)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--section', choices=['sheets', 'proposals', 'draft', 'rename', 'settings', 'position', 'all'], default='all')
    parser.add_argument('--width', type=int, choices=[1440, 420])
    parser.add_argument('--page', choices=['Needs you', 'Home', 'Chat'], help='Limit proposal entry points.')
    parser.add_argument('--act', choices=['accept', 'reject'], help='Limit proposal actions.')
    parser.add_argument('--primary-only', action='store_true', help='Probe a proposal primary act without preceding cancellation.')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--gallery', type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    out = args.out.expanduser().resolve()
    if out.exists():
        raise SystemExit('Use a fresh output directory.')
    started = time.monotonic()
    prepare(out)
    shutil.copytree(args.bundle.resolve(), out / 'build/bundle')
    shutil.copytree(args.gallery.resolve(), out / 'gallery')
    save(out / 'build.json', {
        'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'builder': socket.gethostname(), 'started_utc': time.strftime('%FT%TZ', time.gmtime()),
        'binary_sha256': hashlib.sha256((out / 'build/temper').read_bytes()).hexdigest(),
        'bundle': (out / 'build/bundle/index.html').read_text()})
    failures = []
    sections = ['settings', 'draft', 'rename', 'sheets', 'proposals', 'position'] if args.section == 'all' else [args.section]
    number = 0
    try:
        for section in sections:
            for width in [args.width] if args.width else [1440, 420]:
                number += 1
                w = None
                try:
                    w = Addition(out, width, number, section)
                    w.proposal_page = args.page
                    w.proposal_act = args.act
                    w.primary_only = args.primary_only
                    globals()[section](w)
                except Exception as error:
                    failures.append({'section': section, 'width': width, 'error': str(error)})
                    save(out / 'faults.json', failures)
                    print(f'{section} {width}: {error}', flush=True)
                finally:
                    if w:
                        w.stop()
                    index(out)
    finally:
        shutil.rmtree(out / 'boxes', ignore_errors=True)
        shutil.rmtree(out / 'build', ignore_errors=True)
        save(out / 'timing.json', {'seconds': time.monotonic() - started,
                                  'finished_utc': time.strftime('%FT%TZ', time.gmtime())})
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    main()
