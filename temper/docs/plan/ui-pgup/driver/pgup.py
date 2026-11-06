#!/usr/bin/env python3
"""Drive a real fake-box conversation through Chromium's input and sample frames.

python3 -I -B temper/docs/plan/ui-pgup/driver/pgup.py --bundle BUNDLE --out OUT
Build temper and the loopback bundle first (see reproduced.md). OUT must be new.
No product code, tier, external service, websocket package, or scripted DOM acts.

Contract: temper/docs/design/draft/chat-reading.md, section 5;
skein/docs/design/ui-web.md, section 7.
"""
import argparse
import base64
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import runpy
import signal
import socket
import subprocess
import tempfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[5]
Pipe = runpy.run_path(str(ROOT / 'temper/docs/plan/bundle-loading/driver/load.py'))['Pipe']
DRAFT = 'Keep these draft words intact.'
CARET = len(DRAFT) - 5

# All page script is observation. Scrolling, focus, typing and caret movement use CDP input.
SAMPLER = r"""(() => {
 window.probe = {frames: [], inputs: [], anchor: null, label: 'setup'};
 const visible = e => !!e && e.getBoundingClientRect().height > 0 && getComputedStyle(e).visibility !== 'hidden';
 window.probeLog = () => document.querySelector('[role="log"]');
 window.probePill = () => [...document.querySelectorAll('button')].find(e => e.textContent.includes('Latest'));
 window.probeComposer = () => document.querySelector('textarea');
 window.probeSnapshot = () => {
  const l=probeLog(), c=probeComposer(), a=probe.anchor;
  const paragraphs=l ? [...l.querySelectorAll('p')] : [];
  const p=a ? paragraphs.filter(p => p.textContent === a.text)[a.occurrence || 0] : null;
  return {t:performance.now(), top:l?.scrollTop, height:l?.scrollHeight, client:l?.clientHeight,
   away:l?.hasAttribute('data-log-away'), pill:visible(probePill()),
   focus:document.activeElement?.tagName, words:c?.value, caret:c?.selectionStart,
   writing:!!l?.textContent.includes('▍'),
   tail:l?.textContent.slice(-250), paragraphs:paragraphs.length,
   lineHeight:l ? (parseFloat(getComputedStyle(l).lineHeight) || parseFloat(getComputedStyle(l).fontSize)*1.2) : null,
   calls:l ? [...l.querySelectorAll('summary')].map(e=>e.textContent) : [],
   anchorTop:p?.getBoundingClientRect().top ?? null,
   anchorPresent:a ? !!p : null, label:probe.label};
 };
 window.probePick = label => {
  probe.label=label; probe.anchor=null;
  const l=probeLog(), r=l.getBoundingClientRect(), paragraphs=[...l.querySelectorAll('p')];
  // Attribution labels are not prose. Repeated scripted prose uses its text occurrence.
  const p=paragraphs.find(p=>{const b=p.getBoundingClientRect();return b.top>=r.top && b.top<r.bottom &&
   p.textContent.length>40;});
  if(p) probe.anchor={text:p.textContent,top:p.getBoundingClientRect().top,
    occurrence:paragraphs.filter(q=>q.textContent===p.textContent).indexOf(p)};
  return {before:probeSnapshot(),anchor:probe.anchor,index:probe.frames.length};
 };
 window.addEventListener('keydown', e => {
  if(e.key==='PageUp' && probe.endFired && !probe.endKey)
   probe.endKey=probePick('2/up-at-writing-mark-removal');
  if(e.key==='PageUp' || e.key==='PageDown') {
   if(probe.label.startsWith('race/')) probePick(probe.label);
   probe.inputs.push({...probeSnapshot(),key:e.key,anchor:probe.anchor});
  }
 }, true);
 window.addEventListener('wheel', () => {
  if(probe.label.startsWith('race/')) probePick(probe.label);
  probe.inputs.push({...probeSnapshot(),key:'wheel',anchor:probe.anchor});
 }, {capture:true,passive:true});
 let writing=false;
 new MutationObserver(() => {
  const now=!!probeLog()?.textContent.includes('▍');
  if(probe.endArmed && writing && !now) {
   probe.endArmed=false; probe.endFired=performance.now();
   pgupEnd(String(probe.endFired));
  }
  writing=now;
 }).observe(document, {childList:true,subtree:true,characterData:true});
 const frame=()=>{probe.frames.push(probeSnapshot()); requestAnimationFrame(frame);};
 requestAnimationFrame(frame);
})()"""


