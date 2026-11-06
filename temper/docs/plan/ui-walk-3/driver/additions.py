#!/usr/bin/env python3
"""Walk additions on fake boxes; no client fixes and no fake-script edits.

Build as driver/README.md says, then run on the builder:
  python3 -B temper/docs/plan/ui-walk-3/driver/additions.py --section a4 \
    --out "$HOME/src/fabrig/ui-walk-3/additions" \
    --bundle "$PWD/target/ui-walk-3-bundle" --gallery "$PWD/target/ui-walk-3-gallery"
Sections: a4, rename, scroll, chat-work, close-chat, model, own-way, all.
Each section catches faults independently; rerun one with a fresh --out.
For narrow close-chat replays repeat --case NAME; --calls-only on chat-work
records native gallery disclosures after its live state was attempted.
--gallery-only on model replays the remaining named Model and paste states.
--sheet-look-only on close-chat measures the modal against mock dialog.sheet.
A4 uses ten fresh boxes (seven desktop, three phone); other sections both
widths, except scroll (1280x720, 1000x600). Evidence and index remain; copied
builds, private box directories and Chromium profiles are removed at exit.
References in screenshots are either the mock or the design's named words;
static gallery proofs never claim that an act changed a real box.
"""
import argparse
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

from walk import ROOT, Walk, prepare
from focus import focused, key

SECTIONS = ['a4', 'rename', 'scroll', 'chat-work', 'close-chat', 'model', 'own-way']


def save(path, value):
    path.write_text(json.dumps(value, indent=2))


def has(tab, name):
    return any(c['name'].strip() == name for c in tab.controls())


def press(tab, name, role='button'):
    """A real pointer press with no post-press delay; return release time."""
    point = tab.js("""(()=>{let roles={BUTTON:'button',A:'link',SUMMARY:'button'};
      let e=[...document.querySelectorAll('button,a,summary')].find(e=>
      (roles[e.tagName]===ROLE)&&(e.getAttribute('aria-label')||e.innerText).trim().replace(/\\s+/g,' ')===NAME
      &&e.checkVisibility({checkVisibilityCSS:true})&&!e.disabled);
      if(!e)throw Error('Missing press '+NAME);e.scrollIntoView({block:'center'});
      let r=e.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}})()""".replace('ROLE', json.dumps(role)).replace('NAME', json.dumps(name)))
    for kind in ['mousePressed', 'mouseReleased']:
        tab.call('Input.dispatchMouseEvent', type=kind, **point, button='left', clickCount=1)
    return time.monotonic()


class Addition(Walk):
    def __init__(self, out, width, number, section):
        super().__init__(out, width, number)
        self.section = section
        self.prefix = f'{section}-b{number}'
        self.mock_page = self.m
        for tab in [self.c,self.m]:
            tab.text=lambda tab=tab:tab.js("document.body?.innerText || ''")

    def nav(self,name):
        controls=self.c.controls()
        links=[c['name'] for c in controls if c['tag']=='A']
        if self.width==420 and name!='Settings' and 'Home' not in links and 'Chats' not in links and 'Settings' in links:
            self.c.click('Settings','link')
        super().nav(name)

    def snap(self, state, page, why='', verdict='matches', way=None):
        super().snapshot(8, self.prefix+'-'+state, page, why, verdict, way)
        self.rows[-1]['step'] = self.section
        save(self.out / f'rows-{self.width}-{self.box.number}.json', self.rows)

    def stop(self):
        started=time.monotonic()
        warning=None
        try:
            self.box.stop()
        except RuntimeError as error:
            warning=str(error)
        finally:
            self.browser.stop()
            save(self.out / f'cleanup-{self.box.number}.json', {
                'warning':warning,'exit':self.box.process.poll(),
                'stopped_marker':'Stopped' in self.box.logpath.read_text(),
                'chromium_exit':self.browser.process.poll(),
                'stop_seconds':round(time.monotonic()-started,3)})

    def words(self, title, words):
        if self.m is self.mock_page:self.m=self.browser.tab()
        self.m.go('about:blank')
        self.m.js("document.body.innerHTML="+json.dumps('<style>body{font:16px system-ui;margin:32px;line-height:1.6;max-width:720px}</style><h1>'+html.escape(title)+'</h1><p>'+html.escape(words)+'</p>'))

    def gallery_state(self, state, name, why, reference=None):
        live = self.c
        try:
            self.c = self.browser.tab()
            self.c.go((self.out / 'gallery' / (name+'.html')).as_uri())
            if reference:
                self.words(*reference)
            self.snap(state, 'Gallery', why, 'unreachable', f'gallery {name}; box {self.box.number}')
            sheet=sheet_metrics(self.c)
            if sheet:save(self.out/f'gallery-{name}-{self.width}-sheet.json',sheet)
            return {'text': self.c.text(), 'controls': self.c.controls(), 'sheet':sheet}
        finally:
            self.c = live


