// Greenfield mockup: every page drawn from data.js and what the viewer
// did, kept in localStorage so that pages, tabs and the phone frame stay
// in step. Nothing is sent anywhere.
(() => {
  const M = window.MOCK;
  const KEY = 'temper-greenfield-v1';
  const PLAN_MS = 5000;
  const framed = window.top !== window;
  if (framed) document.documentElement.classList.add('framed');
  const params = new URLSearchParams(location.search);
  const page = document.body.dataset.page;
  const NUM = ['Nothing', 'One', 'Two', 'Three', 'Four', 'Five', 'Six', 'Seven', 'Eight', 'Nine'];
  const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const narrow = () => matchMedia('(max-width: 760px)').matches;
  const ICON = {
    chat: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><path d="M2.5 3.5h11v7h-6l-3 2.5v-2.5h-2z"/></svg>',
    split: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><rect x="2" y="3" width="12" height="10" rx="1.5"/><path d="M8 3v10"/></svg>',
    doc: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><path d="M4 2h5.5L12 4.5V14H4z"/><path d="M6 7h4M6 9.5h4M6 12h2.5"/></svg>',
    send: '<svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M8 13V3M3.5 7.5 8 3l4.5 4.5"/></svg>',
    pen: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><path d="m10.5 2.5 3 3L6 13H3v-3z"/></svg>',
  };

  // ---- State -----------------------------------------------------------
  const fresh = () => ({ project: 'fabrig', scenario: 'morning', answered: {}, later: {}, overturned: {}, away: {}, parts: {}, settings: {}, goalPatch: {}, chatPatch: {}, newGoals: {}, draftGoal: {}, threads: {}, panel: {}, sent: {}, extraNeeds: {}, newChats: {}, newDrafts: {}, draftChat: {}, newDocs: {}, docChanged: {}, goalFilter: 'all', chatFilter: 'all' });
  const load = () => { try { return Object.assign(fresh(), JSON.parse(localStorage.getItem(KEY) || '{}')); } catch { return fresh(); } };
  let S = load();
  if (M.projects[params.get('project')]) S.project = params.get('project');
  const save = () => { try { localStorage.setItem(KEY, JSON.stringify(S)); } catch { /* no storage: this page only */ } };

  const pid = () => S.project;
  const P = () => M.projects[pid()];
  const parts = () => Object.assign({}, P().parts, S.parts[pid()] || {});
  const setting = (k, d) => (S.settings[pid()] || {})[k] ?? d;
  const awayOn = () => !!S.away[pid()];
  const scenario = () => (pid() === 'fabrig' ? S.scenario : 'morning');
  const list = (o, k) => (o[pid()] = o[pid()] || []);

  // ---- The world, as the viewer has changed it --------------------------
  function goals() {
    return [...(S.newGoals[pid()] || []), ...P().goals].map(g => {
      let x = { ...g };
      if (scenario() === 'away' && g.awayPatch) x = { ...x, ...g.awayPatch };
      if (x.plannedPatch && Date.now() - x.created >= PLAN_MS) x = { ...x, ...x.plannedPatch };
      if (S.goalPatch[g.id]) x = { ...x, ...S.goalPatch[g.id] };
      return x;
    });
  }
  const goal = id => goals().find(g => g.id === id);
  function needs() {
    const sc = scenario();
    const out = P().needs.filter(n => !n.scenario || n.scenario === sc);
    if (sc === 'away') for (const d of P().decided) if (S.overturned[d.id]) {
      out.push({ id: `o-${d.id}`, goal: d.goal, kind: 'question', asked: 'overturned just now', short: d.question, question: d.question, context: `Decided for you at ${d.at}: ${d.took}. You overturned it, so it asks you again.`, choices: d.options.map((label, i) => ({ label, rec: label === d.took ? false : i === 0 })) });
    }
    out.push(...(S.extraNeeds[pid()] || []).filter(n => !n.showAfter || Date.now() >= n.showAfter));
    return out;
  }
  const open = () => needs().filter(n => !S.answered[n.id]).sort((a, b) => (S.later[a.id] ? 1 : 0) - (S.later[b.id] ? 1 : 0));
  const recent = (ms = 120000) => needs().filter(n => S.answered[n.id] && Date.now() - S.answered[n.id].at < ms);
  const needsOf = g => open().filter(n => n.goal === g);
  function chats() {
    return [...(S.newChats[pid()] || []), ...P().chats].map(c => ({ ...c, ...(S.chatPatch[c.id] || {}) }));
  }
  const chat = id => chats().find(c => c.id === id);
  function drafts() {
    return [...(S.newDrafts[pid()] || []), ...P().drafts].map(d => {
      const g = S.draftGoal[d.id];
      const c = S.draftChat[d.id];
      if (g) return { ...d, chat: d.chat || c, goal: g, status: 'goal running', updated: 'just now' };
      return c ? { ...d, chat: c, status: 'writing', updated: 'just now' } : d;
    });
  }
  function goalState(g) {
    const n = needsOf(g.id);
    if (n.length) return ['needs', n.some(x => x.kind === 'funds') ? 'needs funds' : 'needs you'];
    if (g.paused) return ['paused', 'paused by you'];
    const words = { running: 'running', planning: 'planning', landed: g.ended ? `landed ${g.ended}` : 'landed', stopped: 'stopped' };
    return [g.state, words[g.state] || g.state];
  }
  const wayName = id => (P().ways.find(w => w.id === id) || { name: id }).name;
  const needSentence = (n, still) => n === 0 ? (still ? 'Nothing else needs you.' : 'Nothing needs you.') : n === 1 ? (still ? 'One thing still needs you.' : 'One thing needs you.') : `${NUM[n] || n} things ${still ? 'still need' : 'need'} you.`;

  // ---- Acts --------------------------------------------------------------
  function answer(id, idx, reply) {
    const n = needs().find(x => x.id === id);
    if (!n) return;
    const c = reply != null ? { label: `Replied: “${reply.length > 60 ? `${reply.slice(0, 57)}…` : reply}”`, effect: { line: 'Read your reply; carrying on with it in mind.' } } : n.choices[idx];
    const prev = { goal: n.goal ? S.goalPatch[n.goal] : undefined, chat: n.chat ? S.chatPatch[n.chat] : undefined };
    S.answered[id] = { label: c.label, at: Date.now(), prev };
    delete S.later[id];
    const e = c.effect;
    if (e && n.goal) {
      const patch = { ...(S.goalPatch[n.goal] || {}), by: 'Written by its agent just now' };
      if (e.line) { patch.line = e.line; patch.short = e.line; }
      if (e.state) patch.state = e.state;
      if (e.budget) patch.budget = e.budget;
      S.goalPatch[n.goal] = patch;
    }
    if (e && n.chat) S.chatPatch[n.chat] = { ...(S.chatPatch[n.chat] || {}), status: e.status || 'working', live: true };
    save();
    render();
    toast(`Answered: ${c.label}`, 'Undo', () => undo(id));
  }
  function undo(id) {
    const a = S.answered[id];
    if (!a) return;
    const n = needs().find(x => x.id === id);
    if (n && n.goal) { if (a.prev.goal) S.goalPatch[n.goal] = a.prev.goal; else delete S.goalPatch[n.goal]; }
    if (n && n.chat) { if (a.prev.chat) S.chatPatch[n.chat] = a.prev.chat; else delete S.chatPatch[n.chat]; }
    delete S.answered[id];
    save();
    render();
  }
  function toggleAway() {
    S.away[pid()] = !awayOn();
    save();
    render();
    if (awayOn()) toast(`Away until ${P().awayUntil}: agents decide within ${P().name}’s limits.`, 'Limits', () => { location.href = 'settings.html#away'; });
    else toast('Away is off: questions wait for you again.');
  }
  function startGoal(form) {
    const d = drafts().find(x => x.id === form.dataset.draft);
    const ways = P().ways;
    const way = form.querySelector('input[name=way]:checked')?.value || P().defaultWay;
    let id = (form.querySelector('#goal-name').value || d.id.replace(/\.md$/, '')).trim().toLowerCase().replace(/[^a-z0-9-]+/g, '-');
    while (goal(id)) id = `${id}-2`;
    const budget = parseInt(String(form.querySelector('#goal-budget').value).replace(/\D/g, ''), 10) || 40;
    const asks = form.querySelector('input[name=asks]:checked')?.value || 'away';
    const created = Date.now();
    const g = {
      id, state: 'planning', way, draft: d.id, done: 0, total: 0, spent: 0, budget, since: '08:06', created, asks,
      line: `Planning from ${d.id}: reading it and the designs it cites.`, short: 'Planning from the draft.',
      by: 'Written by its agent just now',
      report: [`Reading ${d.id} and the designs it cites, to propose its tasks.`], left: 'the plan.', risk: 'none known yet.',
      tasks: [], landing: `${wayName(way)}.`, decided: [],
      plannedPatch: {
        state: 'running', total: 5, spent: 1,
        line: 'Planned five tasks from the draft; two are running. Its first question is the draft’s open one.',
        short: 'Five tasks planned from the draft; two are running.', by: 'Written by its agent a moment ago',
        report: [`The plan has five tasks: the setting and its switch, the kinds agents may decide, decided-for-you in Needs you, coming back, and the tests. The first two run.`, 'The draft leaves one question open, how long a decision can be overturned; I’ve asked you rather than guess.'],
        left: 'all five tasks.', risk: 'Overturn needs a stored record of each decision; that is a stored format.',
        tasks: [{ name: 'The away setting and switch', state: 'running', when: 'just now' }, { name: 'Kinds agents may decide', state: 'running', when: 'just now' }, { name: 'Decided for you', state: 'waits', when: 'on you' }, { name: 'Coming back', state: 'waits', when: 'on the first two' }, { name: 'Tests', state: 'waits', when: 'on all' }],
      },
    };
    list(S.newGoals).unshift(g);
    S.draftGoal[d.id] = id;
    if (d.openQuestion) list(S.extraNeeds).push({ id: `n-${id}`, goal: id, kind: 'question', asked: 'just now', showAfter: created + PLAN_MS, short: d.openQuestion.short, question: d.openQuestion.question, context: 'From the draft’s open questions.', choices: d.openQuestion.choices });
    if (d.chat) (S.threads[d.chat] = S.threads[d.chat] || []).push({ who: 'tool', text: `Started goal ${id}`, detail: wayName(way), goal: id });
    save();
    location.href = `goal.html?g=${encodeURIComponent(id)}`;
  }

  // ---- Toast ------------------------------------------------------------
  let toastTimer;
  function toast(text, act, fn) {
    let t = document.getElementById('toast');
    if (!t) { t = document.createElement('div'); t.id = 'toast'; t.className = 'toast'; t.setAttribute('role', 'status'); document.body.append(t); }
    t.innerHTML = `<span>${esc(text)}</span>${act ? `<button type="button">${esc(act)}</button>` : ''}`;
    if (act) t.querySelector('button').onclick = () => { t.classList.remove('show'); fn(); };
    t.classList.add('show');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => t.classList.remove('show'), 4500);
  }

  // ---- The frame --------------------------------------------------------
  function switcher() {
    return `<details class="switcher"><summary>${esc(P().name)}</summary><div class="switcher-menu">
      ${Object.values(M.projects).map(p => `<button type="button" data-project="${esc(p.name)}"${p.name === pid() ? ' aria-current="true"' : ''}>${esc(p.name)} <small>${esc(p.repo)}</small></button>`).join('')}
      <a class="btn quiet small" href="settings.html">Settings for ${esc(P().name)}</a></div></details>`;
  }
  function navItems() {
    const pt = parts();
    const n = open().length;
    return [
      ['home', 'home.html', 'Home'],
      ['needs', 'needs.html', 'Needs you', n],
      pt.goals && ['goals', 'goals.html', 'Goals'],
      pt.drafts && ['drafts', 'drafts.html', 'Drafts'],
      ['chats', 'chats.html', 'Chats'],
      pt.changes && ['changes', 'changes.html', 'Changes'],
    ].filter(Boolean);
  }
  function rail(active) {
    const pt = parts();
    return `<nav class="rail" aria-label="${esc(P().name)}">
      <a class="brand" href="home.html">temper</a>
      ${switcher()}
      <div class="nav">${navItems().map(([k, h, l, n]) => `<a href="${h}"${k === active ? ' aria-current="page"' : ''}><span>${l}</span>${n ? `<span class="count">${n}</span>` : ''}</a>`).join('')}</div>
      ${pt.drafts ? '<a class="new" href="chat.html?id=new&amp;draft=1">+ New draft</a>' : '<a class="new" href="chat.html?id=new">+ New chat</a>'}
      <div class="rail-foot">
        ${pt.away ? `<button type="button" class="away" role="switch" aria-checked="${awayOn()}" data-away><span>${awayOn() ? `Away until ${esc(P().awayUntil)}` : 'Away'}</span><i></i></button>` : ''}
        <a href="settings.html"${active === 'settings' ? ' aria-current="page"' : ''}>Settings</a>
        <span>${esc(M.person)}</span>
      </div>
    </nav>`;
  }
  function phonebar() {
    return `<header class="phonebar">${switcher()}${parts().away ? `<button type="button" class="away-chip" role="switch" aria-checked="${awayOn()}" data-away>${awayOn() ? `Away until ${esc(P().awayUntil)}` : 'Away: off'}</button>` : ''}</header>`;
  }
  function tabbar(active) {
    const pt = parts();
    const n = open().length;
    const tabs = [['home', 'home.html', 'Home'], ['needs', 'needs.html', 'Needs you', n], pt.goals ? ['goals', 'goals.html', 'Goals'] : ['chats', 'chats.html', 'Chats'], pt.goals ? ['chats', 'chats.html', 'Chats'] : ['settings', 'settings.html', 'Settings']];
    return `<nav class="tabbar" aria-label="Sections">${tabs.map(([k, h, l, c]) => `<a href="${h}"${k === active ? ' aria-current="page"' : ''}>${l}${c ? ` <span class="count">${c}</span>` : ''}</a>`).join('')}</nav>`;
  }
  const frame = (active, inner) => `<div class="app">${rail(active)}${phonebar()}<main class="main" id="main">${inner}</main>${tabbar(active)}</div>`;

  // ---- Pieces -----------------------------------------------------------
  function owner(n) {
    if (n.goal) return `<a class="link" href="goal.html?g=${encodeURIComponent(n.goal)}">${esc(n.goal)}</a>`;
    const c = chat(n.chat);
    return `<a class="link" href="chat.html?id=${encodeURIComponent(n.chat)}">${esc(c ? c.title : n.chat)}</a>`;
  }
  const kindWord = n => ({ question: 'a question', funds: 'funds', review: 'a review' }[n.kind] || n.kind);
  function choiceButtons(n, extra = true) {
    return `<div class="choices">${n.choices.map((c, i) => `<button type="button" class="btn${c.rec ? ' primary' : ''}${c.quiet ? ' quiet' : ''}" data-answer="${esc(n.id)}" data-choice="${i}">${esc(c.label)}${c.rec && n.kind === 'question' ? '<span class="rec">recommended</span>' : ''}</button>`).join('')}
      ${extra ? `<button type="button" class="btn quiet" data-reply="${esc(n.id)}">Reply</button><button type="button" class="btn quiet" data-later="${esc(n.id)}">Later</button>` : ''}</div>`;
  }
  const replyBox = n => `<form class="reply" data-reply-form="${esc(n.id)}" hidden><textarea aria-label="Your reply" placeholder="Your words go to ${esc(n.goal || 'the chat')}’s agent"></textarea><div class="acts"><button class="btn primary small">Send reply</button><button type="button" class="btn quiet small" data-reply-cancel>Cancel</button></div></form>`;
  function askCard(n) {
    return `<article class="ask" data-ask="${esc(n.id)}">
      <p class="mono">${owner(n)} · ${kindWord(n)} · ${esc(n.asked)}${S.later[n.id] ? ' · later' : ''}</p>
      <p class="q">${esc(n.question)}</p>
      ${n.context ? `<p class="ctx">${esc(n.context)}</p>` : ''}
      ${n.saw ? `<details><summary>What the agent saw</summary><p>${esc(n.saw)}</p></details>` : ''}
      ${choiceButtons(n)}${replyBox(n)}
    </article>`;
  }
  const answeredLines = items => items.map(n => `<div class="answered"><span>${owner(n)}:</span><strong>${esc(S.answered[n.id].label)}</strong><button type="button" data-undo="${esc(n.id)}">Undo</button></div>`).join('');
  const shortOf = n => (n.kind === 'funds' ? `${n.goal} ${n.short}` : n.short);
  function goalRow(g) {
    const [st, words] = goalState(g);
    const ended = st === 'landed' || st === 'stopped';
    return `<a class="goal-row${ended ? ' ended' : ''}" href="goal.html?g=${encodeURIComponent(g.id)}">
      <span class="top mono"><span>${esc(g.id)}</span><span>${g.total ? `${g.done} / ${g.total} · ` : ''}<span class="state-${st}">${esc(words)}</span></span></span>
      <span class="line">${esc(g.short)}</span></a>`;
  }
  const composer = (attr, placeholder, hint) => `<form class="composer-wrap" ${attr}><div class="composer"><textarea rows="1" placeholder="${esc(placeholder)}" aria-label="${esc(placeholder)}"></textarea><button type="submit" aria-label="Send" disabled>${ICON.send}</button></div>${hint ? `<p class="composer-hint">${esc(hint)}</p>` : ''}</form>`;

  // ---- Pages ------------------------------------------------------------
  function home() {
    const p = P();
    const sc = scenario();
    const intro = sc === 'away' ? p.away : p.morning;
    const o = open();
    const dec = sc === 'away' ? p.decided : [];
    const headline = sc === 'away' ? `${intro.headline} ${NUM[dec.length]} things were decided for you.` : `${intro.headline} ${needSentence(o.length)}`;
    const pt = parts();
    const gs = goals();
    const live = gs.filter(g => !['landed', 'stopped'].includes(goalState(g)[0]));
    const shown = [...live, ...gs.filter(g => !live.includes(g))].slice(0, Math.max(5, live.length));
    return frame('home', `<div class="col">
      <header class="intro">
        <p class="kicker">${esc(intro.kicker)}</p>
        <h1 class="headline">${esc(headline)}</h1>
        <p class="lede">${esc(intro.text)}</p>
        <p class="byline">${esc(intro.by)}</p>
      </header>
      ${dec.length ? `<section class="section" aria-labelledby="dec">
        <div class="section-head"><h2 class="section-title" id="dec">Decided for you · ${dec.length}</h2><a class="link" href="settings.html#away">Limits</a></div>
        <div class="decided">${dec.map(d => `<div class="decision${S.overturned[d.id] ? ' overturned' : ''}"><div>
            <span class="mono">${esc(d.goal)} · ${esc(d.at)}</span><span class="muted">${esc(d.question)}</span><span class="took">${esc(d.took)}</span><span class="why">${esc(d.why)}</span></div>
            ${S.overturned[d.id] ? '<span class="late">Overturned: it asks you again</span>' : d.landed ? `<span class="late">landed ${esc(d.landed)} · a new goal can undo it</span>` : `<button type="button" class="btn small" data-overturn="${esc(d.id)}">Overturn</button>`}</div>`).join('')}</div>
      </section>` : ''}
      <section class="section" aria-labelledby="needs">
        <div class="section-head"><h2 class="section-title${o.length ? ' signal' : ''}" id="needs">${sc === 'away' ? 'Still needs you' : 'Needs you'}${o.length ? ` · ${o.length}` : ''}</h2>${o.length > 1 ? '<a class="link hide-narrow" href="needs.html">Answer in a row</a>' : ''}</div>
        ${answeredLines(recent())}
        ${o.length ? `<div class="needs-full section">${askCard(o[0])}${o.length > 1 ? `<div class="lines">${o.slice(1).map(n => `<a href="needs.html?id=${encodeURIComponent(n.id)}"><span>${esc(shortOf(n))}</span><span class="when">${kindWord(n)} · ${esc(n.asked)}</span></a>`).join('')}</div>` : ''}</div>
          <a class="needs-cta" href="needs.html"><span>${esc(o[0].goal || chat(o[0].chat)?.title || '')}: ${esc(o[0].short)}</span><span class="go">${o.length === 1 ? 'Answer' : 'Answer, one at a time'}</span></a>`
        : '<p class="empty">Nothing needs you. Goals carry on, and this page tells you what changed when you come back.</p>'}
      </section>
      ${pt.goals ? `<section class="section" aria-labelledby="goals">
        <div class="section-head"><h2 class="section-title" id="goals">Goals</h2><a class="link" href="goals.html">All ${gs.length}</a></div>
        ${gs.length ? `<div class="goal-rows">${shown.map(goalRow).join('')}</div>` : '<p class="empty">No goals yet. Write a draft with a chat, then start a goal from it.</p>'}
      </section>` : `<section class="section" aria-labelledby="chats">
        <div class="section-head"><h2 class="section-title" id="chats">Chats</h2><a class="link" href="chats.html">All ${chats().length}</a></div>
        <div class="lines">${chats().slice(0, 4).map(c => `<a href="chat.html?id=${encodeURIComponent(c.id)}"><span>${esc(c.title)}</span><span class="when">${esc(c.status)} · ${esc(c.when)}</span></a>`).join('')}</div>
        <p class="facts">A chat suits one change. For bigger work, a draft written with a chat becomes a goal: a plan, its tasks, landing. <a class="link" href="settings.html">Turn on drafts and goals</a></p>
      </section>`}
      <p class="facts">Landing: ${esc(p.landing.line)}${pt.changes ? ' · <a class="link" href="changes.html">Changes</a>' : ''}</p>
    </div>`);
  }

  function needsPage() {
    const o = open();
    const done = recent(600000);
    const want = params.get('id');
    const n = (want && o.find(x => x.id === want)) || o[0];
    if (!n) {
      return frame('needs', `<div class="col narrow">
        <header class="intro"><p class="kicker">Needs you</p><h1 class="headline">Nothing needs you.</h1><p class="lede">Every question is answered. Goals carry on; Home tells you what changed.</p></header>
        ${answeredLines(done)}
        <p><a class="btn" href="home.html">Home</a></p></div>`);
    }
    const total = done.length + o.length;
    const at = done.length + 1;
    const next = o.filter(x => x !== n)[0];
    return frame('needs', `<div class="col narrow flow">
      <div class="flow-head"><div class="dots" role="img" aria-label="${at} of ${total}">${Array.from({ length: total }, (_, i) => `<span${i < done.length ? ' class="done"' : ''}></span>`).join('')}</div><span class="kicker">${at} of ${total}</span></div>
      <article class="ask" data-ask="${esc(n.id)}" style="background:none;box-shadow:none;padding:0">
        <p class="mono">${owner(n)} · ${kindWord(n)} · ${esc(n.asked)}</p>
        <h1 class="flow-q">${esc(n.question)}</h1>
        ${n.context ? `<p class="ctx">${esc(n.context)}</p>` : ''}
        ${n.saw ? `<details><summary>What the agent saw</summary><p>${esc(n.saw)}</p></details>` : ''}
        ${choiceButtons(n, false)}
        <div class="pair"><button type="button" class="btn" data-reply="${esc(n.id)}">Reply…</button><button type="button" class="btn" data-later="${esc(n.id)}">Later</button></div>
        ${replyBox(n)}
      </article>
      ${next ? `<p class="next-up">Next: ${esc(next.goal || chat(next.chat)?.title)} · ${esc(next.short)}</p>` : '<p class="next-up">The last one</p>'}
      ${done.length ? `<div class="section"><h2 class="section-title">Answered</h2>${answeredLines(done)}</div>` : ''}
    </div>`);
  }

  function goalsPage() {
    const f = S.goalFilter;
    const all = goals();
    const pick = { all: () => true, needs: g => goalState(g)[0] === 'needs', running: g => ['running', 'planning', 'paused'].includes(goalState(g)[0]), ended: g => ['landed', 'stopped'].includes(goalState(g)[0]) };
    const shown = all.filter(pick[f] || pick.all);
    const fb = (k, l) => `<button type="button" aria-pressed="${f === k}" data-goal-filter="${k}">${l}</button>`;
    return frame('goals', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · goals</p><h1 class="headline">${all.length} goals, ${all.filter(pick.running).length + all.filter(pick.needs).length} under way.</h1>
      <p class="lede">A goal turns a draft into landed work, its own way: ${esc(P().ways.map(w => w.name.toLowerCase()).join(', '))}.</p></header>
      <section class="section"><div class="section-head"><div class="filters">${fb('all', 'All')}${fb('needs', 'Needs you')}${fb('running', 'Under way')}${fb('ended', 'Ended')}</div>${parts().drafts ? '<a class="link" href="drafts.html">Start one from a draft</a>' : ''}</div>
      ${shown.length ? `<div class="goal-rows">${shown.map(goalRow).join('')}</div>` : '<p class="empty">None.</p>'}</section>
    </div>`);
  }

  function goalPage() {
    const gs = goals();
    const g = goal(params.get('g')) || gs[0];
    if (!g) return frame('goals', '<div class="col"><p class="empty">No goals in this project yet.</p></div>');
    const [st, words] = goalState(g);
    const n = needsOf(g.id);
    const planning = g.state === 'planning';
    const left = g.tasks.filter(t => t.state !== 'landed');
    const landed = g.tasks.filter(t => t.state === 'landed');
    const sent = S.sent[g.id] || [];
    return frame('goals', `<div class="col wide">
      <header class="goal-head">
        <p class="meta mono"><b>${esc(g.id)}</b><span>·</span><span class="state-${st}">${esc(words)}</span>${g.total ? `<span>·</span><span>${g.done} of ${g.total} tasks</span>` : ''}<span>·</span><span>$${g.spent} of $${g.budget}</span><span>·</span><span>since ${esc(g.since)}</span></p>
        <h1>${esc(g.line)}</h1>
        <p class="byline">${esc(g.by)}</p>
        <div class="acts">
          ${g.draft ? `<a class="btn small" href="${draftHref(g.draft)}">Draft</a>` : g.chat ? `<a class="btn small" href="chat.html?id=${encodeURIComponent(g.chat)}">Chat ${esc(g.chat)}</a>` : ''}
          ${['landed', 'stopped'].includes(g.state) ? '' : `<button type="button" class="btn small" data-amend>Amend</button><button type="button" class="btn small" data-pause="${esc(g.id)}">${g.paused ? 'Resume' : 'Pause'}</button><button type="button" class="btn quiet small" data-stop="${esc(g.id)}">Stop</button>`}
        </div>
      </header>
      ${n.map(x => `<section class="band" aria-label="Needs you"><p><span class="tag">Needs you · ${kindWord(x)}</span>${esc(x.question)}</p>${choiceButtons(x, false)}</section>`).join('')}
      <div class="two">
        <article class="report" aria-labelledby="rep">
          <h2 class="section-title" id="rep">Report</h2>
          ${g.report.map(r => `<p>${esc(r)}</p>`).join('')}
          ${planning ? '<div class="typing" aria-label="Planning"><i></i><i></i><i></i></div>' : `<p><strong>Left:</strong> ${esc(g.left)}</p><p><strong>Risk:</strong> ${esc(g.risk)}</p>`}
          <p class="byline">${esc(g.by.replace('Written by ', 'By '))} · <a class="link" href="#" data-later-note="Earlier reports: each rewrite is kept, newest first.">earlier reports</a></p>
        </article>
        <aside class="side">
          <section class="section" style="gap:4px" aria-labelledby="tasks">
            <h2 class="section-title" id="tasks" style="margin-bottom:8px">Tasks</h2>
            ${g.tasks.length ? `<div class="bar" role="img" aria-label="${landed.length} landed of ${g.tasks.length}">${g.tasks.map(t => `<span class="${t.state}"></span>`).join('')}</div>
            ${left.map(t => `<p class="task-line ${t.state}"><span>${esc(t.name)}</span><span>${t.state === 'running' ? 'running' : `waits ${esc(t.when)}`}</span></p>`).join('')}
            ${landed.length ? `<details><summary class="link" style="font-size:13px;margin-top:6px;cursor:pointer">${landed.length} landed</summary>${landed.map(t => `<p class="task-line"><span>${esc(t.name)}</span><span>${esc(t.when)}</span></p>`).join('')}</details>` : ''}` : `<p class="facts">${planning ? 'Proposing tasks from the draft…' : 'No tasks.'}</p>`}
          </section>
          <section class="section" style="gap:6px"><h2 class="section-title">Landing</h2><p class="facts">${esc(g.landing)}</p></section>
          <section class="section" style="gap:6px"><h2 class="section-title">How it runs</h2><p class="facts">${esc(wayName(g.way))} · ${g.asks === 'always' ? 'always asks you' : 'follows the away switch'}</p></section>
          ${g.decided.length ? `<section class="section" style="gap:6px"><h2 class="section-title">Decided by the goal</h2><p class="facts">${g.decided.map(esc).join(' · ')}</p></section>` : ''}
        </aside>
      </div>
      ${sent.length ? `<div class="sent">${sent.map(m => `<p class="${m.who === 'you' ? 'you' : 'muted'}">${m.who === 'you' ? '' : 'Its agent: '}${esc(m.text)}</p>`).join('')}</div>` : ''}
      ${['landed', 'stopped'].includes(g.state) ? '' : `<div style="max-width:720px">${composer(`data-goal-say="${esc(g.id)}"`, `Tell ${g.id} something`, 'Its agent reads it at once. A change of plan becomes an amendment it shows you first.')}</div>`}
    </div>`);
  }
  function draftHref(id) {
    const d = drafts().find(x => x.id === id);
    if (!d) return 'drafts.html';
    return d.chat ? `chat.html?id=${encodeURIComponent(d.chat)}` : d.doc ? `chat.html?draft=${encodeURIComponent(d.id)}` : 'drafts.html';
  }

  function draftsPage() {
    const ds = drafts();
    return frame('drafts', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · drafts</p><h1 class="headline">Write it down with an agent, then start a goal from it.</h1>
      <p class="lede">Drafts live in the repository, in <span class="mono" style="font-size:14px">docs/design/draft/</span>, and travel with the code.</p></header>
      <section class="section"><div class="section-head"><h2 class="section-title">Drafts · ${ds.length}</h2><a class="btn small" href="chat.html?id=new&amp;draft=1">+ New draft</a></div>
      <div class="goal-rows">${ds.map(d => {
        const href = d.goal ? `goal.html?g=${encodeURIComponent(d.goal)}` : draftHref(d.id);
        const st = d.goal ? `<span class="state-running">goal ${esc(d.goal)}</span>` : d.chat ? `<span class="state-needs">${esc(d.status)} with ${esc(d.chat)}</span>` : esc(d.status);
        return `<a class="goal-row" href="${href}"><span class="top mono"><span>${esc(d.path)}</span><span>${st}</span></span><span class="line">${esc(d.id.replace(/\.md$/, '').replace(/-/g, ' '))}</span><span class="byline">updated ${esc(d.updated)}</span></a>`;
      }).join('') || '<p class="empty">No drafts yet.</p>'}</div></section>
    </div>`);
  }

  function chatsPage() {
    const f = S.chatFilter;
    const cs = chats().filter(c => f === 'all' || (f === 'live' ? c.live : !c.live));
    const fb = (k, l) => `<button type="button" aria-pressed="${f === k}" data-chat-filter="${k}">${l}</button>`;
    return frame('chats', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · chats</p><h1 class="headline">Talk to an agent about ${esc(P().repo)}.</h1></header>
      <section class="section"><div class="section-head"><div class="filters">${fb('all', 'All')}${fb('live', 'Live')}${fb('closed', 'Closed')}</div><a class="btn small" href="chat.html?id=new">+ New chat</a></div>
      <div class="lines">${cs.map(c => `<a href="chat.html?id=${encodeURIComponent(c.id)}"><span>${esc(c.title)} <span class="mono">${esc(c.id)}</span></span><span class="when">${c.status === 'waiting on you' || c.status === 'waiting on your words' ? `<span class="state-needs">${esc(c.status)}</span>` : esc(c.status)} · ${esc(c.when)}</span></a>`).join('') || '<p class="empty">None.</p>'}</div></section>
    </div>`);
  }

  function changesPage() {
    const ch = P().changes;
    return frame('changes', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · landing</p><h1 class="headline">${ch.queue.length ? `${NUM[ch.queue.length]} ${ch.queue.length === 1 ? 'change is' : 'changes are'} on the way to main.` : 'Nothing is on the way to main.'}</h1>
      <p class="lede">${esc(P().landing.line)}. Each change is checked on top of main before it lands.</p></header>
      <section class="section"><h2 class="section-title">Queue</h2><div class="lines">${ch.queue.map((c, i) => `<div><span>${esc(c.title)} <span class="mono">${esc(c.from)}</span></span><span class="when">${c.state === 'checking' ? `<span class="state-running">checking</span> · ${esc(c.note)}` : `queued ${esc(c.note)}`}</span></div>`).join('') || '<p class="empty">Empty.</p>'}</div></section>
      <section class="section"><h2 class="section-title">Landed recently</h2><div class="lines">${ch.landed.map(c => `<div><span>${esc(c.title)} <span class="mono">${esc(c.from)}</span></span><span class="when">${esc(c.when)}</span></div>`).join('')}</div></section>
    </div>`);
  }

  function settingsPage() {
    const pt = parts();
    const sw = (k, label, small) => `<div class="setting"><div><span>${label}</span><small>${small}</small></div><label class="toggle-row"><span>${pt[k] ? 'Shown' : 'Hidden'}</span><input type="checkbox" class="switch" data-part="${k}"${pt[k] ? ' checked' : ''} aria-label="${label}"></label></div>`;
    const sel = (k, label, small, opts) => `<div class="setting"><div><label for="s-${k}">${label}</label>${small ? `<small>${small}</small>` : ''}</div><select id="s-${k}" data-set="${k}">${opts.map(o => `<option${setting(k, opts[0]) === o ? ' selected' : ''}>${esc(o)}</option>`).join('')}</select></div>`;
    const kinds = setting('kinds', { impl: true, retry: true, words: true, deps: false });
    const kind = (k, l) => `<label><input type="checkbox" data-kind="${k}"${kinds[k] ? ' checked' : ''}> ${l}</label>`;
    return frame('settings', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · settings</p><h1 class="headline">How work runs in ${esc(P().name)}.</h1>
      <p class="lede">Everything the other pages do for this project comes from here, so each project, and each person, can work its own way.</p></header>
      <nav class="tabs" aria-label="Settings"><a href="#" aria-current="page">How work runs</a><a href="#" data-later-note="Repositories: ${esc(P().repo)} on ${esc(P().forge)}.">Repositories</a><a href="#" data-later-note="Forges: not drawn in this experiment.">Forges</a><a href="#" data-later-note="Accounts: not drawn in this experiment.">Accounts</a><a href="#" data-later-note="Builders: a separate app; temper only names which to use.">Builders</a></nav>
      <div class="settings">
        <section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">What the project shows</h2>
          ${sw('drafts', 'Drafts', 'Documents you write with a chat, before any work starts.')}
          ${sw('goals', 'Goals', 'Bigger work from a draft: a plan, tasks, landing.')}
          ${sw('changes', 'Changes', 'The landing queue, as a page.')}
          ${sw('away', 'Away switch', 'Lets agents decide within limits while you’re away.')}
        </section>
        ${pt.goals ? `<section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">Ways of working</h2>
          ${P().ways.map(w => `<div class="setting"><div><span>${esc(w.name)}</span><small>${esc(w.text)} Used by ${w.used || 'none yet'}.</small></div><label class="toggle-row"><input type="radio" name="way" data-way="${w.id}"${setting('defaultWay', P().defaultWay) === w.id ? ' checked' : ''}> The default</label></div>`).join('')}
        </section>` : ''}
        <section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">Status</h2>
          ${sel('writer', 'Status lines and reports', 'Written words, beside the counts temper keeps itself.', ['Written by each goal’s agent', 'Written by one status agent', 'None: counts only'])}
          ${sel('when', 'Rewritten', '', ['On each change', 'At most hourly', 'When someone opens it'])}
          ${sel('brief', 'The brief on Home', 'What changed since you last looked.', ['Written by the status agent', 'A list of what changed', 'Off'])}
        </section>
        <section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">What needs you</h2>
          ${sel('within', 'Questions a goal can answer from its plan', '', ['Answered by the goal, kept in its decisions', 'Asked of me too'])}
          ${sel('funds', 'Funds', '', ['Ask when 80% of a budget is used', 'Ask when a budget is used up'])}
        </section>
        ${pt.away ? `<section class="section" id="away" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">While you’re away, agents may decide</h2>
          <div class="setting"><div><span>Kinds of decision</span><small>Each shows afterwards as decided for you, with Overturn.</small></div>
          <div class="checks">${kind('impl', 'Implementation choices within a task')}${kind('retry', 'Retries and splitting a failing task')}${kind('words', 'A design document’s wording')}${kind('deps', 'Adding a dependency')}<label class="locked"><input type="checkbox" disabled> Budgets, sign-ins, machines, what is hard to reverse: always wait</label></div></div>
          ${sel('until', 'Away until', 'When it isn’t turned off by hand.', ['08:00, the end of quiet hours', 'I turn it off'])}
        </section>` : ''}
        <section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">Telling you</h2>
          ${sel('push', 'Notifications', 'Only when something needs you; never for status.', ['To this phone’s browser', 'Off'])}
          ${sel('quiet', 'Quiet hours', '', ['23:00 to 08:00', '22:00 to 07:00', 'None'])}
        </section>
      </div>
    </div>`);
  }

  // The session: a chat, and the draft it writes beside it.
  function chatPage() {
    const p = P();
    let c;
    const id = params.get('id');
    const draftId = params.get('draft');
    if (id === 'new') c = { id: null, title: params.get('draft') ? 'New draft' : 'New chat', status: 'new', thread: [], replies: ['Here’s a first outline; it’s in the draft beside us.', 'Done; the change is highlighted in the draft.', 'Noted, it’s in.'], writes: params.get('draft') ? 'untitled.md' : null, model: 'Claude, through your subscription · Opus', fresh: true };
    else if (draftId) { const d = drafts().find(x => x.id === draftId); c = { id: null, title: d.id, status: 'new', thread: [], replies: ['Read it. What would you like to change first?', 'Done; the change is highlighted in the draft.'], writes: d.id, model: 'Claude, through your subscription · Opus', fresh: true }; }
    else c = chat(id) || chats()[0];
    const d = c.writes ? (drafts().find(x => x.id === c.writes) || { id: c.writes, path: `docs/design/draft/${c.writes}`, status: 'writing', updated: 'not yet' }) : null;
    const docSecs = d ? (S.newDocs[c.id || ''] || d.doc || []) : [];
    const key = c.writes || 'none';
    const def = d ? 'split' : 'off';
    const mode = d ? (S.panel[key] || def) : 'off';
    const shownMode = narrow() && mode === 'split' ? 'off' : mode;
    const thread = [...c.thread, ...(S.threads[c.id] || [])];
    const changed = S.docChanged[key] ?? d?.changed;
    const started = d && S.draftGoal[d.id];
    const pt = parts();
    const segBtn = (m, label, icon, cls = '') => `<button type="button" class="${cls}" data-mode="${m}" aria-pressed="${shownMode === m}" title="${label}">${ICON[icon]}<span>${label}</span></button>`;
    const turn = t => t.who === 'you' ? `<p class="turn-you">${esc(t.text)}</p>`
      : t.who === 'agent' ? `<p class="turn-agent">${esc(t.text)}</p>`
      : t.who === 'typing' ? '<div class="typing" aria-label="Writing"><i></i><i></i><i></i></div>'
      : `<p class="turn-tool">${ICON.pen}<span>${esc(t.text.replace(/ [^ ]+$/, ''))}</span><span class="file">${esc(t.text.split(' ').pop())}</span><span>${esc(t.detail || '')}</span>${t.ref && d ? `<button type="button" data-show="${esc(t.ref)}">Show</button>` : ''}${t.goal ? `<a class="link" href="goal.html?g=${encodeURIComponent(t.goal)}">Open</a>` : ''}</p>`;
    const where = d ? ['Drafts', 'drafts.html'] : ['Chats', 'chats.html'];
    return `<div class="session-app">
      <header class="session-top">
        <nav class="crumbs" aria-label="Where"><a class="back" href="${where[1]}" aria-label="Back">‹</a><span class="trail"><a href="home.html">${esc(p.name)}</a> / <a href="${where[1]}">${where[0]}</a> /</span><strong>${esc(d ? d.id : c.title)}</strong></nav>
        <div class="right">
          <span class="byline" style="white-space:nowrap">${c.id ? `${esc(c.id)} · ` : ''}${esc(c.model)}</span>
          ${d ? `<div class="seg" role="group" aria-label="The draft" data-key="${esc(key)}">${segBtn('off', 'Chat', 'chat')}${segBtn('split', 'Split', 'split', 'split')}${segBtn('full', 'Draft', 'doc')}</div>` : ''}
          ${d && pt.goals && !c.fresh ? (started ? `<a class="btn small" href="goal.html?g=${encodeURIComponent(started)}">Goal ${esc(started)} →</a>` : `<button type="button" class="btn primary small" data-start-goal>Start goal…</button>`) : ''}
        </div>
      </header>
      <div class="session" data-mode="${mode}" data-key="${esc(key)}">
        <section class="convo" aria-label="Conversation">
          <div class="thread" id="thread"><div class="thread-inner">
            ${thread.length ? '' : `<p class="meta-line">${d ? `A chat that writes ${esc(d.id)}. Say what it should hold.` : 'Say what you want; the agent works in ' + esc(p.repo) + '.'}</p>`}
            ${thread.map(turn).join('')}
          </div></div>
          <div class="convo-foot">${composer(`data-chat-say="${esc(c.id || '')}" data-writes="${esc(c.writes || '')}"`, c.fresh ? (d ? 'What should this draft hold?' : 'What do you want done?') : `Reply to ${c.id}`, 'Enter sends · Shift+Enter for a new line · Ctrl+\\ switches the draft')}</div>
        </section>
        ${d ? `<article class="doc" aria-label="${esc(d.id)}"><div class="doc-inner">
          <p class="doc-path"><span>${esc(d.path)}</span><span>${d.chat || c.id ? `edited ${esc(d.updated)}` : esc(d.status)}</span></p>
          ${docSecs.length ? docSecs.map(s => `<section class="doc-sec${s.id === changed ? ' changed' : ''}" data-sec="${esc(s.id)}">${s.html}${s.id === changed ? `<p class="changed-note">Changed by ${esc(c.id || 'the agent')} just now</p>` : ''}</section>`).join('') : '<p class="doc-empty">The draft appears here as you write it together.</p>'}
        </div></article>` : ''}
      </div>
      ${d ? startSheet(d) : ''}
    </div>`;
  }
  function startSheet(d) {
    const p = P();
    const def = setting('defaultWay', p.defaultWay);
    return `<dialog class="sheet" id="start-sheet" aria-labelledby="start-title"><form method="dialog" data-start="${esc(d.id)}" data-draft="${esc(d.id)}">
      <div style="display:flex;flex-direction:column;gap:6px"><h2 id="start-title">Start a goal from ${esc(d.id)}</h2><p class="muted" style="font-size:14px">The goal takes the draft as it is now. Later edits reach it through Amend.</p></div>
      <div class="field"><label for="goal-name">Name</label><input class="text" id="goal-name" value="${esc(d.id.replace(/\.md$/, ''))}"></div>
      <fieldset><legend>How it runs</legend>
        ${p.ways.map(w => `<label class="option"><input type="radio" name="way" value="${w.id}"${w.id === def ? ' checked' : ''}><b>${esc(w.name)}</b><small>${esc(w.text)}${w.id === def ? ` ${esc(p.name)}’s default.` : ''}</small></label>`).join('')}
        <a class="link" href="settings.html" style="font-size:13px;color:var(--muted)">The project’s ways of working</a>
      </fieldset>
      <div class="field"><label for="goal-budget">Budget</label><input class="text" id="goal-budget" value="$40" style="width:7em"><span class="muted" style="font-size:13px">asks you before going over</span></div>
      <fieldset><legend>When it needs you</legend>
        <label class="option"><input type="radio" name="asks" value="away" checked><b>Follow the away switch</b><small>Asks while you’re here; while you’re away, decides within the project’s limits and tells you.</small></label>
        <label class="option"><input type="radio" name="asks" value="always"><b>Always ask</b><small>Waits for you on every question it can’t answer from the plan.</small></label>
      </fieldset>
      <div class="foot"><button type="button" class="btn quiet" data-close>Cancel</button><button type="submit" class="btn primary" value="start">Start goal</button></div>
    </form></dialog>`;
  }

  // The phone frame: any page at a phone's size, beside the laptop view.
  function phonePage() {
    const src = params.get('p') || 'home.html';
    document.body.style.background = '#e4e4e0';
    const links = [['home.html', 'Home'], ['needs.html', 'Needs you'], ['goals.html', 'Goals'], ['goal.html', 'A goal'], ['chats.html', 'Chats'], ['chat.html?id=T4', 'Draft session'], ['settings.html', 'Settings']];
    return `<div style="min-height:100vh;display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:40px;padding:32px 16px">
      <iframe name="phone" src="${esc(src)}" title="temper at phone size" style="width:390px;height:844px;border:10px solid #18181b;border-radius:46px;background:#f7f7f5;box-shadow:0 24px 60px rgba(0,0,0,.22)"></iframe>
      <aside style="width:240px;display:flex;flex-direction:column;gap:14px;font-size:14px">
        <p class="section-title">Phone frame</p>
        <p class="muted">The same pages at 390 by 844. What you do here shows in the laptop tab too.</p>
        <div style="display:flex;flex-direction:column;gap:4px">${links.map(([h, l]) => `<a class="link" href="${h}" target="phone">${l}</a>`).join('')}</div>
        <a class="btn small" href="${esc(src)}">Laptop view</a>
      </aside></div>`;
  }

  // ---- The mockup's corner ----------------------------------------------
  function mockCorner() {
    if (framed || page === 'phone') return '';
    const here = `${location.pathname.split('/').pop()}${location.search}`;
    return `<button type="button" class="mock-fab" data-mock aria-expanded="false">Mockup</button>
      <div class="mock-panel" id="mock-panel">
        <h3>Scenario</h3>
        <label><input type="radio" name="scenario" value="morning"${S.scenario === 'morning' ? ' checked' : ''}> Morning check-in, after 9 hours</label>
        <label><input type="radio" name="scenario" value="away"${S.scenario === 'away' ? ' checked' : ''}> Back from away</label>
        <p style="color:#52616f;font-size:12px">Scenarios are fabrig’s; hello is a small new project. Switch projects with the project menu.</p>
        <h3>Pages</h3>
        <div class="pages"><a href="home.html">Home</a><a href="needs.html">Needs you</a><a href="goals.html">Goals</a><a href="goal.html?g=first-run-projects">A goal</a><a href="drafts.html">Drafts</a><a href="chat.html?id=T4">Draft session</a><a href="chats.html">Chats</a><a href="changes.html">Changes</a><a href="settings.html">Settings</a></div>
        <a href="phone.html?p=${encodeURIComponent(here)}">Open this page in a phone frame</a>
        <button type="button" class="reset" data-reset>Reset everything</button>
      </div>`;
  }

  // ---- Drawing and wiring -----------------------------------------------
  const PAGES = { home, needs: needsPage, goals: goalsPage, goal: goalPage, drafts: draftsPage, chats: chatsPage, changes: changesPage, settings: settingsPage, chat: chatPage, phone: phonePage };
  function render() {
    const scrollY = window.scrollY;
    const thread = document.getElementById('thread');
    const threadAtEnd = !thread || thread.scrollHeight - thread.scrollTop - thread.clientHeight < 40;
    const panelOpen = document.getElementById('mock-panel')?.classList.contains('open');
    document.body.innerHTML = (PAGES[page] || home)() + mockCorner();
    if (panelOpen) document.getElementById('mock-panel')?.classList.add('open');
    document.title = `${{ home: 'Home', needs: 'Needs you', goals: 'Goals', goal: params.get('g') || 'Goal', drafts: 'Drafts', chats: 'Chats', changes: 'Changes', settings: 'Settings', chat: 'Session', phone: 'Phone frame' }[page] || 'temper'} · ${P().name} · temper`;
    window.scrollTo(0, scrollY);
    const t = document.getElementById('thread');
    if (t && threadAtEnd) t.scrollTop = t.scrollHeight;
    // A goal still planning, or a question not shown yet, draws again when due.
    const due = [...goals().filter(g => g.state === 'planning' && g.created).map(g => g.created + PLAN_MS), ...(S.extraNeeds[pid()] || []).filter(n => n.showAfter > Date.now()).map(n => n.showAfter)];
    if (due.length) setTimeout(render, Math.max(50, Math.min(...due) - Date.now() + 50));
  }

  function sendChat(form, text) {
    let cid = form.dataset.chatSay;
    const writes = form.dataset.writes || null;
    if (!cid) {
      const n = Math.max(6, ...chats().map(c => parseInt(c.id.slice(1), 10) || 0)) + 1;
      cid = `T${n}`;
      const title = text.length > 42 ? `${text.slice(0, 40)}…` : text;
      const isDraft = writes === 'untitled.md';
      const docName = isDraft ? `${text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 28) || 'untitled'}.md` : writes;
      list(S.newChats).unshift({ id: cid, title: isDraft ? `${title}, draft` : title, status: 'working', live: true, when: 'now', writes: docName, model: 'Claude, through your subscription · Opus', thread: [], replies: ['Here’s a first outline; it’s in the draft beside us.', 'Done; the change is highlighted in the draft.', 'Noted, it’s in.'] });
      if (isDraft) list(S.newDrafts).unshift({ id: docName, chat: cid, status: 'writing', updated: 'just now', path: `docs/design/draft/${docName}`, doc: [] });
      else if (writes) S.draftChat[writes] = cid;
      history.replaceState(null, '', `chat.html?id=${cid}`);
      params.set('id', cid); params.delete('draft');
    }
    const th = (S.threads[cid] = S.threads[cid] || []);
    th.push({ who: 'you', text });
    save();
    render();
    const typingAt = th.length;
    th.push({ who: 'typing' });
    render();
    setTimeout(() => {
      th.splice(typingAt, 1);
      const c = chat(cid);
      const said = th.filter(x => x.who === 'agent').length;
      th.push({ who: 'agent', text: c.replies[said % c.replies.length] });
      const w = c.writes;
      if (w) {
        let secs = S.newDocs[cid];
        const d = drafts().find(x => x.id === w);
        if (!d?.doc?.length && !secs) {
          secs = S.newDocs[cid] = [
            { id: 'top', html: `<h1>${esc(text.length > 60 ? `${text.slice(0, 57)}…` : text)}</h1><p class="doc-muted">Draft, 2026-11-04, with the user.</p>` },
            { id: 'one', html: '<h2>1. In one page</h2><ul><li>[What the draft decides, in a few lines]</li></ul>' },
            { id: 'open', html: '<h2>Open questions</h2><ol><li>[What is left for you to decide]</li></ol>' },
          ];
          th.push({ who: 'tool', text: `Wrote ${w}`, detail: '12 lines', ref: 'one' });
          S.docChanged[w] = 'one';
        } else {
          const sec = (secs || d.doc).at(-2)?.id || 'one';
          th.push({ who: 'tool', text: `Edited ${w}`, detail: '+3 −1', ref: sec });
          S.docChanged[w] = sec;
        }
      }
      save();
      render();
      const s = document.querySelector('.doc-sec.changed');
      if (s) s.classList.add('flash');
    }, 1100);
  }

  document.addEventListener('click', e => {
    const t = e.target.closest('button, a, summary');
    if (!t) return;
    const d = t.dataset;
    if (d.answer) answer(d.answer, +d.choice);
    else if (d.undo) undo(d.undo);
    else if (d.later) { S.later[d.later] = true; save(); render(); toast('Later: it stays in Needs you, after the others.'); }
    else if (d.reply) { const f = document.querySelector(`[data-reply-form="${CSS.escape(d.reply)}"]`); if (f) { f.hidden = false; f.querySelector('textarea').focus(); } }
    else if ('replyCancel' in d) t.closest('form').hidden = true;
    else if (d.overturn) { S.overturned[d.overturn] = true; save(); render(); toast('Overturned: its goal asks you again, under Still needs you.'); }
    else if (d.project) { S.project = d.project; save(); location.href = 'home.html'; }
    else if ('away' in d) toggleAway();
    else if (d.goalFilter) { S.goalFilter = d.goalFilter; save(); render(); }
    else if (d.chatFilter) { S.chatFilter = d.chatFilter; save(); render(); }
    else if (d.mode) { S.panel[t.closest('.seg').dataset.key] = d.mode; save(); render(); }
    else if (d.show) {
      const key = document.querySelector('.session')?.dataset.key;
      if (key && (S.panel[key] || 'split') === 'off') { S.panel[key] = narrow() ? 'full' : 'split'; save(); render(); }
      else if (key && narrow()) { S.panel[key] = 'full'; save(); render(); }
      const s = document.querySelector(`[data-sec="${CSS.escape(d.show)}"]`);
      if (s) { s.scrollIntoView({ behavior: 'smooth', block: 'center' }); s.classList.remove('flash'); void s.offsetWidth; s.classList.add('flash'); }
    }
    else if ('startGoal' in d) document.getElementById('start-sheet').showModal();
    else if ('close' in d) t.closest('dialog').close();
    else if ('amend' in d) toast('Amend: you edit the draft beside the goal, and its agent shows you the plan’s changes first. Not drawn yet.');
    else if (d.pause) { const p = (S.goalPatch[d.pause] = S.goalPatch[d.pause] || {}); p.paused = !p.paused; save(); render(); toast(p.paused ? 'Paused: running tasks finish, nothing new starts.' : 'Resumed.'); }
    else if (d.stop) { if (confirm(`Stop ${d.stop}? What landed on its branch stays there.`)) { S.goalPatch[d.stop] = { ...(S.goalPatch[d.stop] || {}), state: 'stopped', line: 'Stopped by you. What landed on the goal branch stays there.', short: 'Stopped by you.' }; save(); render(); } }
    else if (d.laterNote != null) { e.preventDefault(); toast(d.laterNote); }
    else if ('mock' in d) { const p = document.getElementById('mock-panel'); p.classList.toggle('open'); t.setAttribute('aria-expanded', p.classList.contains('open')); }
    else if ('reset' in d) { try { localStorage.removeItem(KEY); } catch { /* none */ } location.href = 'home.html'; }
  });
  document.addEventListener('change', e => {
    const t = e.target;
    const d = t.dataset;
    const mine = () => (S.settings[pid()] = S.settings[pid()] || {});
    if (t.name === 'scenario') { S.scenario = t.value; S.project = 'fabrig'; S.away.fabrig = t.value === 'away'; save(); render(); }
    else if (d.part) { (S.parts[pid()] = S.parts[pid()] || {})[d.part] = t.checked; save(); render(); }
    else if (d.way) { mine().defaultWay = d.way; save(); }
    else if (d.set) { mine()[d.set] = t.value; save(); }
    else if (d.kind) { const k = { ...setting('kinds', { impl: true, retry: true, words: true, deps: false }) }; k[d.kind] = t.checked; mine().kinds = k; save(); }
  });
  document.addEventListener('input', e => {
    const ta = e.target.closest('.composer textarea');
    if (!ta) return;
    ta.style.height = 'auto';
    ta.style.height = `${Math.min(ta.scrollHeight, 220)}px`;
    ta.closest('.composer').querySelector('button').disabled = !ta.value.trim();
  });
  document.addEventListener('keydown', e => {
    const ta = e.target.closest && e.target.closest('.composer textarea');
    if (ta && e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); ta.closest('form').requestSubmit(); }
    if (page === 'chat' && e.key === '\\' && (e.ctrlKey || e.metaKey)) {
      const b = document.querySelectorAll('.seg button');
      const shown = [...b].filter(x => getComputedStyle(x).display !== 'none');
      const i = shown.findIndex(x => x.getAttribute('aria-pressed') === 'true');
      if (shown.length) shown[(i + 1) % shown.length].click();
    }
  });
  document.addEventListener('submit', e => {
    const f = e.target;
    if (f.dataset.start) { e.preventDefault(); startGoal(f); return; }
    e.preventDefault();
    if (f.dataset.replyForm) { const v = f.querySelector('textarea').value.trim(); if (v) answer(f.dataset.replyForm, null, v); return; }
    const v = f.querySelector('textarea')?.value.trim();
    if (!v) return;
    if (f.dataset.goalSay) {
      const g = f.dataset.goalSay;
      const m = (S.sent[g] = S.sent[g] || []);
      m.push({ who: 'you', text: v });
      save(); render();
      setTimeout(() => { m.push({ who: 'agent', text: 'Noted. If it changes the plan, I’ll show you the amendment before acting on it.' }); save(); render(); }, 1200);
    } else if ('chatSay' in f.dataset) sendChat(f, v);
  });
  window.addEventListener('storage', e => { if (e.key === KEY) { S = load(); render(); } });
  matchMedia('(max-width: 760px)').addEventListener('change', render);
  render();
})();