def text_piece(label, n):
    return (f'{label} paragraph {n:03d}. ' +
            'Read these words while the model adds another paragraph. ' * (3 if label == 'History' else 9) + '\n\n' +
            ('- First list item.\n- Second list item.\n\n' if n == 3 else ''))


def turns(label, count=16, call=True):
    pieces = [{'text': text_piece(label, n), 'hold': None} for n in range(count)]
    first = {'lines': [{'pieces': pieces}], 'finish': 'tool_calls' if call else 'stop', 'tokens': 1}
    if not call:
        return [first]
    first['lines'].append({'call': 'read', 'arguments': {'path': 'src/main.rs'}})
    second = {'lines': [{'pieces': [{'text': text_piece(label + ' after call', n), 'hold': None}
                                    for n in range(16)]}], 'finish': 'stop', 'tokens': 1}
    return [first, second]


def scripts(fast):
    result = [{'cue': f'Seed {n}', 'turns': [{'lines': [{'text': text_piece('History', n)}],
                                          'finish': 'stop', 'tokens': 1}]} for n in range(6)]
    if fast:
        for name in ('key-up', 'key-down', 'wheel-up', 'wheel-down'):
            # A fresh sixteen-piece stream supplies each independent input trial.
            result.append({'cue': name, 'turns': [{'lines': [{'pieces': [
                {'text': text_piece(name, 0), 'hold': None},
                *[{'text': f'{name} segment {n}. ' + 'Words arrive in this delivery. ' * 4 + '\n\n',
                   'hold': 'race-ready' if n == 0 else None} for n in range(14)],
                {'text': '\n\nRace complete.', 'hold': None}]}], 'finish': 'stop', 'tokens': 1}]})
    else:
        for name in ('slow-key', 'slow-wheel'):
            result.append({'cue': name, 'turns': turns(name)})
        for name in ('ending-held', 'ending-key'):
            result.append({'cue': name, 'turns': turns(name, count=16, call=False)})
    result.append({'cue': 'After ending', 'turns': [{'lines': [{'text': 'The next message arrived.'}],
                                                  'finish': 'stop', 'tokens': 1}]})
    return result