def a4(w):
    w.first(False, False)
    w.settings(False)
    w.send('Start a local away goal.')
    w.c.wait('I started the local away goal.')
    w.nav('Needs you')
    w.c.wait('Which local implementation while here?')
    w.everyday_mock()
    w.mock_menu('Needs you')
    w.snap('question', 'Needs you')
    w.c.click('Later', 'button')
    w.c.click('Reply…', 'button')
    w.c.fill('Your reply', 'Use the small implementation')
    w.c.click('Send reply', 'button')
    w.c.js("window.answerWalk=[];new MutationObserver(()=>window.answerWalk.push({at:performance.now(),text:document.body.innerText})).observe(document.body,{subtree:true,childList:true,characterData:true})")
    undo = press(w.c, 'Undo')
    choice = press(w.c, 'Use the small implementation')
    # No screenshot, mock action, ctl or intentional wait between these presses.
    immediate = w.c.text()
    time.sleep(max(0, 5.5-(time.monotonic()-choice)))
    after = w.c.text()
    events = w.c.js('window.answerWalk')
    w.box.login_ctl()
    inbox = json.loads(w.box.ctl('inbox', '--json'))
    pages=[inbox]
    seen=set()
    while pages[-1].get('read_on'):
        cursor=pages[-1]['read_on'].split('--after ',1)[-1]
        if cursor in seen:raise RuntimeError('Inbox cursor repeated')
        seen.add(cursor)
        pages.append(json.loads(w.box.ctl('inbox','--after',cursor,'--json')))
    save(w.out / f'a4-{w.box.number}-inbox-pages.json',pages)
    tasks = w.box.ctl('task', 'show', '2', '--json')
    save(w.out / f'a4-{w.box.number}-inbox.json', inbox)
    (w.out / f'a4-{w.box.number}-goal.json').write_text(tasks)
    result = {'box':w.box.number,'width':w.width,'undo_to_choice_seconds':round(choice-undo,6),
              'immediate':immediate,'after':after,'events':events,'inbox':inbox}
    save(w.out / f'a4-{w.box.number}.json', result)
    w.words('Needs you — answered', 'After the five-second Undo hold the card reads answered. Its question and count have gone. Every pressed answer sends. A retry toast counts as not sent. Reference: client/README.md, 3.5; notes/decisions.md, D5.')
    w.snap('after-hold', 'Needs you', 'A4; see measured interval and public inbox JSON. Verdict reviewed after the run.')


def focus_sample(tab):
    return {'focused':focused(tab),'targets':tab.js("[...document.querySelectorAll('main,h1')].map(e=>({tag:e.tagName,outline:getComputedStyle(e).outlineStyle,width:getComputedStyle(e).outlineWidth}))")}


def rename(w):
    w.first(False, False)
    results = {}
    name = 'Hello'
    w.c.click(name, 'button')
    w.c.click('Rename '+name, 'button')
    w.m.click('Default', 'button')
    w.m.click('Rename Default', 'button')
    results['form'] = w.c.js("[...document.querySelectorAll('input,button,form')].filter(e=>e.checkVisibility({checkVisibilityCSS:true})).map(e=>({tag:e.tagName,name:e.getAttribute('aria-label')||e.innerText,cls:e.className,height:e.getBoundingClientRect().height,minHeight:getComputedStyle(e).minHeight,width:e.getBoundingClientRect().width}))")
    w.snap('rename-form', 'Project switcher', 'A2: selected row becomes the form; primary Save; no Cancel or Rename act.')
    for action, value in [('save','Saved project'),('enter','Entered project'),('escape','Discarded project'),('empty','')]:
        if action != 'save':
            if not has(w.c,'Rename '+name):w.c.click(name,'button')
            w.c.click('Rename '+name,'button')
        field = 'Project name' if has(w.c, 'Project name') else 'Rename'
        w.c.fill(field,value)
        if action in ['save','empty']: w.c.click('Save','button')
        else:key(w.c,'Enter' if action=='enter' else 'Escape',13 if action=='enter' else 27)
        if action in ['save','enter']:name=value
        immediate_focus=focus_sample(w.c)
        time.sleep(.4)
        results[action] = focus_sample(w.c)
        results[action]['immediate']=immediate_focus
        results[action]['controls'] = w.c.controls()
        w.words('Rename — '+action, 'Save and Enter save; Escape cancels; an empty name keeps the old one. Focus returns to the opener or switcher summary. Main and h1 have no outline; the next Tab gives a real control its ring. Reference: notes/decisions.md, final section; greenfield/app.js, 229–240.')
        w.snap('rename-'+action,'Project switcher')
        key(w.c,'Tab',9)
        time.sleep(.1)
        results[action]['next_tab'] = focus_sample(w.c)
        w.snap('rename-'+action+'-tab','Project switcher')
    save(w.out / f'focus-metrics-{w.width}.json',results)
    metrics_file=w.out/'focus-metrics.json'
    all_metrics=json.loads(metrics_file.read_text()) if metrics_file.exists() else {}
    all_metrics[str(w.width)]=results
    save(metrics_file,all_metrics)
    # Sign out is last on this box. The phone's Accounts page is also compared.
    if w.width==420:
        w.nav('Settings');w.c.click('Accounts','link')
        w.m=w.mock_page
        key(w.m,'Escape',27)
        w.mock_nav('Settings');w.m.click('Accounts','link')
        w.snap('accounts','Accounts','Accounts tab against the same-width mock; Sign out at its foot.')
    else:
        w.c.click('Owner','button')
        w.words('Owner menu', 'The person’s menu offers Sign out at desktop width. Reference: client/README.md, 3.5, Accounts; 5.3.')
        w.snap('owner-menu','Owner menu')
    if w.width==1440:
        opened=w.c.js("[...document.querySelectorAll('[role=menuitem]')].some(e=>e.checkVisibility({checkVisibilityCSS:true})&&e.innerText.trim()==='Sign out')")
        if not opened:w.c.click('Owner','button')
    w.c.click('Sign out','menuitem' if w.width==1440 else 'button');w.c.wait('Signed out')
    w.words('Signed out', 'Sign out ends this browser’s session. A reload and typing /accounts keep it signed out. Back in only by a fresh printed link.')
    w.snap('signed-out','Sign out')
    w.c.call('Page.reload');w.c.wait('Signed out');w.snap('signed-out-reload','Sign out')
    origin=w.c.js('location.origin')
    w.c.go(origin+'/accounts');w.c.wait('Signed out');w.snap('signed-out-accounts','Sign out','Typed /accounts as the task requires.')
    before=sum(line.startswith('http://') for line in w.box.logpath.read_text().splitlines())
    w.c.go(w.box.link)
    deadline=time.monotonic()+10
    while time.monotonic()<deadline:
        links=[line for line in w.box.logpath.read_text().splitlines() if line.startswith('http://')]
        if len(links)>before:break
        time.sleep(.1)
    else:raise RuntimeError('No fresh printed fake sign-in link')
    w.c.go(links[-1]);w.c.wait('Chats');w.snap('back-in','Home','Fresh fake printed link; no real sign-in used.')



