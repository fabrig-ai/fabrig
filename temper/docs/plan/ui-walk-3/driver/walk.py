#!/usr/bin/env python3
"""Replay the seven-step fake-box walk; evidence stays outside the checkout."""
import argparse
import atexit
import base64
import html
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import time
import urllib.request

from cdp import CDP

ROOT = Path(__file__).resolve().parents[5]
DEFAULT = Path.home() / 'src/fabrig/ui-walk-3/walk'


class Browser:
    def __init__(self, out, width):
        self.width = width
        self.profile = out / f'chromium-{width}-{time.time_ns()}'
        self.profile.mkdir(mode=0o700)
        self.log = open(out / f'chromium-{width}.log', 'ab')
        executable = os.environ.get('SKEIN_TEST_CHROMIUM', 'chromium')
        self.process = subprocess.Popen([executable, '--headless', '--no-sandbox', '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--remote-debugging-port=0', f'--user-data-dir={self.profile}', '--allow-file-access-from-files', 'about:blank'], stdout=self.log, stderr=self.log, stdin=subprocess.DEVNULL, start_new_session=True)
        atexit.register(self.stop)
        deadline = time.monotonic() + 20
        port_file = self.profile / 'DevToolsActivePort'
        while not port_file.exists():
            if time.monotonic() > deadline:
                raise RuntimeError('Chromium did not start')
            time.sleep(.1)
        self.port = int(port_file.read_text().splitlines()[0])
        self.client = self.tab()
        self.mock = self.tab()

    def tab(self):
        request = urllib.request.Request(f'http://127.0.0.1:{self.port}/json/new?about:blank', method='PUT')
        target = json.load(urllib.request.urlopen(request))
        tab = CDP(target['webSocketDebuggerUrl'])
        tab.call('Page.enable')
        tab.call('Runtime.enable')
        tab.call('Emulation.setDeviceMetricsOverride', width=self.width, height=1000, deviceScaleFactor=1, mobile=False)
        return tab

    def close_tab(self, tab):
        target=tab.call('Target.getTargetInfo')['targetInfo']['targetId']
        urllib.request.urlopen(f'http://127.0.0.1:{self.port}/json/close/{target}').read()
        tab.sock.close()

    def stop(self):
        if self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=20)
        self.log.close()
        shutil.rmtree(self.profile, ignore_errors=True)


class Box:
    def __init__(self, out, number):
        self.out = out
        self.number = number
        self.path = out / 'boxes' / str(number)
        for name in ['config', 'data', 'cache', 'ctl']:
            (self.path / name).mkdir(mode=0o700, parents=True)
        self.logpath = out / f'box-{number}.log'
        self.log = open(self.logpath, 'wb')
        self.process = subprocess.Popen([str(out / 'build/temper'), 'box', '--fake-backends', '--bundle', str(out / 'build/bundle'), '--config', str(self.path / 'config'), '--data', str(self.path / 'data'), '--cache', str(self.path / 'cache')], stdout=self.log, stderr=self.log, stdin=subprocess.DEVNULL, start_new_session=True)
        atexit.register(self.stop)
        deadline = time.monotonic() + 40
        while True:
            text = self.logpath.read_text()
            match = re.search(r'Open this link in a browser on this machine\.[\s\S]*?\n(http://[^\s]+)', text)
            if match:
                self.link = match[1]
                break
            if self.process.poll() is not None or time.monotonic() > deadline:
                raise RuntimeError(f'Box {number} did not start; read {self.logpath}')
            time.sleep(.2)

    def restart_with_sitting(self, millis):
        self.stop()
        config=self.path/'data/box/fake-backends/config/temper.json'
        document=json.loads(config.read_text())
        document['application']['limits'].setdefault('overrides',{})['forge.sitting_ms']=millis
        config.write_text(json.dumps(document))
        self.log=open(self.logpath,'ab')
        before=self.logpath.stat().st_size
        self.process=subprocess.Popen(self.process.args,stdout=self.log,stderr=self.log,stdin=subprocess.DEVNULL,start_new_session=True)
        deadline=time.monotonic()+40
        while time.monotonic()<deadline:
            text=self.logpath.read_text()[before:]
            links=re.findall(r'\n(http://[^\s]+)',text)
            if links:
                self.link=links[-1];return
            if self.process.poll() is not None:break
            time.sleep(.2)
        raise RuntimeError('Restart did not print its sign-in link')

    def ctl(self, *words, input=None):
        command = [str(self.out / 'build/temper'), 'ctl', str(self.path / 'ctl'), *words]
        result = subprocess.run(command, input=input, stdin=subprocess.DEVNULL if input is None else None, capture_output=True, text=True, timeout=330)
        if result.returncode:
            raise RuntimeError(f'ctl {words}: {result.stderr}')
        return result.stdout

    def login_ctl(self):
        before=len(re.findall(r'\n(http://[^\s]+)',self.logpath.read_text()))
        try:
            self.ctl('sign-in','-',input=self.link+'\n')
            return
        except RuntimeError as error:
            if 'refused' not in str(error):raise
        deadline=time.monotonic()+10
        while time.monotonic()<deadline:
            links=re.findall(r'\n(http://[^\s]+)',self.logpath.read_text())
            if len(links)>before:
                self.ctl('sign-in','-',input=links[-1]+'\n');return
            time.sleep(.1)
        raise RuntimeError('No fresh printed link for ctl')

    def stop(self):
        if self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=60)
            lifecycle=self.path/'data/box/fake-backends/data/box/logs/box.log'
            if not lifecycle.exists() or 'Stopped' not in lifecycle.read_text():
                raise RuntimeError(f'Box {self.number} did not record Stopped in its lifecycle log')
            shutil.copy2(lifecycle,self.out/f'box-{self.number}-lifecycle.log')
        self.log.close()


