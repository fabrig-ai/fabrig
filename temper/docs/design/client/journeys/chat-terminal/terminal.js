'use strict';
// The look request chat-terminal. Nothing here talks to anything: a mock, drawn to be tried.
(() => {
  const tt = document.getElementById('temper-terminal');
  const app = document.getElementById('app');
  const log = tt.querySelector('.tt-log');
  const box = tt.querySelector('textarea');
  const list = tt.querySelector('.tt-list');
  const line = tt.querySelector('.tt-line');
  const run = tt.querySelector('.tt-run');
  const doc = tt.querySelector('.tt-doc');
  const draftButton = tt.querySelector('.tt-draft');
  const modelButton = tt.querySelector('.tt-model');
  const title = tt.querySelector('.tt-title');

  const state = { model: 'Claude Opus 5.5', effort: 'high', spent: 778, paused: false, closed: false, draft: true };
  let choosing = null;   // what the list holds now: {items, at, pick}
  let working = null;

  const COMMANDS = [
    ['/model', 'Change the model, from the next turn', () => chooseModel()],
    ['/effort', 'Change the effort', () => chooseEffort()],
    ['/draft', 'Show or hide the draft', () => toggleDraft()],
    ['/pause', 'Pause this chat; a message resumes it', () => pause()],
    ['/close', 'Close this chat; it stays in Chats', () => closeChat()],
    ['/start-over', 'Start a new configuring chat', () => startOver()],
    ['/new', 'Start a new chat', () => newChat()],
    ['/chats', 'Open another chat', () => chooseChat()],
    ['/theme', 'Dark or light', () => theme()],
    ['/font', 'DejaVu Sans Mono or IBM Plex Mono, to compare', () => font()],
    ['/help', 'Commands and keys', () => help()],
    ['/exit', 'Leave the terminal', () => leave()],
  ];

  // Entering and leaving.
  function enter(full) {
    tt.hidden = false;
    document.body.classList.add('in-terminal');
    app.inert = true;
    if (full && document.documentElement.requestFullscreen) document.documentElement.requestFullscreen().catch(() => {});
    log.scrollTop = log.scrollHeight;
    box.focus();
  }
  function leave() {
    tt.hidden = true;
    document.body.classList.remove('in-terminal');
    app.inert = false;
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
  }

  // The log.
  function add(node) { log.append(node); log.scrollTop = log.scrollHeight; return node; }
  function note(words) { const p = document.createElement('p'); p.className = 'tt-note'; p.textContent = words; return add(p); }
  function say(words) { const d = document.createElement('div'); d.className = 'tt-entry tt-says'; const p = document.createElement('p'); p.textContent = words; d.append(p); return add(d); }
  function you(words) { const d = document.createElement('div'); d.className = 'tt-entry tt-you'; d.textContent = words; return add(d); }

  function status() {
    modelButton.textContent = `${state.model} · ${state.effort}`;
    tt.querySelector('.tt-spent').textContent = `${state.spent} spent`;
    tt.querySelector('.tt-hint').textContent = state.paused ? 'paused · a message resumes it' : '/ for commands';
  }

  // The draft.
  function toggleDraft(show) {
    if (!state.draft && show !== false) { note('The agent starts a draft.'); return; }
    const on = show === undefined ? !tt.classList.contains('with-draft') : show;
    tt.classList.toggle('with-draft', on);
    doc.hidden = !on;
    draftButton.setAttribute('aria-pressed', on ? 'true' : 'false');
    log.scrollTop = log.scrollHeight;
    box.focus();
  }

  // The list under the box.
  function closeList() {
    list.hidden = true; list.textContent = ''; list.classList.remove('tt-choices'); choosing = null;
    line.hidden = false; box.setAttribute('aria-expanded', 'false');
  }
  function drawList() {
    list.textContent = '';
    if (choosing.head) { const h = document.createElement('div'); h.className = 'tt-head'; h.textContent = choosing.head; list.append(h); }
    choosing.items.forEach((item, i) => {
      const row = document.createElement('div');
      row.setAttribute('role', 'option');
      row.setAttribute('aria-selected', i === choosing.at ? 'true' : 'false');
      const a = document.createElement('span'); a.textContent = item[0];
      const b = document.createElement('span'); b.textContent = item[1] || '';
      row.append(a, b);
      row.addEventListener('mousemove', () => { if (choosing.at !== i) { choosing.at = i; drawList(); } });
      row.addEventListener('mousedown', e => { e.preventDefault(); choosing.at = i; take(); });
      list.append(row);
    });
    list.hidden = false; line.hidden = true;
    box.setAttribute('aria-expanded', 'true');
  }
  function offer(head, items, at, pick, choices) {
    choosing = { head, items, at: Math.max(0, at), pick };
    list.classList.toggle('tt-choices', !!choices);
    drawList();
    log.scrollTop = log.scrollHeight;
  }
  function take() {
    const { items, at, pick } = choosing;
    const item = items[at];
    box.value = ''; size(); closeList();
    pick(item);
    box.focus();
  }

  // What the commands do.
  function commandsFor(text) {
    const q = text.toLowerCase();
    return COMMANDS.filter(c => c[0].startsWith(q));
  }
  function typed() {
    const text = box.value;
    if (choosing && choosing.head && text) closeList();
    if (/^\/\S*$/.test(text)) {
      const found = commandsFor(text);
      if (found.length) { offer('', found.map(c => [c[0], c[1]]), 0, item => runCommand(item[0])); return; }
    }
    if (choosing && !choosing.head) closeList();
  }
  function runCommand(name) {
    const c = COMMANDS.find(x => x[0] === name);
    if (c) c[2](); else note(`Unknown command ${name}. /help lists them.`);
  }
  function chooseModel() {
    const rows = [['Claude Opus 5.5', 'your subscription · default'], ['Claude Sonnet 5.5', 'your subscription'], ['Claude Fable 5.1', 'your subscription'], ['OpenAI', 'Connect a plan or key ›']];
    offer('Model. A change applies from the next turn.', rows, rows.findIndex(r => r[0] === state.model), item => {
      if (item[0] === 'OpenAI') { note('Connect a plan or key opens here.'); return; }
      state.model = item[0]; status(); note(`Model: ${item[0]}, from the next turn.`);
    }, true);
  }
  function chooseEffort() {
    const rows = ['low', 'medium', 'high', 'xhigh', 'max'].map(e => [e, '']);
    offer('Effort. A change applies from the next turn.', rows, rows.findIndex(r => r[0] === state.effort), item => {
      state.effort = item[0]; status(); note(`Effort: ${item[0]}, from the next turn.`);
    }, true);
  }
  function chooseChat() {
    offer('Chats', [['T2', 'How does temper work? Briefly.'], ['T3', 'hello']], 0, item => note(`${item[0]} opens here, in this look.`), false);
  }
  function pause() { stopWorking(); state.paused = true; status(); note('Paused. A message resumes it.'); }
  function closeChat() {
    offer('Close this chat? It stays in Chats.', [['Close', ''], ['Keep it open', '']], 0, item => {
      if (item[0] !== 'Close') return;
      stopWorking(); state.closed = true; note('Closed. /new starts another.');
      box.placeholder = '/new starts another chat';
    }, true);
  }
  function startOver() {
    offer('Start over? A new configuring chat starts. This one stays in Chats.', [['Start over', ''], ['Keep this one', '']], 0, item => {
      if (item[0] === 'Start over') note('A new configuring chat opens here, in this look.');
    }, true);
  }
  function newChat() {
    stopWorking();
    log.textContent = '';
    toggleDraft(false);
    state.draft = false;
    title.textContent = 'New chat';
    tt.querySelector('.tt-task').textContent = '';
    draftButton.textContent = '+ Draft';
    box.placeholder = 'Say what you want; the agent works in work/hello.';
    state.spent = 0; state.closed = false; state.paused = false; status();
  }
  function font() {
    tt.dataset.font = tt.dataset.font === 'plex' ? 'dejavu' : 'plex';
    note(`Font: ${tt.dataset.font === 'plex' ? 'IBM Plex Mono' : 'DejaVu Sans Mono'}.`);
  }
  function theme() { tt.dataset.theme = tt.dataset.theme === 'light' ? 'dark' : 'light'; }
  function help() {
    const d = document.createElement('div'); d.className = 'tt-note';
    const pre = document.createElement('pre');
    const width = Math.max(...COMMANDS.map(c => c[0].length)) + 2;
    pre.textContent = COMMANDS.map(c => c[0].padEnd(width) + c[1]).join('\n') +
      '\n\n' + ['Enter sends', 'Shift+Enter makes a new line', 'Ctrl+\\ shows or hides the draft', 'PgUp and PgDn read back', 'Esc closes a list'].join('\n');
    d.append(pre); add(d);
  }

  // A turn. Nobody answers on a mock, so it says so.
  function startWorking(then, ms) {
    stopWorking();
    const began = Date.now();
    run.hidden = false;
    const tick = () => {
      const s = Math.floor((Date.now() - began) / 1000);
      run.querySelector('b').textContent = 'Writing…';
      run.querySelector('span').textContent = ` ${s} s`;
      state.spent += 1; status();
    };
    tick();
    working = { timer: setInterval(tick, 1000), end: ms ? setTimeout(() => { stopWorking(); if (then) then(); }, ms) : null };
  }
  function stopWorking() {
    if (!working) return;
    clearInterval(working.timer); clearTimeout(working.end); working = null; run.hidden = true;
  }
  function send(text) {
    you(text);
    state.paused = false; status();
    startWorking(() => say('This page is a mock, so nobody answers. On a real chat the agent’s words arrive here as it writes them, with its tools folded above them.'), 2600);
  }

  // The box.
  function size() { box.style.height = 'auto'; box.style.height = box.scrollHeight + 'px'; }
  box.addEventListener('input', () => { size(); typed(); });
  box.addEventListener('keydown', e => {
    if (choosing) {
      if (e.key === 'ArrowDown') { e.preventDefault(); choosing.at = (choosing.at + 1) % choosing.items.length; drawList(); return; }
      if (e.key === 'ArrowUp') { e.preventDefault(); choosing.at = (choosing.at + choosing.items.length - 1) % choosing.items.length; drawList(); return; }
      if (e.key === 'Tab' && !choosing.head) { e.preventDefault(); box.value = choosing.items[choosing.at][0]; typed(); return; }
      if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); take(); return; }
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); box.value = box.value.startsWith('/') ? '' : box.value; size(); closeList(); return; }
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      const text = box.value.trim();
      if (!text) return;
      box.value = ''; size();
      if (text.startsWith('/')) runCommand(text.split(/\s/)[0]); else send(text);
      return;
    }
    if (e.key === 'PageUp' || e.key === 'PageDown') {
      e.preventDefault();
      log.scrollBy({ top: (e.key === 'PageUp' ? -1 : 1) * log.clientHeight * 0.9 });
    }
  });

  // Controls.
  document.addEventListener('keydown', e => {
    if (tt.hidden) return;
    if (e.ctrlKey && e.key === '\\') { e.preventDefault(); toggleDraft(); }
  });
  tt.addEventListener('click', e => {
    e.stopPropagation();
    const t = e.target;
    if (!(t instanceof Element)) return;
    if (t.closest('.tt-exit')) { leave(); return; }
    if (t.closest('.tt-draft')) { toggleDraft(); return; }
    if (t.closest('.tt-model')) { box.focus(); chooseModel(); return; }
    if (t.closest('.tt-show')) { toggleDraft(true); return; }
    if (t.closest('a, button, summary, .tt-doc')) return;
    if (!String(getSelection())) box.focus();
  });
  document.querySelector('.temper-to-terminal').addEventListener('click', e => {
    e.preventDefault(); e.stopPropagation(); enter(true);
  });

  // Where a page of the journey starts.
  status();
  const start = document.body.dataset.start;
  if (start) {
    enter(false);
    if (start === 'draft') toggleDraft(true);
    if (start === 'menu') { box.value = '/'; typed(); }
    if (start === 'model') chooseModel();
    if (start === 'working') startWorking();
  }
})();