def viewport(w, height):
    for tab in [w.c,w.m]:
        tab.call('Emulation.setDeviceMetricsOverride',width=w.width,height=height,deviceScaleFactor=1,mobile=False)


def scroll_metrics(tab):
    return tab.js("""(()=>{let dim=e=>({height:e.scrollHeight,width:e.scrollWidth,clientHeight:e.clientHeight,clientWidth:e.clientWidth});
      let pane=[...document.querySelectorAll('*')].filter(e=>{let s=getComputedStyle(e),r=e.getBoundingClientRect();return e.checkVisibility({checkVisibilityCSS:true})&&r.bottom>0&&r.right>0&&r.top<innerHeight&&r.left<innerWidth&&['auto','scroll'].includes(s.overflowY)&&e.scrollHeight>e.clientHeight+1});
      return {window:{height:innerHeight,width:innerWidth},document:dim(document.documentElement),body:dim(document.body),panes:pane.map(e=>({tag:e.tagName,cls:e.className,role:e.getAttribute('role'),overflow:getComputedStyle(e).overflowY,top:e.scrollTop,...dim(e)})),log:[...document.querySelectorAll('[role=log]')].map(e=>({top:e.scrollTop,...dim(e)}))}})()""")


def wheel(tab, down):
    point=tab.js("(()=>{let e=document.querySelector('[role=log]');if(!e)throw Error('No conversation log');let r=e.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}})()")
    tab.call('Input.dispatchMouseEvent',type='mouseWheel',**point,deltaX=0,deltaY=100000 if down else -100000)
    time.sleep(.3)


def long_chat(w, rounds=66):
    w.send('Let us keep a long local conversation.')
    w.c.wait('Hello from the local fake model.')
    task=re.search(r'/chats/([0-9]+)',w.c.js('location.pathname'))[1]
    w.box.login_ctl()
    sent=[]
    for number in range(rounds):
        words=f'Reading paragraph {number+1}. '+('Keep these words available while reading back. '*24)
        result=w.box.ctl('task','message',task,words,'--key',f'history-{number+1}')
        sent.append({'number':number+1,'bytes':len(words.encode()),'result':result})
    w.history_way=f'box {w.box.number}; initial composer message, {rounds} history messages by ctl; reopened from Chats; pane controls in browser'
    save(w.out/f'long-chat-{w.width}.json',{'way':w.history_way,'task':task,'messages':sent})
    (w.out/f'long-chat-activity-{w.width}.json').write_text(w.box.ctl('task','activity',task,'--json'))
    w.nav('Chats');w.c.click('T'+task,'link')
    w.c.wait('Hello from the local fake model.')
    time.sleep(.5)
    for number in range(6):
        w.send(f'Visible reading paragraph {number+1}. '+('Keep these words available while reading back. '*4),False)
        time.sleep(.7)
    w.history_way+='; six bounded visible paragraphs through the composer (browser_pages fixture shape)'
    save(w.out/f'long-chat-visible-{w.width}.json',{'text':w.c.text(),'metrics':scroll_metrics(w.c)})