def prepare(out):
    out.mkdir(mode=0o700, parents=True, exist_ok=True)
    (out / 'build').mkdir(mode=0o700, exist_ok=True)
    for name in ['temper', 'fake-claude']:
        shutil.copy2(ROOT / 'target/debug' / name, out / 'build' / name)



class Walk:
    def __init__(self, out, width, number):
        self.started=time.time()
        self.out, self.width = out, width
        self.box = Box(out, number)
        self.browser = Browser(out, width)
        self.c, self.m = self.browser.client, self.browser.mock
        self.pair = self.browser.tab()
        self.rows = []

    def snapshot(self, step, state, page, why='', verdict=None, way=None):
        state=getattr(self,'state_prefix','')+state
        stem = f'{step:02}-{state}-{self.width}'
        for kind, tab in [('mock', self.m), ('client', self.c)]:
            heading={'goal-tasks':'Tasks','goal-task-rows':'Tasks','goal-landing':'Landing','goal-how-it-runs':'How it runs'}.get(state)
            if heading:
                tab.js("[...document.querySelectorAll('h2,h3')].find(e=>e.innerText.toLowerCase()==="+json.dumps(heading.lower())+")?.scrollIntoView({block:'center'})")
            tab.shot(self.out / f'{stem}-{kind}.png', reset=not bool(heading))
            metrics = tab.js("""[...document.querySelectorAll('h1,h2,h3,main,nav,aside,button,textarea,dialog,[role=dialog],.headline,.col,.rail,.intro,.flow-q,.temper-doc,.temper-session,.temper-turn-tool,.turn-tool')].filter(e=>e.getClientRects().length).map(e=>{const b=e.getBoundingClientRect(),s=getComputedStyle(e);return {tag:e.tagName,cls:e.className,text:e.innerText?.slice(0,120),x:b.x,y:b.y,width:b.width,height:b.height,font:s.fontFamily,size:s.fontSize,weight:s.fontWeight,line:s.lineHeight,background:s.backgroundColor,padding:s.padding,gap:s.gap,margin:s.margin,position:s.position,inset:s.inset,outline:s.outlineStyle}})""")
            (self.out / f'{stem}-{kind}.json').write_text(json.dumps({'text':tab.text(),'metrics':metrics,'controls':tab.controls()}, indent=2))
        self.pair.call('Emulation.setDeviceMetricsOverride', width=2*self.width, height=1000, deviceScaleFactor=1, mobile=False)
        sources = ['data:image/png;base64,'+base64.b64encode((self.out / f'{stem}-{kind}.png').read_bytes()).decode() for kind in ['mock','client']]
        self.pair.js("document.body.style='margin:0;display:flex';document.body.replaceChildren();")
        self.pair.js("Promise.all("+json.dumps(sources)+".map(src=>new Promise(resolve=>{let i=new Image;i.onload=resolve;i.src=src;document.body.append(i)})))")
        self.pair.shot(self.out / f'{stem}-pair.png')
        row = dict(step=step,state=state,page=page,width=self.width,verdict=verdict or ('differs' if why else 'matches'),why=why or 'Same structure, controls and look; scripted data differs.',way=way or getattr(self,'way',f'box {self.box.number}'),pair=f'{stem}-pair.png')
        self.rows.append(row)
        (self.out / f'rows-{self.width}-{self.box.number}.json').write_text(json.dumps(self.rows,indent=2))
        print(f'{self.width} box {self.box.number}: {step} {state}',flush=True)

    def mock_menu(self, name):
        self.m.click('Mockup','button')
        self.m.click(name,'link')

    def mock_nav(self, name):
        controls=self.m.controls()
        if any(x['name']=='Back' for x in controls):
            self.m.click('Back','link');controls=self.m.controls()
        if not any(x['tag']=='A' and x['name'].strip()==name for x in controls):
            if any(x['tag']=='A' and x['name']=='Default' for x in controls):
                self.m.click('Default','link')
            elif name=='Home':
                self.m.click('Chats','link')
            elif name=='Settings':
                summary=next(x['name'] for x in controls if x['tag']=='SUMMARY')
                self.m.click(summary,'button')
                self.m.click('Settings for '+summary,'link');return
        name=next((x['name'].strip().replace('\n',' ') for x in self.m.controls() if x['tag']=='A' and (x['name']==name or x['name'].startswith(name+'\n') or x['name'].startswith(name+' '))),name)
        self.m.click(name,'link')

    def nav(self, name):
        if self.width==420 and name=='Settings' and any(x['tag']=='A' and x['name']=='Settings' for x in self.c.controls()):
            self.c.click('Settings','link');return
        if self.width==420 and not any(x['name']=='Home' for x in self.c.controls()):
            back='Chats' if any(x['tag']=='A' and x['name']=='Chats' for x in self.c.controls()) else 'Settings'
            self.c.click(back,'link')
        if self.width == 420 and name == 'Settings':
            summary=next(x['name'] for x in self.c.controls() if x['tag']=='SUMMARY')
            self.c.click(summary,'button')
            self.c.click('Settings for '+summary,'link')
        else:
            if self.width == 420 and name == 'Changes':
                self.c.click('Home','link')
                if any(x['name']=='Skip' for x in self.c.controls()):self.c.click('Skip','button')
            name=next((x['name'].strip().replace('\n',' ') for x in self.c.controls() if x['tag']=='A' and (x['name']==name or x['name'].startswith(name+' '))),name)
            self.c.click(name,'link')

    def first(self, evidence=True, settle=True):
        self.c.go(self.box.link)
        self.c.wait('Set up temper')
        self.m.go((ROOT / 'temper/docs/design/client/greenfield/welcome.html').as_uri())
        if 'Start the first run' in self.m.text():
            self.m.click('Start the first run','button')
        else:
            self.m.click('Mockup','button')
            self.m.click('Start the first run over','button')
        self.m.wait('Set up temper')
        if evidence:self.snapshot(1,'welcome','Welcome')
        self.c.click('Set up temper','link');self.m.click('Set up temper','link')
        if evidence:self.snapshot(1,'forge','Forge')
        self.c.click('Run on this machine','link');self.m.click('Run on this machine','button')
        if evidence:self.snapshot(1,'forge-empty-hello','Forge')
        self.c.click('Run on this machine','button');self.m.click('Run on this machine','button')
        self.c.wait('Continue with ChatGPT');self.m.wait('Continue with ChatGPT')
        if evidence:self.snapshot(1,'model-offers','Model','By page, First run: the forge pane stays on Model.')
        self.c.click('Use your Claude subscription','link');self.m.click('Use your Claude subscription','link')
        if evidence:self.snapshot(1,'model-claude','Model','By page, First run: Claude alert and fetch sentence.')
        self.c.click('All ways','link');self.m.click('All ways','link')
        self.c.click('Continue with ChatGPT','button');self.m.click('Continue with ChatGPT','button')
        self.c.ready('Create your project');self.m.ready('Create your project')
        if evidence:
            self.snapshot(1,'model-chatgpt-signed-in','Model','Fake issuer returns to Ready without credentials; ChatGPT plan is usable.')
            self.snapshot(1,'ready','Ready','By page, First run: Hello and ChatGPT plan, no email; fake forge offers no browser sign-in.')
        self.c.click('Create your project','button');self.m.click('Create your project','button')
        self.c.wait('To begin');self.m.wait('To begin')
        if settle:
            self.c.wait('work/hello is in Hello');time.sleep(1)
        if evidence:self.snapshot(1,'home-to-begin','Home','By page, First run: ready headline, no Connected; W2 acceptance copy fixed.')

    def settings(self, evidence=True):
        self.nav('Settings');self.mock_nav('Settings')
        self.c.wait('Chats can write drafts')
        if evidence:self.snapshot(2,'settings-how-work-runs','Settings','D11; By page, Settings: Configure temper first.')
        try:
            self.c.ready('Chats can write drafts',seconds=8)
        except RuntimeError:
            self.snapshot(2,'settings-switches-stalled','Settings','W1: feature switches remain disabled.')
            self.box.login_ctl()
            inventory=json.loads(self.box.ctl('settings','show','--json'))
            file=self.box.path/'ways.json'
            file.write_text(json.dumps({'items':[{'setting':{'kind':'project-ways','project':1},'set':{'drafts':True,'goals':True,'changes':True,'away':True,'way':0}}]}))
            self.box.ctl('settings','apply',str(file),'--against',str(inventory['newest']),'--key','walk-ways')
            self.way=f'box {self.box.number}; ctl project-ways after W1'
            self.c.click('Home','link'); self.nav('Settings')
        if evidence:self.snapshot(2,'settings-read-fast-path','Settings','W1: four switches available on the fast first-run path; no ctl fallback.' if not hasattr(self,'way') else 'Settings needed ctl fallback; see stalled row.')
        for label in ['Chats can write drafts','Goals','Changes','Away switch']:
            self.c.ready(label)
            if not self.c.js("[...document.querySelectorAll('input[type=checkbox]')].find(e=>(e.getAttribute('aria-label')==="+json.dumps(label)+"||[...e.labels||[]].some(l=>l.innerText.trim()==="+json.dumps(label)+"))).checked"):
                self.c.click(label,'checkbox')
            time.sleep(.4)
        # The mock labels the four checkboxes with its page's words.
        for label in ['Chats can write drafts','Goals','Changes','Away switch']:

            if not self.m.js("[...document.querySelectorAll('input[type=checkbox]')].find(e=>(e.getAttribute('aria-label')==="+json.dumps(label)+"||[...e.labels||[]].some(l=>l.innerText.trim()==="+json.dumps(label)+"))).checked"):
                self.m.click(label,'checkbox')
        if evidence:self.snapshot(2,'rail-features-on','Settings','D11; Ruling 9; By page, Phone: Changes is absent from tabs.')

    def send(self, words, fresh=True):
        if fresh:
            self.nav('Chats')
            self.c.click('+ New chat','link')
        controls=self.c.controls()
        field=next(x['name'] for x in controls if x['tag']=='TEXTAREA')
        self.c.fill(field,words)
        self.c.click('Send','button')

    def change(self, before_home=None):
        self.send('Land a local hello status change.')
        self.c.wait('Land a local hello status change.')
        self.mock_menu('Chats');self.m.click('+ New chat','link')
        self.m.fill('What do you want done?','Add a local status line to README.md.')
        self.m.click('Send','button')
        self.snapshot(3,'chat-change','Chat')
        time.sleep(1)
        self.snapshot(3,'tool-lines','Chat')
        self.nav('Changes');self.everyday_mock();self.mock_menu('Changes')
        self.snapshot(3,'changes-queue','Changes')
        # Landing can be held by a real setup fault; record it instead of hiding it.
        self.c.wait('Record the local hello status change',120)
        queued = 'main last moved' not in self.c.text()
        if queued:
            self.snapshot(3,'changes-queued-row','Changes','By page, Changes: no green time; live queued row.')
        self.c.wait('main last moved',120)
        self.snapshot(3,'changes-landed','Changes')
        if not queued:
            self.gallery(3,'changes-queued-row','changes-one','The local change landed before Queue could be observed nonempty; gallery shows a queued row.')
        if before_home:before_home()
        self.nav('Home');self.mock_nav('Home')
        if any(x['name']=='Skip' for x in self.c.controls()): self.c.click('Skip','button')
        self.snapshot(3,'home-after-change','Home','D2; By page, Home: brief waits for a new sitting.')
        self.box.login_ctl()
        (self.out/f'landing-project-{self.width}-{self.box.number}.json').write_text(self.box.ctl('project','show','1','--json'))
        (self.out/f'landing-accounts-{self.width}-{self.box.number}.json').write_text(self.box.ctl('account','list','--json'))
        self.left_home=time.time()
        (self.out/f'absence-{self.width}-{self.box.number}.json').write_text(json.dumps({'left_home':self.left_home,'sitting_ms':3000 if self.box.number==6 else 1800000}))
        self.home_url=self.c.js('location.href')
        self.browser.close_tab(self.c)
        self.c=self.browser.tab()

    def configure_reference(self):
        # The written look request is the reference on this trunk. Its old
        # journey commit is absent here; the mock remains the independent pair.
        self.snapshot(2,'configure-look-request','Configure',
                      'client/README.md, 3.5, The configuring chat’s page: retained rail, introduction and temper’s own way; the greenfield mock has the earlier chat starter.')

    def configure(self):
        self.nav('Settings'); self.c.click('Start','link')
        self.mock_nav('Home'); self.m.click('Start','link')
        self.snapshot(2,'configure-temper','Configure','By page, Settings: full page; configure-chat look request.')
        self.configure_reference()
        self.c.fill('Describe how you work, or what to change','I write a design draft first; agents plan its tasks and land them together once checked.')
        self.c.click('Send','button'); self.mock_send('I write a design draft first; agents plan its tasks and land them together once checked.'); self.c.wait('Hello from the local fake model.')
        # Keep the first draft chat as task 1 on its separate fresh box.
        self.snapshot(2,'configure-answer','Configure','The configuring fake has no settings/document script; gallery configure-chat-proposal is shown in step 5.',verdict='unreachable')

    def mock_send(self, words):
        if not any(x['tag']=='TEXTAREA' for x in self.m.controls()):self.m.click('Chat','button')
        field=next(x['name'] for x in self.m.controls() if x['tag']=='TEXTAREA')
        self.m.fill(field,words); self.m.click('Send','button'); time.sleep(1.3)

    def draft(self):
        self.send('Let us design Away mode.')
        self.c.wait('Hello from the local fake model.')
        self.m.click('Chats','link');self.m.click('+ New chat','link')
        self.mock_send('Let us design Away mode.')
        self.snapshot(4,'new-chat','Chat','D7: the rail stays on a chat.')
        self.c.click('+ Draft','button');self.m.click('+ Draft','button')
        self.c.wait('docs/design/draft/away-mode.md');self.mock_send('Write this up as a draft beside us.')
        if self.width==420:self.m.click('Draft','button')
        self.snapshot(4,'draft-first-turn','Chat and draft','D7; By page, Phone: no Split at 420; W4 heading rhythm fixed.')
        self.c.click('Chat','button');self.m.click('Chat','button')
        self.send('Add what happens to a running goal.',False);self.mock_send('Add what happens to a running goal.')
        self.c.wait('Edited');self.snapshot(4,'draft-second-turn','Chat','D7: rail stays.')
        self.working_notes()
        if self.width==1440:
            self.c.click('Split','button');self.m.click('Split','button')
            self.snapshot(4,'split','Chat and draft','D7: rail folds to a strip; W4 heading rhythm fixed.')
        else:
            self.snapshot(4,'split','Chat and draft','By page, Phone: Split intentionally absent; Draft shows the document.',verdict='unreachable')
        self.c.click('Draft','button');self.m.click('Draft','button')
        self.snapshot(4,'draft','Draft','By page, A chat and its draft: composer stays on white; W4 heading rhythm fixed.')
        self.c.click('Start goal…','button');self.m.click('Start goal…','button')
        self.snapshot(4,'start-goal-sheet','Start goal','Rulings 3–8 and 10; By page, Goals: offered ways only; Budgets.')
        self.c.fill('Name','walk-away-mode');self.m.fill('Name','walk-away-mode')
        self.c.click('Start goal','button');self.m.click('Start goal','button')
        self.c.wait('The local goal is waiting for direction.')
        self.snapshot(4,'goal-report','Goal','D8: goal-owned report; scripted goal waits for direction.')
        self.snapshot(4,'goal-tasks','Goal','Script has no delegates: empty task section; gallery goal-running shows task rows.',verdict='unreachable')
        self.snapshot(4,'goal-landing','Goal','By page, A goal: no branch-update line.')
        self.snapshot(4,'goal-how-it-runs','Goal','Ruling 10; By page, Goals: available ways only.')
        self.c.click('Pause','button');self.m.click('Pause','button')
        self.c.wait('Resume');self.snapshot(4,'goal-paused','Goal','D4: Pause stops running tasks immediately.')
        self.c.click('Resume','button');self.m.click('Resume','button')
        self.c.wait('Pause');self.snapshot(4,'goal-resumed','Goal')
        self.c.click('Stop','button');self.m.click('Stop','button')
        self.snapshot(4,'goal-stop-sheet','Goal')
        self.c.click('Stop goal','button')
        self.c.wait('stopped');self.snapshot(4,'goal-stopped','Goal')
        if any(x['name']=='Draft' and x['tag']=='A' for x in self.c.controls()):
            self.c.click('Draft','link'); why='By page, A chat and its draft: reopen in Chat.'
        else:
            self.nav('Chats'); self.c.click('T1','link'); why='Draft link missing; returned via Chats → T1. Review this state.'
        self.m.click('Draft','link')
        self.snapshot(4,'draft-back-to-chat','Chat',why)
        if self.width==1440:
            self.c.click('More ways to start work','button')
            # The mock has no corresponding menu; its New chat is the nearest baseline.
            self.snapshot(4,'rail-menu','Rail','By page, A chat and its draft: held rail menu; Goal and All ways.')
            self.c.call('Input.dispatchKeyEvent',type='keyDown',key='Escape',code='Escape')
        else:
            self.snapshot(4,'rail-menu','Rail','By page, Phone: rail hidden; Chats and its + New chat replace it.',verdict='unreachable')

    def working_notes(self):
        self.send('Keep only working notes for this chat.', False)
        self.c.wait('Only working notes remain.')
        self.nav('Chats')
        self.c.click('T1', 'link')
        if any(x['name'] == 'Draft' for x in self.c.controls()):
            raise RuntimeError('W7: working notes were offered as the chat document')
        self.snapshot(4, 'working-notes-only', 'Chat', 'W7 fixed: only work: notes offer no document; mock retains its document.')
        self.send('Restore the Away mode document after the working notes.', False)
        self.c.wait('The document follows the working notes.')
        self.nav('Chats')
        self.c.click('T1', 'link')
        self.c.click('Draft', 'button')
        self.c.wait('A running goal')
        self.box.login_ctl()
        notes = self.box.ctl('task', 'notes', '1', '--project', '1', '--json')
        (self.out / f'working-notes-{self.width}.json').write_text(notes)
        pages = [json.loads(notes)]
        while pages[-1].get('after'):
            pages.append(json.loads(self.box.ctl('task', 'notes', '1', '--project', '1', '--after', pages[-1]['after'], '--json')))
        (self.out / f'working-notes-all-{self.width}.json').write_text(json.dumps(pages, indent=2))
        self.m.click('Draft', 'button')
        self.snapshot(4, 'document-after-working-notes', 'Draft', 'W7 fixed: reopened document follows five lower-numbered work: notes, across a page boundary.')
        self.c.click('Chat', 'button')
        self.m.click('Chat', 'button')

    def gallery(self, step, state, name, why):
        old=self.c
        self.c=self.browser.tab()
        self.c.call('Emulation.setDeviceMetricsOverride',width=self.width,height=1000,deviceScaleFactor=1,mobile=False)
        self.c.go((self.out/'gallery'/f'{name}.html').as_uri())
        self.snapshot(step,state,'Gallery',why,verdict='unreachable',way=f'gallery {name}; box {self.box.number} attempt')
        self.c=old

    def stop(self):
        self.box.stop();self.browser.stop()

    def everyday_mock(self, away=False):
        if not any(x['name']=='Mockup' for x in self.m.controls()):
            self.m.click('Back','link')
        self.m.click('Mockup','button')
        self.m.click('Everyday: fabrig and hello','radio')
        if away:
            self.m.click('Mockup','button');self.m.click('Back from away','radio');self.m.click('Mockup','button')

    def needs(self):
        self.send('Start a local away goal.')
        self.c.wait('I started the local away goal.')
        self.nav('Home')
        self.c.wait('Which local implementation while here?')
        self.everyday_mock()
        self.snapshot(5,'home-needs-card','Home','D3: fixed question format; scripted question and counts.')
        self.nav('Needs you')
        self.mock_menu('Needs you')
        self.snapshot(5,'question-with-choices','Needs you')
        self.c.click('Later','button');self.m.click('Later','button')
        self.snapshot(5,'later','Needs you','By page, Needs you: Later lives only in the tab.')
        self.m.click('Later','button');self.m.click('Later','button')
        self.c.click('Reply…','button');self.m.click('Reply…','button')
        self.snapshot(5,'reply-open','Needs you')
        self.c.fill('Your reply','Use the small implementation')
        field=next(x['name'] for x in self.m.controls() if x['tag']=='TEXTAREA')
        self.m.fill(field,'Use the small implementation')
        self.c.click('Send reply','button');self.m.click('Send reply','button')
        # Capture Undo before the five-second send; the mock has the same action.
        self.snapshot(5,'reply-answered','Needs you','D5: five-second send delay; reply words restored on Undo.')
        self.c.click('Undo','button');self.m.click('Undo','button')
        self.snapshot(5,'undo','Needs you','D5; By page, Needs you: Undo restores reply words.')
        self.c.click('Use the small implementation','button')
        time.sleep(6)
        self.snapshot(5,'needs-count','Needs you','By page, Needs you: counts omit answered cards.')

    def away(self):
        self.nav('Home')
        if any(x['name']=='Skip' for x in self.c.controls()): self.c.click('Skip','button')
        self.c.click('Away' if self.width==1440 else 'Away: off','button')
        self.everyday_mock(True)
        self.snapshot(6,'away-on','Home','D10; By page, Away: implementation/retry/wording only.')
        self.send('Start a local away goal.')
        self.c.wait('I started the local away goal.')
        self.nav('Home');self.c.wait('Decided for you')
        self.snapshot(6,'goal-decided','Home','D10: decided tool; By page, Away: Home only, 24 hours.')
        self.snapshot(6,'decided-for-you','Home','D10; By page, Away: Home only, 24 hours.')
        self.c.click('Overturn','button');self.m.click('Overturn','button')
        self.c.wait('Which local implementation after overturn?')
        self.snapshot(6,'overturned','Home','By page, Away: overturn while the task lives.')

    def other_settings(self):
        self.rename_comparison()
        self.nav('Settings')
        self.mock_nav('Settings')
        for name in ['Repositories','Forges','Accounts','Builders']:
            self.c.click(name,'link');self.m.click(name,'link')
            self.snapshot(7,'settings-'+name.lower(),name,'By page, Settings: tab contents use the mock language; mock tab not drawn.')
        self.c.click('Hello','button')
        self.m.click('Default','button')
        self.snapshot(7,'project-switcher','Project switcher')
        self.c.fill('+ New project','Second walk project')
        self.c.call('Input.dispatchKeyEvent',type='keyDown',key='Enter',code='Enter',windowsVirtualKeyCode=13)
        self.c.call('Input.dispatchKeyEvent',type='keyUp',key='Enter',code='Enter',windowsVirtualKeyCode=13)
        self.c.wait('Second walk project')
        self.m.prompt='Second walk project'
        self.m.click('+ New project','button')
        self.snapshot(7,'project-added','Home','Mock + New project only explains name/Create in a toast; client creates the stock-policy project.',verdict='unreachable')
        self.c.click('Second walk project','button')
        control=next(x['name'].strip().replace('\n',' ') for x in self.c.controls() if x['tag']=='BUTTON' and x['name'].startswith('Hello'))
        self.c.click(control,'button')
        self.m.click('Default','button')
        self.mock_nav('Home')
        self.snapshot(7,'project-switched','Home')
        self.nav('Settings');self.c.click('Accounts','link')

        if self.width==1440:self.c.click('Owner','button')
        self.c.click('Sign out','menuitem' if self.width==1440 else 'button');self.c.wait('Signed out')
        self.snapshot(7,'signed-out','Sign out','By page, Settings: Sign out is in Accounts; mock has no sign-out page.')
        self.c.call('Page.reload');self.c.wait('Signed out')
        self.snapshot(7,'signed-out-reloaded','Sign out','Mock has no sign-out page.')
        # A spent printed link asks the box to print a fresh one, as its terminal explains.
        self.c.go(self.box.link)
        time.sleep(.5)
        links=re.findall(r'\n(http://[^\s]+)',self.box.logpath.read_text())
        self.c.go(links[-1]);self.c.wait('Chats')
        self.snapshot(7,'back-in','Chats','Back in by the fresh printed link; mock has no sign-out page.')

    def rename_comparison(self):
        self.nav('Home');self.mock_nav('Home')
        self.c.click('Hello','button');self.m.click('Default','button')
        self.c.click('Rename Hello','button');self.m.click('Rename Default','button')
        self.snapshot(7,'project-rename','Project switcher','A2 is owned by additions: selected-row rename form compared with mock.')
        for tab in [self.c,self.m]:
            for kind in ['keyDown','keyUp']:
                tab.call('Input.dispatchKeyEvent',type=kind,key='Escape',code='Escape',windowsVirtualKeyCode=27)
        self.snapshot(7,'rename-escape-focus','Project switcher','A3 is owned by additions: Escape closes the selected-row form.')
        # Cancel was removed by the accepted rename request; keep its old row explicit.
        self.snapshot(7,'rename-cancel-focus','Project switcher','Rename now cancels with Escape; visible Cancel intentionally removed by the rename request.',verdict='unreachable')
        for tab,name in [(self.c,'Hello'),(self.m,'Default')]:
            if any(x['name']=='Rename '+name for x in tab.controls()):tab.click(name,'button')

    def mock_proposal(self):
        self.m.click('Mockup','button');self.m.click('First run: a fresh temper box, real backends','radio')
        self.m.click('Mockup','button');self.m.click('Welcome','link')
        self.m.click('Set up temper','link');self.m.click('Run on this machine','button');self.m.click('Run on this machine','button')
        self.m.wait('Continue with ChatGPT');self.m.click('Continue with ChatGPT','button');self.m.ready('Create your project')
        self.m.click('Create your project','button');self.m.wait('To begin');self.m.click('Start','link')
        self.mock_send('I write a design draft first; agents plan its tasks and land them together once checked.')
        self.mock_nav('Home')
        link=next(x['name'].strip().replace('\n',' ') for x in self.m.controls() if x['tag']=='A' and x['name'].startswith('Needs you'))
        self.m.click(link,'link')

    def needs_tail(self):
        self.box.login_ctl()
        stopped=self.box.ctl('task','stop','1','--key','walk-held')
        (self.out/f'held-{self.width}.txt').write_text(stopped)
        self.nav('Needs you')
        self.m.click('Later','button')
        self.gallery(5,'held-task','needs-held','A ctl stop placed an owner hold; no temper-owned held card. Gallery shows the requested kind.')

    def unfunded_period(self):
        self.box.login_ctl()
        project=json.loads(self.box.ctl('project','show','1','--json'))
        cuts=[]
        for role in project['policy']['roles']:
            cuts.append(json.loads(self.box.ctl('project','fund','1','0','--role',str(role['number']),'--key',f'walk-unfunded-role-{role["number"]}','--json')))
        (self.out/f'unfunded-role-cuts-{self.width}.json').write_text(json.dumps(cuts,indent=2))
        (self.out/f'unfunded-period-cut-{self.width}.json').write_text(self.box.ctl('project','fund','1','0','--key','walk-unfunded','--json'))
        (self.out/f'unfunded-project-{self.width}.json').write_text(self.box.ctl('project','show','1','--json'))

    def unfunded(self):
        self.box.restart_with_sitting(3000)
        self.first(False,False)
        self.settings(False)
        self.state_prefix='unfunded-'
        self.change(before_home=self.unfunded_period)
        # Both landing and the last looked count precede this return.
        until=self.left_home+61
        while time.time()<until:
            print('Unfunded Home: waiting for a minute-old landing.',flush=True)
            time.sleep(min(25,max(.1,until-time.time())))
        self.c.go(self.home_url)
        self.c.wait('The last change landed')
        time.sleep(3)
        text=self.c.text()
        headline=self.c.js("document.querySelector('h1')?.innerText")
        if not re.fullmatch(r'The last change landed .+ ago\. Nothing needs you\.',headline or ''):
            raise RuntimeError(f'Unexpected unfunded headline: {headline}')
        if 'Written by the status agent' in text or 'The README now records' in text:
            raise RuntimeError('The unfunded box unexpectedly got a brief')
        self.state_prefix=''
        self.snapshot(3,'home-unfunded','Home','Unfunded period via ctl project fund 1 0 (box cannot add a second member); forge.sitting_ms=3000, computed sentence and no byline/error. Mock draws a funded brief.')

    def proposal(self):
        # The literal fake revision is valid only before the first settings edit.
        self.nav('Settings');self.c.click('Start','link')
        self.mock_nav('Home');self.m.click('Start','link')
        self.c.fill('Describe how you work, or what to change','Show drafts and goals for this project.')
        self.c.click('Send','button')
        self.mock_send('I write a design draft first; agents plan its tasks and land them together once checked.')
        self.c.wait('I proposed showing drafts and goals.')
        self.c.wait('Accept')
        self.chat_url=self.c.js('location.href')
        self.snapshot(5,'configure-proposal','Configure','W3: live configuring proposal; By page, Settings: card omits enforced/follow wording.')
        self.nav('Needs you');self.mock_nav('Home');self.mock_nav('Needs you')
        self.c.wait('Accept')
        self.snapshot(5,'proposal','Needs you','W3 fixed: project proposal, count 1; scripted settings differ.')
        self.box.login_ctl()
        (self.out/f'inbox-proposal-{self.width}.json').write_text(self.box.ctl('inbox','--json'))
        self.nav('Home');self.mock_nav('Home')
        self.c.wait('Accept')
        self.snapshot(5,'home-proposal-card','Home','W3: proposal on Home; To begin keeps its full card at either width.')
        self.nav('Needs you');self.mock_nav('Needs you')
        self.c.click('Accept','button')
        self.c.wait('Accept this proposal?')
        self.snapshot(5,'proposal-accept-sheet','Needs you','The sheet look request: Not yet / Accept; mock accepts without a sheet.')
        self.c.click('Accept','button');self.m.click('Accept','button')
        self.c.wait('Accepted')
        self.snapshot(5,'proposal-accepted-count','Needs you','W3: count cleared; Drafts and Goals now available.')
        (self.out/f'inbox-accepted-{self.width}.json').write_text(self.box.ctl('inbox','--json'))
        self.nav('Chats');self.mock_nav('Chats')
        self.c.wait('Drafts')
        self.snapshot(5,'proposal-accepted-ways','Chats','W3: Goals in navigation and Drafts filter in Chats; By page, A chat and its draft: rail menu offers Goal, not a separate Drafts rail entry.')
        self.c.click('T1','link')
        link=next(x['name'].strip().replace('\n',' ') for x in self.m.controls() if x['tag']=='A' and 'Tell temper how you work' in x['name'])
        self.m.click(link,'link')
        self.c.wait('accepted')
        self.snapshot(5,'configure-proposal-accepted','Configure','W3: original chat card accepted; By page, Settings: proposal wording.')