class Box:
    def __init__(self, binary, bundle, directory, fast):
        self.directory = directory
        self.flags = [part for flag in ('--config', '--data', '--cache') for part in (flag, str(directory))]
        subprocess.run([str(binary), 'init', '--box', *self.flags], check=True,
                       stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        path = directory / 'temper.json'
        config = json.loads(path.read_text())
        app = config['application']
        listeners = [app['clients']['listener'], app['machine']['fake']['listen'],
                     app['demo']['llm']['listen'], app['webhooks']['listener']]
        reservations = []
        replacements = {}
        for listener in listeners:
            sock = socket.socket()
            sock.bind(('127.0.0.1', 0))
            reservations.append(sock)
            replacements[listener['socket']] = '%s:%s' % sock.getsockname()
        def replace(v):
            if isinstance(v, str):
                for old, new in replacements.items():
                    if v == old:
                        return new
                    if v == 'http://' + old:
                        return 'http://' + new
                return v
            if isinstance(v, list):
                return [replace(x) for x in v]
            if isinstance(v, dict):
                return {k: replace(x) for k, x in v.items()}
            return v
        config = replace(config)
        app = config['application']
        app['clients']['bundle'] = str(bundle)
        # The base's demo parser accepts neither pieces nor pace. Its normal model
        # remains available, while the agent's demo endpoint uses a standalone peer.
        app['demo']['llm']['asked'] = [{'cue': 'Seed 0', 'turns': ['History ready.']}]
        app['demo']['llm']['cued'] = []
        model_socket = socket.socket()
        model_socket.bind(('127.0.0.1', 0))
        model_address = '%s:%s' % model_socket.getsockname()
        control_socket = socket.socket()
        control_socket.bind(('127.0.0.1', 0))
        control_address = '%s:%s' % control_socket.getsockname()
        self.release = 'http://' + control_address + '/release/race-ready'
        model_key = directory / 'model.key'
        app['demo']['llm']['key'] = str(model_key)
        model_document = {'listen': {'socket': model_address}, 'key': str(model_key),
                          'control': {'socket': control_address},
                          'tls': None, 'models': [{'name': 'demo', 'window': 1048576, 'output': 8192}],
                          'cued': [], 'asked': scripts(fast), 'otherwise': ['No scripted cue.'],
                          'pace': {'reply_ms': 20 if fast else 300}}
        model_path = directory / 'model.json'
        model_path.write_text(json.dumps(model_document))
        model_socket.close()
        control_socket.close()
        self.model_log = (directory / 'model.log').open('wb')
        self.model = subprocess.Popen([str(binary), 'fake', 'llm', '--document', str(model_path), '--seed', '7'],
                                      stdin=subprocess.DEVNULL, stdout=self.model_log, stderr=self.model_log,
                                      start_new_session=True)
        deadline = time.monotonic() + 10
        while not model_key.exists():
            if self.model.poll() is not None or time.monotonic() > deadline:
                raise RuntimeError('standalone fake model refused: ' + (directory / 'model.log').read_text())
            time.sleep(.02)
        path.write_text(json.dumps(config))
        agent = directory / 'agent.json'
        agent_value = replace(json.loads(agent.read_text()))
        for endpoint in agent_value['endpoints']:
            if endpoint['name'] == 'demo':
                endpoint['address'] = model_address
        agent.write_text(json.dumps(agent_value))
        self.origin = 'http://' + app['clients']['listener']['socket']
        for sock in reservations:
            sock.close()
        # Consume the box's printed one-use browser link, as the browser harness does.
        # The link is never retained in the driver output or report.
        self.log = (directory / 'box.log').open('wb')
        self.process = subprocess.Popen([str(binary), 'box', *self.flags], stdin=subprocess.PIPE,
                                        stdout=self.log, stderr=self.log, start_new_session=True)
        deadline = time.monotonic() + 30
        while True:
            try:
                with urllib.request.urlopen(self.origin, timeout=1) as response:
                    if response.status == 200:
                        links = re.findall(r'http://[^\s]+', (directory / 'box.log').read_text())
                        if links:
                            self.link = links[0]
                            break
            except OSError:
                if self.process.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError('fake box did not serve the welcome: ' + re.sub(r'https?://\S+', '<local link omitted>', (directory / 'box.log').read_text()))
                time.sleep(.05)

    def close(self):
        if self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGTERM)
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait()
        self.process.stdin.close()
        self.log.close()
        os.killpg(self.model.pid, signal.SIGTERM)
        self.model.wait(timeout=10)
        self.model_log.close()