def scroll(w):
    w.first(False,False);w.settings(False)
    height=720 if w.width==1280 else 600
    viewport(w,height)
    long_chat(w)
    w.mock_menu('Chats');w.m.click('+ New chat','link')
    w.mock_send('Keep these words available while reading back. '*24)
    records=[];loads=0
    w.c.js("window.loadingEarlier=[];new MutationObserver(()=>{if(document.body.innerText.includes('Loading earlier messages'))window.loadingEarlier.push(performance.now())}).observe(document.body,{subtree:true,childList:true,characterData:true})")
    for load in range(3):
        for place in ['end','top']:
            wheel(w.c,place=='end')
            metrics=scroll_metrics(w.c)
            records.append({'load':load,'mode':'Chat','place':place,**metrics})
            w.words('Reading back — one scrolling pane', 'The document and body never exceed the window. The chat alone scrolls; Older reads earlier messages. Reference: notes/decisions.md, final section; client/README.md, 3.5, Reading back.')
            w.snap(f'load-{load}-{place}','Chat',json.dumps(metrics),way=w.history_way+f'; {loads} Older loads')
        if load<2 and has(w.c,'Older'):
            press(w.c,'Older');time.sleep(.2)
            deadline=time.monotonic()+40
            while 'Loading earlier messages' in w.c.text() and time.monotonic()<deadline:time.sleep(.2)
            if 'Loading earlier messages' in w.c.text():
                w.snap(f'older-{load+1}-still-loading','Chat','M2: Older was pressed but had not landed within the 40-second observation; later loads unobserved. Ten-page window_page fixture: a_full_chat_window_offers_older_and_reads_it_when_asked.','unreachable')
                w.snap('older-2-unobserved','Chat','M2: no second press while the first read is outstanding; its end/top states are unobserved.','unreachable')
                break
            loads+=1
        elif load<2:
            w.snap(f'older-{load+1}-absent','Chat',f'Only {loads} Older loads available. The ten-page window_page fixture in a_full_chat_window_offers_older_and_reads_it_when_asked (temper-client-domain tests/pages/chat.rs) leaves two older pages; gallery chat-work-reading shows loading.','unreachable')
            break
    loading=w.c.js('window.loadingEarlier')
    # Open the draft pane; its body may stay empty when the fake cannot answer the cue.
    if has(w.c,'+ Draft'):
        w.c.click('+ Draft','button');time.sleep(.7)
    if has(w.c,'Split'):
        w.c.click('Split','button')
        for place in ['end','top']:
            wheel(w.c,place=='end');metrics=scroll_metrics(w.c)
            records.append({'load':loads,'mode':'Split','place':place,**metrics})
            w.snap('split-'+place,'Chat and draft',json.dumps(metrics))
    else:w.snap('split-unavailable','Chat','This width has no Split control.','unreachable')
    save(w.out/f'scroll-metrics-{w.width}.json',{'samples':records,'older_loads':loads,'loading_observed':loading})


def run_line(w, state):
    runs=w.c.js("[...document.querySelectorAll('.temper-work-run > summary')].filter(e=>e.checkVisibility({checkVisibilityCSS:true})).map(e=>({name:e.getAttribute('aria-label')||e.innerText,tag:e.tagName}))")
    if not runs:
        w.snap(state+'-live','Agent' if state.startswith('agent') else 'Chat','The fake’s live page has no grouped call disclosure.','unreachable')
        gallery_runs(w,'task-solo-T9' if state.startswith('agent') else 'chat-work-done',state)
        return
    name=runs[0]['name'].strip().replace('\n',' ')
    w.snap(state+'-shut','Chat','Run of calls occupies one folded line; client/README.md, 3.5.')
    w.c.click(name,'button');w.snap(state+'-open','Chat','Run opens to its call rows.')
    w.c.click(name,'button');w.snap(state+'-shut-again','Chat','Run shuts again.')



def gallery_runs(w,gallery,prefix='agent-calls'):
    live=w.c
    try:
        w.c=w.browser.tab();w.c.go((w.out/'gallery'/(gallery+'.html')).as_uri())
        w.words('Call line','A folded run of calls: open to rows; shut to one line. Reference: client/README.md, 3.5, A call’s line.')
        names=w.c.js("[...document.querySelectorAll('.temper-work-run > summary')].map(e=>e.getAttribute('aria-label')||e.innerText)")
        if not names:raise RuntimeError('Agent gallery has no run line')
        name=names[0].strip().replace('\n',' ')
        for state in ['shut','open','shut-again']:
            if state!='shut':w.c.click(name,'button')
            w.snap(prefix+'-'+state,'Agent' if gallery.startswith('task-') else 'Chat','No live grouped call run was held; native gallery disclosure opened and shut.','unreachable',f'gallery {gallery}; box {w.box.number} attempt')
    finally:w.c=live