def write_index(out):
    rows={}
    for file in sorted(out.glob('rows-*.json')):
        for row in json.loads(file.read_text()):rows[(row['step'],row['state'],row['width'])]=row
    lines=['<!doctype html><meta charset="utf-8"><title>Client walk</title>',
           '<style>body{font:15px system-ui;margin:32px;background:#f7f7f5}td,th{padding:8px;text-align:left;vertical-align:top}table{border-collapse:collapse}tr{border-bottom:1px solid #ddd}img{max-width:480px}a{color:#333}</style>',
           '<h1>Mock / client</h1><p>Mock at left. <a href="report.md">Report</a>. Each state links its two originals and pair; JSON neighbours contain text, styles and rectangles.</p>',
           '<table><tr><th>Step / state / width<th>Verdict / why / way<th>Images']
    for key,r in sorted(rows.items()):
        pair=r['pair'];stem=pair.removesuffix('-pair.png')
        links=' · '.join(f'<a href="{stem}-{kind}.png">{kind}</a>' for kind in ['mock','client','pair'])
        lines.append(f'<tr><td>{r["step"]} · {html.escape(r["state"])} · {r["width"]}<td>{html.escape(r["verdict"]+": "+r["why"])}<br>{html.escape(r["way"])}<td>{links}<br><a href="{pair}"><img loading="lazy" src="{pair}"></a>')
    lines.append('</table>')
    (out/'index.html').write_text('\n'.join(lines))
    return list(rows.values())