class Driver:
    def __init__(self, pipe, width, out):
        self.pipe, self.width, self.out = pipe, width, out
        self.target = pipe.call('Target.createTarget', {'url': 'about:blank'})['targetId']
        self.session = pipe.call('Target.attachToTarget', {'targetId': self.target, 'flatten': True})['sessionId']
        pipe.on_event = self.event
        self.call('Page.enable')
        self.call('Runtime.enable')
        self.call('Runtime.addBinding', {'name': 'pgupEnd'})
        self.call('Emulation.setDeviceMetricsOverride', {'width': width, 'height': 900,
                                                        'deviceScaleFactor': 1, 'mobile': False})
        self.rows = []
        self.races = []
        self.previous_pages = []

    def event(self, message):
        if message.get('sessionId') == self.session and message['method'] == 'Page.javascriptDialogOpening':
            self.pipe.send('Page.handleJavaScriptDialog', {'accept': True}, self.session)
        elif message.get('sessionId') == self.session and message['method'] == 'Runtime.bindingCalled':
            if message['params']['name'] == 'pgupEnd':
                # The page only observes removal. The browser receives both input events over CDP.
                for kind in ('rawKeyDown', 'keyUp'):
                    self.pipe.send('Input.dispatchKeyEvent', {'type': kind, 'key': 'PageUp',
                                   'code': 'PageUp', 'windowsVirtualKeyCode': 33}, self.session)

    def call(self, method, params=None):
        return self.pipe.call(method, params, self.session)

    def js(self, expression):
        answer = self.call('Runtime.evaluate', {'expression': expression, 'returnByValue': True})
        if 'exceptionDetails' in answer:
            raise RuntimeError(str(answer['exceptionDetails']))
        return answer.get('result', {}).get('value')

    def wait(self, predicate, timeout=30):
        deadline = time.monotonic() + timeout
        while True:
            value = self.js(predicate)
            if value:
                return value
            if time.monotonic() >= deadline:
                raise TimeoutError(predicate + '\nPAGE: ' + self.js('document.body.innerText')[:3000])
            self.pause(.02)

    def pause(self, seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            self.pipe.pump(min(.02, end - time.monotonic()))

    def key(self, name, vk=None, modifiers=0):
        values = {'PageUp': 33, 'PageDown': 34, 'Enter': 13, 'End': 35, 'ArrowLeft': 37, 'a': 65}
        for kind in ('rawKeyDown', 'keyUp'):
            self.call('Input.dispatchKeyEvent', {'type': kind, 'key': name,
                      'code': 'KeyA' if name == 'a' else name,
                      'windowsVirtualKeyCode': vk or values[name], 'modifiers': modifiers})

    def click(self, expression):
        point = self.js(f'(() => {{const e={expression}; if(!e) return null; '
                        'const r=e.getBoundingClientRect(); return {x:r.x+r.width/2,y:r.y+r.height/2};})()')
        if not point:
            raise RuntimeError('pointer target absent: ' + expression)
        self.call('Input.dispatchMouseEvent', {'type': 'mouseMoved', **point})
        self.call('Input.dispatchMouseEvent', {'type': 'mousePressed', 'button': 'left', 'clickCount': 1, **point})
        self.call('Input.dispatchMouseEvent', {'type': 'mouseReleased', 'button': 'left', 'clickCount': 1, **point})

    def named(self, tag, name):
        self.click(f'[...document.querySelectorAll({json.dumps(tag)})].find(e => e.textContent.trim()==={json.dumps(name)})')

    def wheel(self, delta):
        point = self.js('(() => {const r=probeLog().getBoundingClientRect(); return {x:r.x+r.width/2,y:r.y+r.height/2};})()')
        self.call('Input.dispatchMouseEvent', {'type': 'mouseWheel', 'deltaX': 0, 'deltaY': delta, **point})

    def draft(self):
        self.click('probeComposer()')
        self.key('a', modifiers=2)
        self.call('Input.insertText', {'text': DRAFT})
        for _ in range(5):
            self.key('ArrowLeft')
        self.wait(f'probeSnapshot().words === {json.dumps(DRAFT)} && probeSnapshot().caret === {CARET}')

    def send(self, words):
        self.click("document.querySelector('textarea')")
        self.key('a', modifiers=2)
        self.call('Input.insertText', {'text': words})
        self.wait("[...document.querySelectorAll('button')].some(e => "
                  "(e.textContent.trim()==='Send' || e.getAttribute('aria-label')==='Send') && "
                  "!e.disabled && e.getAttribute('aria-disabled')!=='true')")
        self.key('Enter')
        self.pause(.08)

    def snapshot(self):
        return self.js('probeSnapshot()')

    def begin(self, label):
        return self.js('probePick(%s)' % json.dumps(label))

    def finish(self, label, start, rule, **extra):
        end = self.snapshot()
        frames = self.js(f'probe.frames.slice({start["index"]})')
        heights = [s['height'] for s in frames if s.get('height') is not None]
        anchor = start['anchor']
        moved = None if not anchor or end['anchorTop'] is None else round(end['anchorTop'] - anchor['top'], 3)
        expected_words, expected_caret = ('', 0) if extra.get('sendingConsumesDraft') else (DRAFT, CARET)
        focus = end['focus'] == 'TEXTAREA' and end['words'] == expected_words and end['caret'] == expected_caret
        row = {'step': label, 'before': start['before'], 'after': end, 'anchor': anchor, 'moved': moved,
               'minHeight': min(heights + [start['before']['height'], end['height']]), 'frames': len(frames),
               'missingFrames': sum(s['anchorPresent'] is False for s in frames),
               'maxFrameMove': max([abs(s['anchorTop'] - anchor['top']) for s in frames
                                    if anchor and s['anchorTop'] is not None] + [0]),
               'focusHolds': focus, **extra}
        row['viewHolds'] = rule(row)
        row['result'] = 'holds' if row['viewHolds'] and focus else 'fails'
        self.rows.append(row)
        print(self.width, label, row['result'], start['before']['top'], end['top'],
              'moved', moved, 'min', row['minHeight'], 'focus', end['focus'], flush=True)
        return row

    def action(self, label, action, rule, delay=.1):
        start = self.begin(label)
        action()
        self.pause(delay)
        return self.finish(label, start, rule)

    def foot(self):
        # Setup/recovery is also input. The pill is the person's explicit end request.
        if self.snapshot()['pill']:
            self.click('probePill()')
            self.pause(.04)
        for _ in range(80):
            s = self.snapshot()
            if s['height'] - s['client'] - s['top'] <= 1 and not s['away']:
                return
            self.key('PageDown')
            self.pause(.025)
        raise RuntimeError('could not return to foot with input')

    def ready(self, origin):
        self.call('Page.navigate', {'url': origin})
        self.wait("[...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='Try the playground')")
        self.named('button', 'Try the playground')
        self.wait("[...document.querySelectorAll('[role=status]')].some(e=>e.textContent.trim()==='Ready')")
        self.wait("[...document.querySelectorAll('a')].some(e=>e.textContent.trim()==='Chats')")
        self.named('a', 'Chats')
        self.wait("[...document.querySelectorAll('a')].some(e=>e.textContent.trim()==='+ New chat')")
        self.seed_chat()

    def seed_chat(self, navigate=False):
        if navigate:
            self.previous_pages.append(self.js('probe.frames'))
            origin = self.js('location.origin')
            self.call('Page.navigate', {'url': origin + '/chats/new'})
        else:
            self.named('a', '+ New chat')
        self.wait("location.pathname.endsWith('/chats/new') && "
                  "[...document.querySelectorAll('h1')].some(e=>e.textContent.trim()==='New chat') && "
                  "!!document.querySelector('textarea')")
        self.wait("!!document.querySelector(\"[data-region='temper-link'] > [role='status'].temper-live\")")
        if not self.js('!!window.probe'):
            self.js(SAMPLER)
        self.send('Seed 0')
        self.wait("probeLog() && probeLog().textContent.includes('History paragraph 000')")
        self.pause(.7)
        for n in range(1, 6):
            self.send(f'Seed {n}')
            self.wait(f"probeLog().textContent.includes('History paragraph {n:03d}')")
            self.pause(.5)
        self.draft()

    def start(self, cue):
        self.foot()
        self.send(cue)
        self.wait(f'probeLog().textContent.includes({json.dumps(cue + " paragraph 000")})')
        self.wait('probeSnapshot().writing')
        self.draft()

    def ended(self, cue):
        self.wait('!probeSnapshot().writing', timeout=40)
        self.pause(.3)

    def shot(self, name):
        data = self.call('Page.captureScreenshot', {'format': 'png'})['data']
        (self.out / f'{self.width}-{name}.png').write_bytes(base64.b64decode(data))

    def slow(self):
        at_foot = lambda r: at_end(r['after']) and not r['after']['pill']
        for modality in ('key', 'wheel'):
            cue = 'slow-' + modality
            self.start(cue)
            self.wait(f"probeLog().textContent.includes('{cue} paragraph 014')")
            move_up = (lambda: self.key('PageUp')) if modality == 'key' else (lambda: self.wheel(-600))
            self.action(f'1/{modality}/up', move_up,
                        lambda r: r['after']['top'] < r['before']['top'] and r['after']['pill'])
            # Start tracking the paragraph after the intentional move, before the call.
            start = self.begin(f'1/{modality}/hold-call-next-line')
            self.wait(f"probeLog().textContent.includes('{cue} after call paragraph 003')", timeout=30)
            self.finish(f'1/{modality}/hold-call-next-line', start,
                        lambda r: r['moved'] == 0 and r['maxFrameMove'] == 0 and not r['missingFrames'] and r['after']['pill'])
            self.shot(f'{modality}-holding')
            self.action(f'1/{modality}/up-again', move_up,
                        lambda r: r['after']['top'] < r['before']['top'] and r['after']['pill'])
            def down():
                for _ in range(80):
                    if at_end(self.snapshot()):
                        break
                    self.key('PageDown')
                    self.pause(.025)
            self.action(f'1/{modality}/down-to-foot', down, at_foot)
            self.action(f'1/{modality}/follow', lambda: self.pause(.95), at_foot, delay=0)
            move_up()
            self.pause(.08)
            move_up()
            self.pause(.08)
            self.action(f'1/{modality}/latest', lambda: self.click('probePill()'), at_foot)
            self.draft()  # Preserve the recorded loss, then restore the next scenario's precondition.
            self.ended(cue)
        self.ending()

    def ending(self):
        at_foot = lambda r: at_end(r['after']) and not r['after']['pill']
        self.start('ending-held')
        self.wait("probeLog().textContent.includes('ending-held paragraph 013')")
        self.action('2/up-before-end', lambda: self.key('PageUp'),
                    lambda r: r['after']['top'] < r['before']['top'] and r['after']['pill'])
        start = self.begin('2/hold-across-end')
        self.ended('ending-held')
        self.finish('2/hold-across-end', start,
                    lambda r: r['moved'] == 0 and r['maxFrameMove'] == 0 and not r['missingFrames'] and r['after']['pill'])
        self.action('2/ended-stays-up', lambda: self.pause(.3),
                    lambda r: r['moved'] == 0 and r['after']['pill'], delay=0)
        start = self.begin('2/send-returns-to-foot')
        self.send('After ending')
        self.wait("probeLog().textContent.includes('The next message arrived.')")
        self.finish('2/send-returns-to-foot', start, at_foot, sendingConsumesDraft=True)
        self.draft()
        self.end_keys()

    def end_keys(self):
        self.start('ending-key')
        self.wait("probeLog().textContent.includes('ending-key paragraph 014')")
        self.js('probe.endArmed=true; probe.endFired=null; probe.endKey=null')
        self.wait('!!probe.endKey', timeout=10)
        start = self.js('probe.endKey')
        ended_at = self.js('probe.endFired')
        self.pause(.04)
        row = self.finish('2/up-at-writing-mark-removal', start,
                          lambda r: r['after']['top'] < r['before']['top'] and r['after']['pill'])
        row['observedEndToPressMs'] = start['before']['t'] - ended_at
        self.pause(max(0, (ended_at + 200 - self.snapshot()['t']) / 1000))
        self.action('2/up-200ms-after-end', lambda: self.key('PageUp'),
                    lambda r: r['after']['top'] < r['before']['top'] and r['after']['pill'])

    def fast(self):
        for path_index, name in enumerate(('key-up', 'key-down', 'wheel-up', 'wheel-down')):
            if path_index:
                self.seed_chat(navigate=True)
            trials = []
            for index in range(30):
                self.start(name)
                # The second piece is held, so draft preparation cannot consume the stream.
                self.pause(.05)
                if name.endswith('down'):
                    self.wheel(-200)
                    self.pause(.05)
                if not self.snapshot()['writing']:
                    raise RuntimeError('race precondition: turn ended before input')
                with urllib.request.urlopen(urllib.request.Request(self.release, data=b'', method='POST'),
                                            timeout=5) as response:
                    if response.status != 204:
                        raise RuntimeError('fake model refused release')
                # Vary where the input falls in the 20 ms delivery cycle.
                self.pause((index % 20) / 1000)
                start = self.begin(f'race/{name}/{index + 1}')
                if name.startswith('key'):
                    self.key('PageDown' if name.endswith('down') else 'PageUp')
                else:
                    self.wheel(10000 if name.endswith('down') else -600)
                immediate = self.snapshot()
                # Allow further deliveries; the wait below requires a sampled text change.
                self.pause(.075 + (index % 5) * .003)
                after = self.snapshot()
                frames = self.js(f'probe.frames.slice({start["index"]})')
                entered = self.js('probe.inputs.findLast(s => s.label === %s)' % json.dumps(
                    f'race/{name}/{index + 1}'))
                if not entered or not entered['writing']:
                    raise RuntimeError('race precondition: no browser input during writing')
                self.wait('probe.frames.slice(%d).some(s => s.t >= %s && s.tail !== %s)' % (
                    start['index'], entered['t'], json.dumps(entered['tail'])), timeout=5)
                frames = self.js(f'probe.frames.slice({start["index"]})')
                after = self.snapshot()
                delivery = next((s for s in frames if s['t'] >= entered['t'] and
                                 s['tail'] != entered['tail']), None)
                trial = {'trial': index + 1, 'before': start['before'], 'immediate': immediate,
                               'input': entered, 'after': after, 'nextDelivery': delivery,
                               'anchor': entered['anchor'],
                               'moved': after['anchorTop'] - entered['anchor']['top']
                                        if entered['anchor'] and after['anchorTop'] is not None else None,
                               'minHeight': min([s['height'] for s in frames] + [after['height']]),
                               'frames': frames,
                               'focusHolds': all(s['focus'] == 'TEXTAREA' and s['words'] == DRAFT and
                                                 s['caret'] == CARET for s in [immediate, after])}
                classify_trial(name, trial)
                trials.append(trial)
                self.ended(name)
            result = {'path': name, 'presses': len(trials), 'lost': sum(t['lost'] for t in trials),
                      'reachedFoot': sum(t['reachedFoot'] for t in trials),
                      'reachedCurrentFoot': sum(t['reachedCurrentFoot'] for t in trials),
                      'focusFailures': sum(not t['focusHolds'] for t in trials),
                      'streamingNextDeliveries': sum(t['nextDelivery'] is not None and
                                                    t['nextDelivery']['writing'] for t in trials),
                      'unobservedDeliveries': sum(t['nextDelivery'] is None for t in trials), 'trials': trials}
            self.races.append(result)
            print(self.width, name, 'lost', result['lost'], '/', len(trials), flush=True)

    def save(self, name):
        frames = self.js('probe.frames')
        (self.out / f'{self.width}-{name}.json').write_text(json.dumps(
            {'width': self.width, 'rows': self.rows, 'races': self.races, 'frames': frames,
             'previousPages': self.previous_pages}, indent=2) + '\n')

    def close(self):
        self.call('Page.navigate', {'url': 'about:blank'})
        self.pipe.call('Target.closeTarget', {'targetId': self.target})


def at_end(s):
    return s.get('top') is not None and s['height'] - s['client'] - s['top'] <= 1


def classify_trial(name, trial):
    """Count a return to the foot as it was at input, including between frames.

    A compositor wheel can reach that former foot before a frame records a new,
    taller log. Retain the stricter current-height count alongside this inference.
    The later observation avoids counting an unapplied wheel event as lost merely
    because its first delivery preceded native scrolling. A later ending can also
    lose an upward scroll that survived its first delivery.
    """
    entered, after = trial['input'], trial['after']
    snapshots = [trial['immediate'], *trial['frames'], after]
    snapshots = [s for s in snapshots if s['t'] >= entered['t']]
    foot_at_input = entered['height'] - entered['client']
    current = any(at_end(s) for s in snapshots)
    reached = current or any(s['top'] >= foot_at_input - 1 for s in snapshots)
    delivery = trial['nextDelivery']
    if name.endswith('up'):
        lost = bool(delivery and (at_end(after) or not after['away']))
    else:
        lost = bool(reached and delivery and (not at_end(delivery) or delivery['away']) and
                    (not at_end(after) or after['away']))
    trial.update(reachedCurrentFoot=current, reachedFoot=reached, lost=lost,
                 footAtInput=foot_at_input)


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/debug/temper')
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--width', type=int, action='append', choices=(1440, 420))
    parser.add_argument('--mode', choices=('slow', 'fast', 'both', 'ending'), default='both')
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    started_utc = datetime.now(timezone.utc).isoformat()
    with tempfile.TemporaryDirectory(prefix='pgup-') as scratch:
        scratch = Path(scratch)
        with (scratch / 'chromium.log').open('wb') as stderr:
            pipe = Pipe(scratch / 'profile', stderr)
            try:
                version = pipe.call('Browser.getVersion')
                (args.out / 'version.json').write_text(json.dumps(version, indent=2) + '\n')
                for width in args.width or (1440, 420):
                    for mode in ('slow', 'fast') if args.mode == 'both' else (args.mode,):
                        directory = scratch / f'box-{width}-{mode}'
                        directory.mkdir(mode=0o700)
                        box = Box(args.binary.resolve(), args.bundle.resolve(), directory, mode == 'fast')
                        driver = Driver(pipe, width, args.out)
                        try:
                            driver.ready(box.link)
                            driver.release = box.release
                            if mode == 'slow':
                                driver.slow()
                            elif mode == 'fast':
                                driver.fast()
                            else:
                                driver.ending()
                            driver.save(mode)
                        finally:
                            driver.close()
                            box.close()
            finally:
                pipe.close()
    (args.out / 'timing.json').write_text(json.dumps({'started': started_utc,
        'ended': datetime.now(timezone.utc).isoformat(),
        'elapsedSeconds': time.monotonic() - started}, indent=2) + '\n')


if __name__ == '__main__':
    main()