def chat_work(w):
    w.first(False,False);w.settings(False)
    w.send('Land a local hello status change.')
    provisional=[];deadline=time.monotonic()+8
    while time.monotonic()<deadline:
        state=w.c.js("({writing:[...document.querySelectorAll('.temper-writing-end')].filter(e=>e.innerText.includes('▍')).map(e=>e.innerText),text:document.body.innerText})")
        if state['writing']:
            provisional.append(state);w.words('Growing answer','Words being written look like committed agent text, with a faint inline ▍. Reference: client/README.md, 3.5, While it works.');w.snap('growing-live','Chat');break
        time.sleep(.04)
    save(w.out/f'growing-poll-{w.width}.json',{'seconds':8,'caught':provisional})
    w.c.wait('I asked a local coder',seconds=40) if 'I asked a local coder' in w.c.text() else time.sleep(1)
    w.words('Call line','One folded run of calls; open and shut. Reference: client/README.md, 3.5, A call’s line.')
    run_line(w,'landing-calls')
    if not provisional:w.gallery_state('growing-answer','chat-work-writing','Eight-second polling did not catch provisional text; fake finishes too quickly. Named gallery state.',('Growing answer','A faint ▍ follows the growing words inline; the name stays hidden. Reference: client/README.md, 3.5.'))
    # The delegated coder is linked from the landing chat, where available.
    links=w.c.js("[...document.querySelectorAll('a')].filter(e=>e.checkVisibility({checkVisibilityCSS:true})&&['/tasks/','/agents/'].some(p=>(e.getAttribute('href')||'').includes(p))).map(e=>({name:e.getAttribute('aria-label')||e.innerText,href:e.getAttribute('href')}))")
    if links:
        w.c.click(links[0]['name'].strip().replace('\n',' '),'link');run_line(w,'agent-calls')
    else:
        gallery_runs(w,'task-solo-T9')
    long_chat(w,12)
    wheel(w.c,False)
    before=scroll_metrics(w.c)
    w.words('Latest ↓','While reading back, new words do not move the view. Latest returns to the end and goes. Reference: client/README.md, 3.5, Reading back.')
    w.snap('latest-shown','Chat')
    # Send through ctl so producing new words does not itself scroll the composer.
    task=re.search(r'/chats/([0-9]+)',w.c.js('location.pathname'))[1]
    w.box.login_ctl();w.box.ctl('task','message',task,'New words while I read back.','--key','read-back')
    time.sleep(.8);after=scroll_metrics(w.c)
    w.snap('latest-new-words','Chat')
    latest=has(w.c,'Latest')
    if latest:w.c.click('Latest','button')
    time.sleep(.4);end=scroll_metrics(w.c)
    w.snap('latest-return','Chat')
    save(w.out/f'latest-{w.width}.json',{'before':before,'new_words':after,'after_press':end,'latest_before_press':latest,'latest_after_press':has(w.c,'Latest')})
    if has(w.c,'Older'):
        w.c.js("window.loadingEarlier=[];new MutationObserver(()=>{if(document.body.innerText.includes('Loading earlier messages'))window.loadingEarlier.push(performance.now())}).observe(document.body,{subtree:true,childList:true,characterData:true})")
        press(w.c,'Older');text=w.c.text()
        if 'Loading earlier messages' in text:w.snap('older-loading-live','Chat')
        else:w.gallery_state('older-loading','chat-work-reading','Older read completed before screenshot; named gallery shows Loading earlier messages… (not a live timing proof).',('Reading back','Loading earlier messages… replaces Older only while its read is out. Reference: client/README.md, 3.5.'))
    else:w.gallery_state('older-loading','chat-work-reading','No Older remains in this window; named gallery shows loading.',('Reading back','Loading earlier messages… replaces Older only while its read is out. Reference: client/README.md, 3.5.'))


SHEETS = [
    ('close','chat-closing','Close this chat?','You can still read it, but it can’t be reopened.','Keep it','Close chat'),
    ('start-over','chat-closing-start-over','Start over?','A new configuring chat starts. This one stays in Chats.','Keep going','Start over'),
    ('stop-goal','goal-stop-dialog','Stop this goal?','What landed on its branch stays there.','Keep going','Stop goal'),
    ('cancel-task','task-cancelling','Cancel this task?','What it changes; only nonzero counts.','Keep it','Cancel task'),
    ('stop-task','task-stopping','Stop this task?','Its run stops, and the task is held until it is released.','Keep going','Stop task'),
    ('accept','proposal-accepting','Accept this proposal?','Its facts.','Not yet','Accept'),
    ('reject','proposal-rejecting','Reject this proposal?','Its facts and reason.','Keep it','Reject'),
    ('release','task-releasing','Release this task?','What releasing does.','Keep it held','Release'),
    ('release-uncertain','task-releasing-uncertain','Release this task?','Repeat the effect; Do not repeat the effect.','Keep it held','Release'),
]


def sheet_metrics(tab):
    return tab.js("""(()=>{let d=document.querySelector('dialog[open],[role=dialog]');if(!d)return null;let r=d.getBoundingClientRect(),s=getComputedStyle(d);return {rect:{x:r.x,y:r.y,width:r.width,height:r.height},margin:s.margin,title:d.getAttribute('aria-label')||d.querySelector('h1,h2,h3')?.innerText,text:d.innerText,buttons:[...d.querySelectorAll('button')].map(e=>({name:e.getAttribute('aria-label')||e.innerText,cls:e.className,background:getComputedStyle(e).backgroundColor,color:getComputedStyle(e).color}))}})()""")