def main():
    import hashlib
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--width',type=int,choices=[1440,420],required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--bundle',type=Path,required=True)
    parser.add_argument('--gallery',type=Path,required=True)
    parser.add_argument('--only',choices=['unfunded','proposal','draft','needs','away','settings'],help='Replay a focused supplemental path on one fresh box.')
    parser.add_argument('--brief-from',type=Path,help='At 420, reuse the first walk’s live 1440-box capture at 420; wait for it if still being taken.')
    args=parser.parse_args()
    os.umask(0o077)
    out=args.out.expanduser().resolve()
    if (out/'boxes').exists() or (out/'build').exists():raise SystemExit('Use a fresh output folder; no retained box is overwritten.')
    prepare(out)
    shutil.copytree(args.bundle.resolve(),out/'build/bundle')
    shutil.copytree(args.gallery.resolve(),out/'gallery')
    (out/'build.json').write_text(json.dumps({'client_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'builder':socket.gethostname(),'fake_source_sha256':hashlib.sha256((ROOT/'temper/crates/temper-fake/src/backends.rs').read_bytes()).hexdigest(),'started_utc':time.strftime('%Y-%m-%d %H:%M:%S UTC',time.gmtime())},indent=2))
    walks=[]
    def start(number):
        w=Walk(out,args.width,number);walks.append(w);w.first(number==1, settle=False);w.settings(number==1);return w
    def finish(w):
        w.stop()
        durations.append({'box':w.box.number,'seconds':round(time.time()-w.started,3)})
    durations=[]
    try:
        if args.only:
            w=Walk(out,args.width,6 if args.only=='unfunded' else 5);walks.append(w)
            if args.only=='unfunded':w.unfunded()
            else:
                w.first(False,False)
                if args.only=='proposal':w.proposal()
                else:
                    w.settings(False)
                    if args.only=='draft':w.draft()
                    elif args.only=='needs':w.needs();w.needs_tail()
                    elif args.only=='away':w.away()
                    elif args.only=='settings':w.other_settings()
            finish(w)
            rows=write_index(out)
            (out/'run.json').write_text(json.dumps({'boxes':durations,'rows':len(rows),'finished_utc':time.strftime('%Y-%m-%d %H:%M:%S UTC',time.gmtime())},indent=2))
            return
        landing=start(1);landing.change()
        errors=[]
        def stage(number, action, settings=True):
            w=Walk(out,args.width,number);walks.append(w)
            try:
                w.first(False,False)
                if settings:w.settings(False)
                action(w)
            except Exception as error:
                errors.append({'box':number,'error':str(error)})
                (out/'errors.json').write_text(json.dumps(errors,indent=2))
                print(f'Box {number} stopped its path: {error}',flush=True)
            finally:finish(w)
        stage(5,lambda w:w.proposal(),False)
        def draft_path(w):
            w.draft()
            w.m.click('Goal walk-away-mode →','link')
            w.gallery(4,'goal-task-rows','goal-running','Scripted goal has no delegated tasks; gallery shows their rows.')
            w.m.click('Draft','link')
            w.configure();w.other_settings()
        stage(2,draft_path)
        stage(3,lambda w:(w.needs(),w.needs_tail()))
        stage(4,lambda w:w.away())
        unfunded=Walk(out,args.width,6);walks.append(unfunded)
        try:unfunded.unfunded()
        except Exception as error:
            errors.append({'box':6,'error':str(error)})
            (out/'errors.json').write_text(json.dumps(errors,indent=2))
            print(f'Box 6 stopped its path: {error}',flush=True)
        finally:finish(unfunded)
        if args.brief_from:
            if args.width!=420:raise RuntimeError('--brief-from is for the second width only')
            stem='03-home-brief-420'
            source=args.brief_from.expanduser().resolve()
            while not (source/f'{stem}-pair.png').exists() or not any(r['state']=='home-brief' and r['width']==420 for f in source.glob('rows-*.json') for r in json.loads(f.read_text())):
                print('Waiting for the first box’s live 420 Home brief capture.',flush=True);time.sleep(25)
            for kind in ['mock','client','pair']:
                shutil.copy2(source/f'{stem}-{kind}.png',out/f'{stem}-{kind}.png')
            for kind in ['mock','client']:
                shutil.copy2(source/f'{stem}-{kind}.json',out/f'{stem}-{kind}.json')
            source_rows=[r for f in source.glob('rows-*.json') for r in json.loads(f.read_text()) if r['state']=='home-brief' and r['width']==420]
            if not source_rows:raise RuntimeError('The brief source has no reviewed state row')
            row=source_rows[-1].copy();row['way']='first-width live box, same state resized to 420; see --brief-from source'
            landing.rows.append(row)
            if row['verdict']=='unreachable':landing.gallery(3,'home-brief-gallery','home-morning','No brief on the source box; gallery shows the requested state.')
            (out/f'rows-{args.width}-1.json').write_text(json.dumps(landing.rows,indent=2))
        else:
            until=landing.left_home+31*60
            while time.time()<until:
                print(f'Home brief: {max(0,int(until-time.time()))} seconds left away.',flush=True)
                time.sleep(min(25,max(.1,until-time.time())))
            landing.c.go(landing.home_url)
            landing.c.wait('Landing:')
            try:landing.c.wait('The README now records',seconds=20)
            except RuntimeError:pass
            text=landing.c.text()
            verdict=None if all(words in text for words in ['1 change landed on the local forge.','The README now records the local status change.','Written by the status agent']) else 'unreachable'
            (out/'brief-return.json').write_text(json.dumps({'left_home':landing.left_home,'returned':time.time(),'absent_seconds':time.time()-landing.left_home,'sitting_ms':1800000}))
            why='D2: returned after 31 minutes.' if verdict is None else 'After 31 minutes Home retained its computed sentence; no brief was delivered. See the status/held state in the capture.'
            landing.snapshot(3,'home-brief','Home',why,verdict=verdict)
            if verdict:landing.gallery(3,'home-brief-gallery','home-morning','No brief was delivered after 31 minutes; gallery shows the requested brief.')
            if args.width==1440:
                landing.width=420
                for tab in [landing.c,landing.m]:tab.call('Emulation.setDeviceMetricsOverride',width=420,height=1000,deviceScaleFactor=1,mobile=False)
                landing.snapshot(3,'home-brief','Home',why,verdict=verdict,way='same first-width box, same brief state resized to 420')
        finish(landing)
        rows=write_index(out)
        counts={verdict:sum(r['verdict']==verdict for r in rows) for verdict in ['matches','differs','unreachable']}
        (out/'run.json').write_text(json.dumps({'boxes':durations,'errors':errors,'verdicts':counts,'finished_utc':time.strftime('%Y-%m-%d %H:%M:%S UTC',time.gmtime())},indent=2))
        print(json.dumps(counts),flush=True)
        if errors:raise SystemExit('Some paths stopped; see errors.json and replay the affected path.')
    finally:
        for w in reversed(walks):w.stop()
        write_index(out)
        shutil.rmtree(out/'boxes',ignore_errors=True)
        shutil.rmtree(out/'build',ignore_errors=True)


if __name__=='__main__':
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM,interrupted)
    main()