def sheet_live(w,case,title,says,keep,does,opener):
    def open_sheet():
        duplicate=case=='reject' and any(c['tag']=='SUMMARY' and c['name'].strip()==opener for c in w.c.controls())
        w.c.click(opener,'button',occurrence=1 if duplicate else 0)
    w.words(title,says+' Acts in order: '+keep+'; '+does+' (primary). Escape and the keep act change nothing; the act does its work. Reference: client/README.md, 3.6, The sheet.')
    w.box.login_ctl()
    before=w.box.ctl('task','list','--project','1','--json')
    open_sheet()
    first=sheet_metrics(w.c)
    if not first:raise RuntimeError('No confirmation sheet after '+opener)
    w.snap(case+'-sheet','Confirmation')
    key(w.c,'Escape',27);time.sleep(.3)
    escape=sheet_metrics(w.c)
    after_escape=w.box.ctl('task','list','--project','1','--json')
    w.snap(case+'-escape','Confirmation','Escape left the sheet open; no work changed.' if escape else 'Escape dismissed the sheet; no work changed.','differs' if escape else 'matches')
    if escape:w.c.click(keep,'button')
    open_sheet();w.c.click(keep,'button')
    kept=sheet_metrics(w.c)
    after_keep=w.box.ctl('task','list','--project','1','--json')
    w.snap(case+'-kept','Confirmation','Keep dismisses the sheet; no work changed.')
    open_sheet()
    # A proposal rejection needs its reason; visible names only.
    w.c.click(does,'button');time.sleep(.7)
    if case=='close':w.c.wait('Closed at',seconds=15)
    w.snap(case+'-done','Confirmation','Pressed '+does+'; resulting live state and ctl retained.')
    save(w.out/f'sheet-{w.box.number}-{case}.json',{'first':first,'after_escape':escape,'after_keep':kept,'after_does':sheet_metrics(w.c),'text':w.c.text(),'tasks_before':json.loads(before),'tasks_after_escape':json.loads(after_escape),'tasks_after_keep':json.loads(after_keep),'tasks_after_does':json.loads(w.box.ctl('task','list','--project','1','--json'))})



def sheet_look(w):
    w.c.go((w.out/'gallery/proposal-rejecting.html').as_uri())
    w.c.js("(()=>{let d=document.querySelector('dialog');d.close();d.showModal()})()")
    w.m.go((ROOT/'temper/docs/design/client/greenfield/home.html').as_uri())
    w.m.js("(()=>{let d=document.createElement('dialog');d.className='sheet';d.innerHTML='<h2>Reject this proposal?</h2><p>Its facts and reason.</p><button>Keep it</button><button>Reject</button>';document.body.append(d);d.showModal()})()")
    metrics={'client':sheet_metrics(w.c),'mock':sheet_metrics(w.m),'width':w.width}
    save(w.out/f'sheet-position-{w.width}.json',metrics)
    w.snap('sheet-position','Confirmation','Native modal measurements: client width 460px (clamped on phone), mock 620px; the narrower sheet is allowed by client/README.md, 3.6.','matches','gallery proposal-rejecting, native modal; mock dialog.sheet with reference words')

def close_chat(w):
    w.stop()
    # Use independent boxes so the literal first-chat/first-goal scripts hold.
    for offset,(case,gallery,title,says,keep,does) in enumerate(SHEETS,1):
        if getattr(w,'only_cases',None) and case not in w.only_cases:continue
        child=Addition(w.out,w.width,100+w.box.number*20+offset,'close-chat')
        try:
            child.first(False,False)
            opener=None
            if case=='close':
                child.send('A local chat to close.');child.c.wait('Hello from the local fake model.');opener='Close'
            elif case in ['start-over','accept','reject']:
                if case=='start-over':child.settings(False)
                child.nav('Settings');child.c.click('Start','link')
                words='Show drafts and goals for this project.' if case in ['accept','reject'] else 'Write this up as a draft beside us.'
                child.c.fill('Describe how you work, or what to change',words);child.c.click('Send','button')
                child.c.wait('I proposed showing drafts and goals.' if case in ['accept','reject'] else 'I wrote an Away mode draft')
                if case=='start-over':
                    child.c.ready('Start over',seconds=8);opener='Start over'
                else:
                    child.nav('Needs you');child.c.wait('Show drafts and goals')
                    opener='Accept' if case=='accept' else 'Reject'
                    if case=='reject':
                        child.c.click('Reject','button')
                        field=next(c['name'] for c in child.c.controls() if c['tag']=='TEXTAREA')
                        child.c.fill(field,'Keep the current project settings.')
                        opener='Reject proposal' if has(child.c,'Reject proposal') else 'Reject'
            elif case=='stop-goal':
                child.settings(False);child.send('Start a local away goal.');child.c.wait('I started the local away goal.')
                child.nav('Goals');child.c.wait('Walk the local away decision.')
                links=[c['name'] for c in child.c.controls() if c['tag']=='A' and 'local' in c['name'].lower()]
                if links:child.c.click(links[0].strip().replace('\n',' '),'link')
                child.c.ready('Stop',seconds=8)
                opener='Stop'
            elif case in ['release','stop-task','cancel-task']:
                child.settings(False);child.send('Start a local away goal.');child.c.wait('I started the local away goal.')
                child.box.login_ctl()
                if case=='release':
                    stopped=child.box.ctl('task','stop','2','--key','sheet-held')
                    (child.out/f'hold-{child.box.number}.txt').write_text(stopped)
                child.nav('Needs you')
                # Keep the public held state even if a temper-owned release card is absent.
                (child.out/f'hold-inbox-{child.box.number}.json').write_text(child.box.ctl('inbox','--json'))
                opener={'release':'Release','stop-task':'Stop','cancel-task':'Cancel'}[case]
            if opener and has(child.c,opener):
                try:sheet_live(child,case,title,says,keep,does,opener)
                except Exception as error:
                    child.snap(case+'-live-fault','Confirmation',str(error),'unreachable')
                    child.gallery_state(case+'-gallery',gallery,'Live act could not complete: '+str(error),(title,says+' Acts: '+keep+' first, '+does+' primary second. Reference: client/README.md, 3.6, The sheet.'))
            else:
                child.gallery_state(case,gallery,'No reachable live '+case+' act in the fake box; gallery shows the sheet. Escape, keep and effect are unobserved on a static gallery.',(title,says+' Acts: '+keep+' first, '+does+' primary second. Reference: client/README.md, 3.6, The sheet.'))
        except Exception as error:
            child.gallery_state(case+'-fallback',gallery,'Fake path unavailable: '+str(error)+'; behaviours unobserved on static gallery.',(title,says+' Acts: '+keep+' first, '+does+' primary second. Reference: client/README.md, 3.6, The sheet.'))
        finally:child.stop()


def model(w):
    w.c.go(w.box.link);w.c.wait('Set up temper')
    w.m.go((ROOT/'temper/docs/design/client/greenfield/welcome.html').as_uri())
    if has(w.m,'Start the first run'):w.m.click('Start the first run','button')
    else:w.m.click('Mockup','button');w.m.click('Start the first run over','button')
    for tab in [w.c,w.m]:tab.click('Set up temper','link')
    w.c.click('Run on this machine','link');w.m.click('Run on this machine','button')
    for tab in [w.c,w.m]:tab.click('Run on this machine','button')
    w.c.wait('Continue with ChatGPT');w.m.wait('Continue with ChatGPT')
    w.snap('choose','Model','Plan cards and Sign in, in a new tab · about a minute; design 3.5, Signing in and The try.')
    for tab in [w.c,w.m]:tab.click('Use your Claude subscription','link')
    w.snap('claude','Model','Private Claude environment; command and I’ve signed in; model-check trial behaviour is owed.')
    # Fake program check: no credential read, no real service.
    button=next(c['name'] for c in w.c.controls() if c['tag']=='BUTTON' and ('signed in' in c['name'].lower()))
    press(w.c,button)
    w.words('The try — Claude','While the host check is out the try box shows the first step; stopped, Claude Code isn’t signed in here yet, with Try again, Sign in again, Continue anyway. A real trial remains owed. Reference: client/README.md, 3.5, The try.')
    w.snap('claude-check','Model')
    time.sleep(.7);w.snap('claude-check-result','Model')
    if has(w.c,'All ways'):w.c.click('All ways','link')
    elif has(w.c,'Sign in again'):
        w.c.click('Sign in again','button');w.c.click('All ways','link')
    if has(w.c,'Continue with ChatGPT'):
        press(w.c,'Continue with ChatGPT')
        w.words('Signing in','Waiting for ChatGPT’s sign-in, in the new tab. Open it again ›; folded paste. Reference: client/README.md, 3.5, Signing in.')
        w.snap('chatgpt-after-press','Model','Fake issuer completes directly; tab and trial behaviour remain owed.')
        w.c.ready('Create your project');w.snap('chatgpt-ready','Ready','Fake sign-in makes the model usable without credentials.')
    model_gallery(w)


def model_gallery(w):
    for state,name in [('trial-running','welcome-model-claude-checking'),('trial-answered','welcome-model-connected'),('trial-stopped','welcome-model-claude-not-yet')]:
        w.gallery_state(state,name,'Trial execution is owed; this named gallery holds the landed host-check appearance, not a paid trial.',('The try', 'Running: Trying your ACCOUNT; first step under way. Answered: ✓ ACCOUNT; Continue. Stopped: WHO didn’t answer; Try again, Sign in again or All ways, Continue anyway. Reference: client/README.md, 3.5, The try.'))
    names=[line for line in (ROOT/'temper/tools/gallery/names.txt').read_text().splitlines() if re.search('sign|check|try',line) or line.startswith(('welcome-model-','welcome-paste-','connect-model-'))]
    save(w.out/f'model-gallery-names-{w.width}.json',names)
    for name in names:
        w.gallery_state(name,name,'Named gallery for sign-in/check state not held by this fake’s immediate success; looks only. Behaviour owed by sign-in-new-tab sections 2–6 and model-check sections 2–5.',('Signing in / The try','Compare the waiting note, external reopen link, folded paste and trial step box. Running: Trying your ACCOUNT. Answered: ✓ ACCOUNT. Stopped: WHO didn’t answer. Reference: client/README.md, 3.5, Signing in and The try.'))


def own_way(w):
    w.first(False,False)
    for fresh in [False,True]:
        project='Hello'
        if fresh:
            w.nav('Settings');w.nav('Home');w.c.click('Hello','button');w.c.fill('+ New project','Fresh project');key(w.c,'Enter',13);w.c.wait('Fresh project');project='Fresh project'
        label='fresh' if fresh else 'initial'
        w.nav('Settings');w.c.click('Start','link')
        w.words('Configure temper — temper’s own way','Ways to begin; Not sure yet? Ask what temper can do. Card: At scale, as temper builds itself; Draft → The cut → Coding agents → Goal’s branch → main. See how it works; Start from it. Reference: client/README.md, 3.5, The configuring chat’s page.')
        w.snap(label+'-card','Configure')
        w.c.click('See how it works','button');w.c.wait('How temper builds temper')
        w.snap(label+'-reference','Configure','Reference beside the page in Split; phone uses Draft; no chat is made.')
        for mode in ['Chat','Draft','Split']:
            if has(w.c,mode):w.c.click(mode,'button');w.snap(label+'-mode-'+mode.lower(),'Configure','Mode only; reference changes nothing.')
            elif mode=='Split' and w.width==420:w.snap(label+'-mode-split','Configure','No Split at 420, as the design says.','unreachable')
        if has(w.c,'Chat'):w.c.click('Chat','button')
        w.c.click('Start from it','button')
        expected=f'Set {project} up the way temper builds itself, sized for this machine.'
        w.c.wait(expected);w.c.wait('Hello from the local fake model.')
        save(w.out/f'own-way-{w.width}-{label}.json',{'expected_sent':expected,'text':w.c.text()})
        w.snap(label+'-started','Configure','Sent '+expected+' Fake says Hello from the local fake model.; this fallback answer is not a finding.')
        w.c.click('Close','button');w.c.click('Close chat','button');w.c.wait('Closed at',seconds=15)
        w.nav('Settings');w.c.click('Start','link')
        name=next(c['name'] for c in w.c.controls() if c['tag']=='BUTTON' and c['name'].startswith('Not sure yet?'))
        w.c.click(name,'button');w.c.wait('Not sure yet. What can temper do?');w.c.wait('Hello from the local fake model.')
        w.snap(label+'-not-sure','Configure','Sent Not sure yet. What can temper do?; fake fallback is named, not a finding.')

def index(out):
    rows=[r for file in sorted(out.glob('rows-*.json')) for r in json.loads(file.read_text())]
    body=['<!doctype html><meta charset="utf-8"><title>Additions walk</title><style>body{font:15px system-ui;margin:30px}td{padding:8px;vertical-align:top}img{max-width:600px}</style><h1>Additions walk</h1><table>']
    for r in rows:
        stem=r['pair'].removesuffix('-pair.png')
        links=' · '.join(f'<a href="{stem}-{k}.png">{k}</a>' for k in ['mock','client','pair'])
        body.append('<tr><td>'+html.escape(f"{r['step']} / {r['state']} / {r['width']}")+'<td>'+html.escape(f"{r['verdict']}: {r['why']} ({r['way']})")+'<td>'+links+f'<br><img loading="lazy" src="{r["pair"]}">')
    (out/'index.html').write_text('\n'.join(body)+ '</table>')
    return rows


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--section',choices=SECTIONS+['all'],required=True)
    p.add_argument('--width',type=int,choices=[1440,420],help='Limit a non-A4, non-scroll section to one width.')
    p.add_argument('--case',action='append',choices=[row[0] for row in SHEETS],help='Restrict close-chat; may be repeated.')
    p.add_argument('--calls-only',action='store_true',help='Only chat-work gallery disclosures, after a live attempt.')
    p.add_argument('--gallery-only',action='store_true',help='Only Model gallery states, after its fresh-box path.')
    p.add_argument('--sheet-look-only',action='store_true',help='Compare close-chat modal position with mock dialog.sheet.')
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--bundle',type=Path,required=True)
    p.add_argument('--gallery',type=Path,required=True)
    args=p.parse_args();os.umask(0o077)
    started=time.monotonic()
    started_utc=subprocess.check_output(['date','-u','+%FT%TZ'],text=True).strip()
    out=args.out.expanduser().resolve()
    if out.exists():raise SystemExit('Use a fresh output directory.')
    prepare(out);shutil.copytree(args.bundle.resolve(),out/'build/bundle');shutil.copytree(args.gallery.resolve(),out/'gallery')
    save(out/'build.json',{'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'builder':socket.gethostname(),'started_utc':time.strftime('%FT%TZ',time.gmtime())})
    failures=[];number=0
    try:
        for section in SECTIONS if args.section=='all' else [args.section]:
            widths=[1440]*7+[420]*3 if section=='a4' else [1280,1000] if section=='scroll' else [1440,420]
            if args.width and section not in ['a4','scroll']:widths=[args.width]
            for width in widths:
                number+=1;w=None
                try:
                    w=Addition(out,width,number,section)
                    w.only_cases=args.case
                    if section=='chat-work' and args.calls_only:
                        gallery_runs(w,'chat-work-done','landing-calls');gallery_runs(w,'task-solo-T9')
                    elif section=='model' and args.gallery_only:model_gallery(w)
                    elif section=='close-chat' and args.sheet_look_only:sheet_look(w)
                    else:globals()[section.replace('-','_')](w)
                except Exception as error:
                    failures.append({'section':section,'width':width,'box':number,'error':str(error)})
                    save(out/'faults.json',failures)
                    print(f'{section} {width} box {number}: {type(error).__name__}: {error}',flush=True)
                    if w:
                        try:w.words('Section fault',str(error));w.snap('fault','Fault',str(error),'unreachable')
                        except Exception:pass
                finally:
                    if w:w.stop()
                    index(out)
    finally:
        index(out)
        shutil.rmtree(out/'boxes',ignore_errors=True);shutil.rmtree(out/'build',ignore_errors=True)
        save(out/'timing.json',{'started_utc':started_utc,'finished_utc':subprocess.check_output(['date','-u','+%FT%TZ'],text=True).strip(),'seconds':round(time.monotonic()-started,3)})
    if failures:raise SystemExit(1)


if __name__=='__main__':
    def interrupted(signum,frame):raise KeyboardInterrupt
    signal.signal(signal.SIGTERM,interrupted)
    main()
