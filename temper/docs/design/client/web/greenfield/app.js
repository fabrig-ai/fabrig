// Greenfield mockup: every page drawn from data.js and what the viewer
// did, kept in localStorage so that pages, tabs and the phone frame stay
// in step. Nothing is sent anywhere.
(() => {
  const M = window.MOCK;
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
  // Two scenarios, each with its own state: the everyday one (fabrig and
  // hello) and the first run (a fresh box, the wizard, the project it makes).
  const MODE_KEY = 'temper-greenfield-mode';
  const readMode = () => { try { return localStorage.getItem(MODE_KEY) || 'everyday'; } catch { return 'everyday'; } };
  const MODE = readMode();
  const FIRST = MODE === 'first-run';
  const KEY = FIRST ? 'temper-greenfield-first-run' : 'temper-greenfield-v2';
  const FR = M.firstRun;
  const fresh = () => ({ project: FIRST ? 'default' : 'fabrig', scenario: 'morning', answered: {}, later: {}, overturned: {}, away: {}, parts: {}, settings: {}, goalPatch: {}, chatPatch: {}, newGoals: {}, draftGoal: {}, threads: {}, panel: {}, sent: {}, extraNeeds: {}, newChats: {}, newDrafts: {}, draftChat: {}, newDocs: {}, docChanged: {}, goalFilter: 'all', chatFilter: 'all', names: {}, adopted: {}, accepted: {}, setup: { forge: null, model: null, making: 0, made: false } });
  const load = () => { try { return Object.assign(fresh(), JSON.parse(localStorage.getItem(KEY) || '{}')); } catch { return fresh(); } };
  let S = load();
  const save = () => { try { localStorage.setItem(KEY, JSON.stringify(S)); } catch { /* no storage: this page only */ } };
  function setMode(mode, to) {
    try { localStorage.setItem(MODE_KEY, mode); if (mode === 'first-run') localStorage.removeItem('temper-greenfield-first-run'); } catch { /* none */ }
    location.href = to;
  }

  // The projects there are: the first run has none until Ready makes one.
  const projects = () => (FIRST ? (S.setup.made ? { default: FR.project } : {}) : M.projects);
  if (projects()[params.get('project')]) S.project = params.get('project');
  const pid = () => S.project;
  function P(k = pid()) {
    const p = projects()[k];
    if (!p) return null;
    const x = { ...p, name: S.names[k] || p.name, repo: S.adopted[k] || p.repo };
    if (FIRST) { const f = forgeInfo(); const m = modelInfo(); x.forge = f && f.name; x.model = m && FR.models[m.way].line; }
    return x;
  }
  const modelLine = () => P().model || 'Claude, through your subscription · Opus';
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
  const slug = t => t.toLowerCase().replace(/[^a-z0-9]+/g, '-').slice(0, 28).replace(/^-+|-+$/g, '') || 'untitled';
  const wayName = id => (P().ways.find(w => w.id === id) || { name: id }).name;
  const needSentence = (n, still) => n === 0 ? (still ? 'Nothing else needs you.' : 'Nothing needs you.') : n === 1 ? (still ? 'One thing still needs you.' : 'One thing needs you.') : `${NUM[n] || n} things ${still ? 'still need' : 'need'} you.`;

  // ---- The first run's set-up, from timestamps, so that every tab agrees --
  function forgeInfo() {
    const f = S.setup.forge;
    if (!f) return null;
    const now = Date.now();
    if (f.kind === 'github') {
      const done = now - f.at >= FR.github.ms;
      return { kind: 'github', name: 'GitHub', done, nextAt: done ? null : f.at + FR.github.ms, doing: 'Installing temper’s App on GitHub', line: `temper’s App on ${FR.github.account}, ${FR.github.repos.length} repositories` };
    }
    const steps = forgejoSteps(f);
    let t = now - f.at;
    let i = 0;
    let next = f.at;
    for (const s of steps) { next += s.ms; if (t < s.ms) break; t -= s.ms; i += 1; }
    const done = i >= steps.length;
    return { kind: 'forgejo', name: FR.forgejo.name, done, i, steps, nextAt: done ? null : next, doneAt: next, doing: done ? '' : steps[i].doing, line: `connected as ${FR.forgejo.login}${startedLine(f)}` };
  }
  // What Forgejo on this machine starts with (the sheet below): an empty
  // repository, copies of public ones, or nothing; each goes into the
  // first project when Ready makes it.
  const startRepos = f => (!f || f.kind !== 'forgejo' ? [] : f.start === 'copy' ? f.copies.map(c => c.to) : f.start === 'none' ? [] : [`${f.org}/${f.repo}`]);
  function forgejoSteps(f) {
    const extra = f.start === 'copy'
      ? f.copies.map(c => ({ ms: FR.forgejo.copyMs, doing: `Copying ${c.to} from ${c.host}`, done: `Copied ${c.to} from ${c.host}` }))
      : f.start === 'none' ? []
      : [{ ms: FR.forgejo.makeMs, doing: `Making ${f.org}/${f.repo}, empty, with a README and a CI check`, done: `Made ${f.org}/${f.repo}, empty, with a README and a CI check` }];
    return [...FR.forgejo.steps, ...extra];
  }
  function startedLine(f) {
    const r = startRepos(f);
    if (!r.length) return '';
    const what = r.length === 1 ? r[0] : `${r.length} repositories`;
    const verb = f.start === 'copy' ? 'copied' : 'made';
    return S.setup.made ? ` · ${what}, in ${S.names.default || FR.project.name}` : ` · ${what} ${verb}`;
  }
  function modelInfo() {
    const m = S.setup.model;
    return m ? { ...FR.models[m.way], ...m, ready: m.state === 'ready' } : null;
  }
  const setupReady = () => !!(forgeInfo()?.done && modelInfo()?.ready);

  // ---- Acts --------------------------------------------------------------
  function answer(id, idx, reply) {
    const n = needs().find(x => x.id === id);
    if (!n) return;
    const c = reply != null ? { label: `Replied: “${reply.length > 60 ? `${reply.slice(0, 57)}…` : reply}”`, effect: { line: 'Read your reply; carrying on with it in mind.' } } : n.choices[idx];
    const prev = { goal: n.goal ? S.goalPatch[n.goal] : undefined, chat: n.chat ? S.chatPatch[n.chat] : undefined };
    S.answered[id] = { label: c.label, at: Date.now(), prev };
    delete S.later[id];
    const e = c.effect;
    if (e && e.accept) { save(); const said = acceptProposal(e.accept.chat, e.accept.key); save(); render(); toast(said); return; }
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
  // The banner's project: the open one, the others, rename in place and a
  // new project (draft/first-run-projects.md, 7).
  let renaming = false;
  function switcher() {
    const rows = Object.keys(projects()).map(k => {
      const p = P(k);
      if (k === pid() && renaming) return `<form class="rename" data-rename-form><input class="text" value="${esc(p.name)}" aria-label="Project name" data-rename-input><button class="btn primary small">Save</button></form>`;
      return `<button type="button" data-project="${esc(k)}"${k === pid() ? ' aria-current="true"' : ''}>${esc(p.name)} <small>${esc(p.repo || 'no repository')}</small></button>`;
    }).join('');
    return `<details class="switcher"${renaming ? ' open' : ''}><summary>${esc(P().name)}</summary><div class="switcher-menu">
      ${rows}
      <div class="switcher-acts">${renaming ? '' : `<button type="button" data-rename>Rename ${esc(P().name)}</button>`}<button type="button" data-new-project>+ New project</button><a href="settings.html">Settings for ${esc(P().name)}</a></div></div></details>`;
  }
  function navItems() {
    const pt = parts();
    const n = open().length;
    return [
      ['home', 'home.html', 'Home'],
      ['needs', 'needs.html', 'Needs you', n],
      pt.goals && ['goals', 'goals.html', 'Goals'],
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
      <a class="new" href="chat.html?id=new">+ New chat</a>
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
  const kindWord = n => ({ question: 'a question', funds: 'funds', review: 'a review', proposal: 'a proposal' }[n.kind] || n.kind);
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
  // Home of the project the wizard just made: what is connected, and the
  // three ways to begin, each ticked once done.
  function freshHome() {
    const p = P();
    const f = forgeInfo();
    const m = modelInfo();
    const cs = chats();
    const configured = Object.keys(S.accepted).length > 0;
    const asked = cs.some(c => !c.configure);
    const gh = f.kind === 'github';
    const adoptAct = gh
      ? `<label class="sr" for="adopt-pick">Repository</label><select class="text" id="adopt-pick">${FR.github.repos.map(r => `<option>${esc(r)}</option>`).join('')}</select><button type="button" class="btn small" data-adopt="">Adopt</button>`
      : '<button type="button" class="btn small" data-repo-open="empty">Make an empty one</button><button type="button" class="btn small" data-repo-open="copy">Copy a public one</button><button type="button" class="btn quiet small" data-later-note="The adoption form: any repository on your Forgejo, one you pushed yourself among them.">Adopt another…</button>';
    const several = !!p.repo && p.repo.includes(', ');
    const copied = S.setup.forge.start === 'copy' || S.copiedLater;
    const steps = [
      { done: configured, title: 'Tell temper how you work', text: configured ? 'Accepted: the project now shows what your way of working needs.' : 'In your own words: how you plan, review and land work. temper proposes how it will run that, and nothing changes until you accept.', act: configured ? '<a class="link" href="settings.html">See the settings</a>' : `<a class="btn primary small" href="${cs.find(c => c.configure) ? `chat.html?id=${cs.find(c => c.configure).id}` : 'chat.html?id=new&configure=1'}">${cs.find(c => c.configure) ? 'Go on' : 'Start'}</a>` },
      { done: !!p.repo, title: p.repo ? `${p.repo} ${several ? 'are' : 'is'} in ${p.name}` : gh ? 'Adopt a repository' : 'Add a repository', text: p.repo ? (copied ? `Chats can change ${several ? 'their' : 'its'} code now, on your Forgejo; nothing goes back where ${several ? 'they were' : 'it was'} copied from.` : 'Chats can change its code now; its CI ran on main and passed.') : gh ? `temper’s App reaches ${FR.github.repos.length} repositories on ${FR.github.account}. Until one is in, chats talk and plan but change no code.` : 'Your Forgejo has none yet. Until one is in, chats talk and plan but change no code.', act: p.repo ? '' : adoptAct },
      { done: asked, title: 'Start a chat', text: asked ? 'Your chats are under Chats.' : 'Ask a question or ask for a change.', act: asked ? '<a class="link" href="chats.html">Chats</a>' : '<a class="btn small" href="chat.html?id=new">New chat</a>' },
    ];
    const left = steps.filter(s => !s.done).length;
    const headline = left === 3 ? 'You’re set up. Nothing has run yet.' : left ? `${NUM[left]} ${left === 1 ? 'thing' : 'things'} left to begin with.` : `${p.name} is ready for work.`;
    const o = open();
    return frame('home', `<div class="col">
      <header class="intro">
        <p class="kicker">${esc(p.name)} · made just now</p>
        <h1 class="headline">${esc(headline)}</h1>
        <p class="lede">Your agents run on your ${esc(m.name)}. ${gh ? `temper’s App is on ${esc(FR.github.account)}’s GitHub.` : 'Forgejo runs on this machine.'}${p.repo ? '' : ` ${esc(p.name)} has no repository yet.`}</p>
      </header>
      ${o.length ? `<section class="section"><div class="section-head"><h2 class="section-title signal">Needs you · ${o.length}</h2></div>${askCard(o[0])}</section>` : ''}
      <section class="section" aria-labelledby="begin">
        <div class="section-head"><h2 class="section-title" id="begin">To begin</h2></div>
        <ol class="begin">${steps.map((s, i) => `<li class="${s.done ? 'done' : ''}"><span class="mark" aria-hidden="true">${s.done ? '✓' : i + 1}</span><div><p class="t">${esc(s.title)}</p><p class="x">${esc(s.text)}</p>${s.act ? `<div class="acts">${s.act}</div>` : ''}</div></li>`).join('')}</ol>
      </section>
      ${cs.length ? `<section class="section" aria-labelledby="chats"><div class="section-head"><h2 class="section-title" id="chats">Chats</h2><a class="link" href="chats.html">All ${cs.length}</a></div>
        <div class="lines">${cs.slice(0, 4).map(c => `<a href="chat.html?id=${encodeURIComponent(c.id)}"><span>${esc(c.title)}</span><span class="when">${esc(c.status)} · ${esc(c.when)}</span></a>`).join('')}</div></section>` : ''}
      <section class="section" aria-labelledby="conn">
        <div class="section-head"><h2 class="section-title" id="conn">Connected</h2></div>
        <div class="lines">
          <div><span>${esc(f.name)} <span class="muted">· ${esc(f.line)}</span></span><span class="when">${gh ? '<a class="link" href="#" data-later-note="github.com, in a new tab.">Open ↗</a>' : '<a class="link" href="#" data-later-note="Forgejo opens in a new tab, signed in through temper.">Open Forgejo ↗</a>'}</span></div>
          <div><span>${esc(m.name)} <span class="muted">· ${esc(m.who)} · ${esc(m.model)}</span></span><span class="when"><a class="link" href="#" data-later-note="Accounts: this one, and a way to add others.">Accounts</a></span></div>
          <div><span>Project ${esc(p.name)} <span class="muted">· ${esc(p.repo || 'no repository yet')}</span></span><span class="when"><button type="button" class="link plain" data-rename>Rename</button></span></div>
        </div>
      </section>
      ${!gh && !p.repo ? repoSheet('home') : ''}
    </div>`);
  }

  function home() {
    if (P().fresh) return freshHome();
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
        <p class="facts">A chat suits one change. For bigger work, a chat can write a draft beside it and start a goal from it: a plan, its tasks, landing. <a class="link" href="settings.html">Turn on drafts and goals</a></p>
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
      <header class="intro"><p class="kicker">${esc(P().name)} · goals</p><h1 class="headline">${all.length ? `${all.length} goals, ${all.filter(pick.running).length + all.filter(pick.needs).length} under way.` : 'No goals yet.'}</h1>
      <p class="lede">A goal turns a draft into landed work, its own way: ${esc(P().ways.map(w => w.name.toLowerCase()).join(', '))}.</p></header>
      <section class="section"><div class="section-head"><div class="filters">${fb('all', 'All')}${fb('needs', 'Needs you')}${fb('running', 'Under way')}${fb('ended', 'Ended')}</div>${parts().drafts ? '<a class="link" href="chats.html?f=drafts">Start one from a draft</a>' : ''}</div>
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
    if (!d) return 'chats.html';
    return d.chat ? `chat.html?id=${encodeURIComponent(d.chat)}` : d.doc ? `chat.html?draft=${encodeURIComponent(d.id)}` : 'chats.html';
  }

  // Chats: every conversation, each saying what it made, and the drafts
  // no chat has taken up yet.
  function chatsPage() {
    const pt = parts();
    if (params.get('f')) S.chatFilter = params.get('f');
    if (!pt.drafts && S.chatFilter === 'drafts') S.chatFilter = 'all';
    const f = S.chatFilter;
    const ds = pt.drafts ? drafts() : [];
    const writes = c => !!(c.writes && pt.drafts);
    const cs = chats().filter(c => f === 'all' || (f === 'live' ? c.live : f === 'drafts' ? writes(c) : !c.live));
    const loose = ds.filter(d => !d.chat && (f === 'all' || f === 'drafts'));
    const fb = (k, l) => `<button type="button" aria-pressed="${f === k}" data-chat-filter="${k}">${l}</button>`;
    const title = d => { const t = d.id.replace(/\.md$/, '').replace(/-/g, ' '); return t[0].toUpperCase() + t.slice(1); };
    const chatRow = c => {
      const d = writes(c) && ds.find(x => x.id === c.writes);
      const made = writes(c) ? `writes ${c.writes}${d && d.goal ? ` · started goal ${d.goal}` : ''}` : c.goal ? `started goal ${c.goal}` : '';
      const st = /waiting/.test(c.status) ? `<span class="state-needs">${esc(c.status)}</span>` : esc(c.status);
      return `<a class="goal-row${c.live ? '' : ' ended'}" href="chat.html?id=${encodeURIComponent(c.id)}"><span class="top mono"><span>${esc(c.id)}${made ? ` · ${esc(made)}` : ''}</span><span>${st} · ${esc(c.when)}</span></span><span class="line">${esc(c.title)}</span></a>`;
    };
    const draftRow = d => `<a class="goal-row" href="chat.html?draft=${encodeURIComponent(d.id)}"><span class="top mono"><span>${esc(d.path)}</span><span>${d.goal ? `started goal ${esc(d.goal)}` : 'draft, not discussed'} · ${esc(d.updated)}</span></span><span class="line">${esc(title(d))}</span></a>`;
    return frame('chats', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · chats</p><h1 class="headline">${P().repo ? `Talk to an agent about ${esc(P().repo)}.` : 'Talk to an agent.'}</h1>
      <p class="lede">A chat answers a question or changes the code${pt.drafts ? ', and can write a draft beside it' : ''}${pt.goals ? '. A draft becomes a goal for bigger work' : ''}.</p></header>
      ${P().repo ? '' : `<section class="band quiet-band" aria-label="No repository"><p><span class="tag">No repository yet</span>Chats in ${esc(P().name)} talk and plan, but change no code until a repository is in.</p><a class="btn small" href="home.html">Adopt a repository</a></section>`}
      <section class="section"><div class="section-head"><div class="filters">${fb('all', 'All')}${fb('live', 'Live')}${pt.drafts ? fb('drafts', 'Drafts') : ''}${fb('closed', 'Closed')}</div><a class="btn small" href="chat.html?id=new">+ New chat</a></div>
      ${cs.length || loose.length ? `<div class="goal-rows">${cs.map(chatRow).join('')}${loose.map(draftRow).join('')}</div>` : '<p class="empty">None.</p>'}</section>
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
    const sw = (k, label, small, on = 'Shown', off = 'Hidden') => `<div class="setting"><div><span>${label}</span><small>${small}</small></div><label class="toggle-row"><span>${pt[k] ? on : off}</span><input type="checkbox" class="switch" data-part="${k}"${pt[k] ? ' checked' : ''} aria-label="${label}"></label></div>`;
    const sel = (k, label, small, opts) => `<div class="setting"><div><label for="s-${k}">${label}</label>${small ? `<small>${small}</small>` : ''}</div><select id="s-${k}" data-set="${k}">${opts.map(o => `<option${setting(k, opts[0]) === o ? ' selected' : ''}>${esc(o)}</option>`).join('')}</select></div>`;
    const kinds = setting('kinds', { impl: true, retry: true, words: true, deps: false });
    const kind = (k, l) => `<label><input type="checkbox" data-kind="${k}"${kinds[k] ? ' checked' : ''}> ${l}</label>`;
    return frame('settings', `<div class="col">
      <header class="intro"><p class="kicker">${esc(P().name)} · settings</p><h1 class="headline">How work runs in ${esc(P().name)}.</h1>
      <p class="lede">Everything the other pages do for this project comes from here, so each project, and each person, can work its own way.</p></header>
      <nav class="tabs" aria-label="Settings"><a href="#" aria-current="page">How work runs</a><a href="#" data-later-note="Repositories: ${esc(P().repo ? `${P().repo} on ${P().forge}` : 'none yet')}.">Repositories</a><a href="#" data-later-note="Forges: not drawn in this experiment.">Forges</a><a href="#" data-later-note="Accounts: not drawn in this experiment.">Accounts</a><a href="#" data-later-note="Builders: a separate app; temper only names which to use.">Builders</a></nav>
      <div class="settings">
        <section class="section" style="gap:0"><h2 class="section-title" style="margin-bottom:6px">What the project shows</h2>
          ${sw('drafts', 'Chats can write drafts', 'A chat keeps a document beside it, in docs/design/draft/, that a goal can start from.', 'On', 'Off')}
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
    const pt = parts();
    const draftId = id === 'new' ? null : params.get('draft');
    if (id === 'new' && params.get('configure')) c = { id: null, title: 'Tell temper how you work', configure: true, status: 'new', thread: [], replies: [], writes: null, model: modelLine(), fresh: true };
    else if (id === 'new') c = { id: null, title: 'New chat', status: 'new', thread: [], replies: ['Here’s a first outline; it’s in the draft beside us.', 'Done; the change is highlighted in the draft.', 'Noted, it’s in.'], writes: pt.drafts && params.get('draft') ? 'untitled.md' : null, model: modelLine(), fresh: true };
    else if (draftId) { const d = drafts().find(x => x.id === draftId); c = { id: null, title: d.id, status: 'new', thread: [], replies: ['Read it. What would you like to change first?', 'Done; the change is highlighted in the draft.'], writes: d.id, model: modelLine(), fresh: true }; }
    else c = chat(id) || chats()[0];
    const d = c.writes && pt.drafts ? (drafts().find(x => x.id === c.writes) || { id: c.writes, path: `docs/design/draft/${c.writes}`, status: 'writing', updated: 'not yet' }) : null;
    const docSecs = d ? (S.newDocs[c.id || ''] || d.doc || []) : [];
    const key = c.writes || 'none';
    const thread = [...c.thread, ...(S.threads[c.id] || [])];
    const def = draftId && !thread.length ? 'full' : 'off';
    const mode = d ? (S.panel[key] || def) : 'off';
    const shownMode = narrow() && mode === 'split' ? 'off' : mode;
    const changed = S.docChanged[key] ?? d?.changed;
    const started = d && (S.draftGoal[d.id] || d.goal);
    const segBtn = (m, label, icon, cls = '') => `<button type="button" class="${cls}" data-mode="${m}" aria-pressed="${shownMode === m}" title="${label}">${ICON[icon]}<span>${label}</span></button>`;
    const turn = t => t.who === 'proposal' ? proposalCard(t, c)
      : t.who === 'you' ? `<p class="turn-you">${esc(t.text)}</p>`
      : t.who === 'agent' ? `<p class="turn-agent">${esc(t.text)}</p>`
      : t.who === 'typing' ? '<div class="typing" aria-label="Writing"><i></i><i></i><i></i></div>'
      : `<p class="turn-tool">${ICON.pen}<span>${esc(t.text.replace(/ [^ ]+$/, ''))}</span><span class="file">${esc(t.text.split(' ').pop())}</span><span>${esc(t.detail || '')}</span>${t.ref && d ? `<button type="button" data-show="${esc(t.ref)}">Show</button>` : ''}${t.goal ? `<a class="link" href="goal.html?g=${encodeURIComponent(t.goal)}">Open</a>` : ''}</p>`;
    const where = ['Chats', 'chats.html'];
    return `<div class="session-app">
      <header class="session-top">
        <nav class="crumbs" aria-label="Where"><a class="back" href="${where[1]}" aria-label="Back">‹</a><span class="trail"><a href="home.html">${esc(p.name)}</a> / <a href="${where[1]}">${where[0]}</a> /</span><strong>${esc(c.title)}</strong></nav>
        <div class="right">
          <span class="byline" style="white-space:nowrap">${c.id ? `${esc(c.id)} · ` : ''}${esc(c.model)}</span>
          ${d ? `<div class="seg" role="group" aria-label="The draft" data-key="${esc(key)}">${segBtn('off', 'Chat', 'chat')}${segBtn('split', 'Split', 'split', 'split')}${segBtn('full', 'Draft', 'doc')}</div>` : ''}
          ${!d && pt.drafts ? '<button type="button" class="btn small" data-add-draft>+ Draft</button>' : ''}
          ${d && pt.goals && docSecs.length ? (started ? `<a class="btn small" href="goal.html?g=${encodeURIComponent(started)}">Goal ${esc(started)} →</a>` : `<button type="button" class="btn primary small" data-start-goal>Start goal…</button>`) : ''}
        </div>
      </header>
      <div class="session" data-mode="${mode}" data-key="${esc(key)}">
        <section class="convo" aria-label="Conversation">
          <div class="thread" id="thread"><div class="thread-inner">
            ${thread.length ? '' : c.configure ? `<div class="starter"><p class="meta-line">temper’s configuring agent. Describe how you work, in your own words or by pointing at a document. It proposes how temper will run that; nothing changes until you accept.</p><div class="starters">${FR.configure.starters.map(x => `<button type="button" class="btn small" data-starter="${esc(x)}">${esc(x)}</button>`).join('')}</div></div>`
              : `<p class="meta-line">${d ? `A chat that writes ${esc(d.id)}. Say what it should hold.` : p.repo ? `Say what you want; the agent works in ${esc(p.repo)}.` : `Say what you want. ${esc(p.name)} has no repository yet, so the agent can talk and plan but not change code.`}</p>`}
            ${thread.map(turn).join('')}
          </div></div>
          <div class="convo-foot">${composer(`data-chat-say="${esc(c.id || '')}" data-writes="${esc(c.writes || '')}"${c.configure ? ' data-configure="1"' : ''}`, c.configure && c.fresh ? 'How do you work?' : c.fresh ? (d ? 'What should this draft hold?' : 'What do you want done?') : `Reply to ${c.id}`, d ? 'Enter sends · Shift+Enter for a new line · Ctrl+\\ switches the draft' : 'Enter sends · Shift+Enter for a new line')}</div>
        </section>
        ${d ? `<article class="doc" aria-label="${esc(d.id)}"><div class="doc-inner">
          <p class="doc-path"><span>${esc(d.path)}</span><span>${d.chat || c.id ? `edited ${esc(d.updated)}` : esc(d.status)}</span></p>
          ${docSecs.length ? docSecs.map(s => `<section class="doc-sec${s.id === changed ? ' changed' : ''}" data-sec="${esc(s.id)}">${s.html}${s.id === changed ? `<p class="changed-note">Changed by ${esc(c.id || 'the agent')} just now</p>` : ''}</section>`).join('') : '<p class="doc-empty">The draft appears here as you write it together.</p>'}
        </div>${mode === 'full' && !thread.length ? `<div class="doc-foot">${composer(`data-chat-say="" data-writes="${esc(c.writes)}"`, 'Ask about this draft, or say what to change', '')}</div>` : ''}</article>` : ''}
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

  // The configuring chat's turns: an explanation, or a proposal in the
  // person's words with the settings underneath, which an owner accepts
  // (draft/configuring.md, 1). A proposal waiting is also in Needs you.
  function configureAnswer(c, th, text) {
    if (/not sure|what can/i.test(text)) { th.push({ who: 'agent', text: FR.configure.explain }); return; }
    const key = /draft|design|plan|goal|task/i.test(text) ? 'drafts' : 'small';
    const replaced = th.filter(t => t.who === 'proposal' && !t.accepted && !t.superseded);
    for (const t of replaced) t.superseded = true;
    for (const n of S.extraNeeds[pid()] || []) if (n.chat === c.id && !S.answered[n.id]) S.answered[n.id] = { label: 'Replaced', at: 0, prev: {} };
    th.push({ who: 'agent', text: replaced.length ? 'I’ve changed the proposal to follow that.' : FR.configure.proposals[key].intro });
    th.push({ who: 'proposal', key });
    S.chatPatch[c.id] = { ...(S.chatPatch[c.id] || {}), status: 'waiting on you' };
    list(S.extraNeeds).push({ id: `p-${c.id}-${th.length}`, chat: c.id, kind: 'proposal', asked: 'just now', short: 'Accept how temper will run your work?', question: `Accept how temper will run ${P().name}?`, context: FR.configure.proposals[key].lines.map(l => l[1]).join(' · '), choices: [{ label: 'Accept', rec: true, effect: { accept: { chat: c.id, key } } }] });
  }
  function acceptProposal(cid, key) {
    const pr = FR.configure.proposals[key];
    const before = parts();
    S.parts[pid()] = { ...pr.parts };
    (S.settings[pid()] = S.settings[pid()] || {}).defaultWay = pr.way;
    S.accepted[cid] = key;
    const th = (S.threads[cid] = S.threads[cid] || []);
    const t = [...th].reverse().find(x => x.who === 'proposal' && x.key === key && !x.superseded);
    if (t) t.accepted = true;
    th.push({ who: 'agent', text: `Accepted. New work in ${P().name} follows it from now on; nothing was running yet. Ask me to change any of it whenever you like.` });
    S.chatPatch[cid] = { ...(S.chatPatch[cid] || {}), status: 'accepted', live: false };
    for (const n of S.extraNeeds[pid()] || []) if (n.chat === cid && !S.answered[n.id]) S.answered[n.id] = { label: 'Accepted', at: Date.now(), prev: {} };
    const words = { goals: 'Goals', changes: 'Changes', away: 'the away switch' };
    const shown = Object.keys(words).filter(k => pr.parts[k] && !before[k]).map(k => words[k]);
    const drafts = pr.parts.drafts && !before.drafts ? 'chats can write drafts' : '';
    const pages = shown.length ? `${shown.join(', ').replace(/, ([^,]*)$/, ' and $1')} now ${shown.length === 1 ? 'shows' : 'show'}` : '';
    return `Accepted${drafts || pages ? `: ${[drafts, pages].filter(Boolean).join(', and ')}` : ''}.`;
  }
  function proposalCard(t, c) {
    const pr = FR.configure.proposals[t.key];
    const state = t.accepted ? '<p class="proposal-state ok">✓ Accepted · new work follows it</p>'
      : t.superseded ? '<p class="proposal-state">Replaced by the proposal below</p>'
      : `<div class="acts"><button type="button" class="btn primary small" data-accept="${esc(c.id)}" data-proposal="${esc(t.key)}">Accept</button><button type="button" class="btn quiet small" data-focus-composer>Change something</button></div>`;
    return `<article class="proposal${t.superseded ? ' superseded' : ''}" aria-label="Proposal">
      <p class="proposal-title">How temper will run ${esc(P().name)}</p>
      <ul>${pr.lines.map(([words, set]) => `<li><span>${esc(words)}</span><small>${esc(set)}</small></li>`).join('')}</ul>
      <p class="proposal-split"><span><b>temper enforces</b> ${esc(pr.enforced)}.</span> <span><b>Agents follow</b> ${esc(pr.followed)}.</span></p>
      ${state}</article>`;
  }

  // ---- The first run: the box's terminal and the welcome's wizard --------
  const WIZ = ['welcome', 'forge', 'model', 'ready'];
  const WIZ_NAMES = { welcome: 'Welcome', forge: 'Forge', model: 'Model', ready: 'Ready' };
  const utc = t => `${new Date(t).toISOString().slice(11, 19)}Z`;
  const notFirstRun = what => `<div class="setup"><main class="setup-main"><header class="setup-head"><p class="kicker">Mockup</p><h1 class="headline">${esc(what)} belongs to the first run.</h1><p class="lede">The everyday scenario is open: fabrig and hello, long set up. The first run starts from a fresh temper box.</p></header><div class="acts"><button type="button" class="btn primary" data-first-run>Start the first run</button><a class="btn" href="home.html">Back to fabrig</a></div></main></div>`;

  function boxPage() {
    if (!FIRST) return notFirstRun('The box’s terminal');
    const f = forgeInfo();
    const events = [];
    if (f && f.kind === 'forgejo') {
      events.push(`${utc(S.setup.forge.at)} Forge ${FR.forgejo.n} (Forgejo): starting`);
      if (f.done) events.push(`${utc(f.doneAt)} Forge ${FR.forgejo.n} (Forgejo): started`);
    }
    return `<div class="box-page">
      <header class="setup-head">
        <p class="kicker">The first run · your terminal</p>
        <h1 class="headline">You ran temper box.</h1>
        <p class="lede">It starts temper here, in the foreground, and prints a link that signs you in as its owner. After that it stays quiet: a line when something starts, ends or fails. Open the link.</p>
      </header>
      <div class="term" role="log" aria-label="Terminal">
        <div class="term-bar"><i></i><i></i><i></i><span>owner@laptop: ~</span></div>
<pre>$ temper box
First run: wrote the configuration, ~/.config/temper/temper.json
Logs: ~/.local/share/temper/box/logs/
Starting serve

Open this link in a browser on this machine. It signs you in as this
temper’s owner, once and for a short while, so keep it private.
Opened again once spent, it prints a fresh one here.

<a href="welcome.html" target="_blank" data-link>${esc(FR.link)}</a>

Ctrl-C stops temper.
${events.length ? `\n${events.map(esc).join('\n')}\n` : ''}<span class="cursor"></span></pre>
      </div>
      <p class="facts">The link opens the welcome in a new tab; this tab keeps showing what the box prints. Each step’s own lines go to its log, so the terminal shows only the box’s.</p>
    </div>`;
  }

  function welcomePage() {
    if (!FIRST) return notFirstRun('The welcome');
    const at = WIZ.includes(params.get('at')) ? params.get('at') : 'welcome';
    const f = forgeInfo();
    const m = modelInfo();
    const done = { welcome: at !== 'welcome' || !!f, forge: !!f, model: !!m?.ready, ready: S.setup.made };
    const steps = `<ol class="steps">${WIZ.map((k, i) => `<li class="${done[k] && k !== at ? 'done' : ''}"${k === at ? ' aria-current="step"' : ''}><a href="welcome.html?at=${k}"><span class="n">${done[k] && k !== at ? '✓' : i + 1}</span><span class="l">${WIZ_NAMES[k]}</span></a></li>`).join('')}</ol>`;
    const prev = WIZ[Math.max(0, WIZ.indexOf(at) - 1)];
    const foot = `<nav class="setup-foot"><a class="link" href="welcome.html?at=${prev}">‹ Back</a>${f ? '' : '<button type="button" class="link plain" data-playground>Try the playground instead</button>'}</nav>`;
    const head = (h, l) => `<header class="setup-head"><h1 class="headline">${h}</h1>${l ? `<p class="lede">${l}</p>` : ''}</header>`;
    const choice = (title, text, meta, acts, cls = '') => `<section class="choice${cls}"><div><h2>${title}</h2><p>${text}</p>${meta ? `<p class="meta">${meta}</p>` : ''}</div><div class="acts">${acts}</div></section>`;
    let body;
    if (at === 'welcome') {
      body = `<header class="setup-head"><p class="kicker">Reliable, scalable and flexible software factory.</p><h1 class="headline">Welcome to temper</h1><p class="lede">You describe <strong>what</strong> you want and <strong>how</strong> you want it done, and it happens.</p></header>
        <div class="choices-grid">
          ${choice('Set up temper', 'Your forge: a Forgejo temper runs for you, or GitHub. Your model: ChatGPT or Claude.', 'About five minutes', `<a class="btn primary" href="welcome.html?at=${f ? (m?.ready ? 'ready' : 'model') : 'forge'}">${f ? 'Go on setting up' : 'Set up temper'}</a>`, ' lead')}
          ${choice('Try the playground', 'A scripted model and a forge, built in. A taste of how temper feels, with nothing to connect.', 'Fully offline · Ready at once', '<button type="button" class="btn" data-playground>Try the playground</button>')}
        </div>
        <p class="facts">Tried the playground first? Set up whenever you like; the playground then steps aside.</p>`;
    } else if (at === 'forge') {
      if (f && f.kind === 'github' && !f.done) {
        body = `${head('Connect a forge')}<div class="away-note"><span class="spin" aria-hidden="true"></span><div><p><b>On GitHub, in this tab</b></p><p class="muted">You name temper’s App, choose the account it goes on and the repositories it may use, and GitHub sends this tab back here.</p><p class="mock-inline">Mockup: GitHub’s own pages are skipped; back in a moment.</p></div></div>`;
      } else if (f) {
        body = `${head('Connect a forge', 'One forge is enough to begin; add others later, from Settings.')}${forgeStatus(f)}<div class="acts"><a class="btn primary" href="welcome.html?at=${m?.ready ? 'ready' : 'model'}">Continue</a></div>${foot}`;
      } else {
        body = `${head('Connect a forge', 'Where your code lives: you tell temper what you want, agents push and land code.')}
          <div class="choices-grid">
            ${choice('Forgejo', 'A forge of your own: temper starts it on this machine and manages it. You can import existing repositories into it, from GitHub or elsewhere.', 'Here · about a minute', '<button type="button" class="btn primary" data-forgejo-open>Run on this machine</button>', ' lead')}
            ${choice('GitHub', 'Your repositories on GitHub. temper makes its own App there; you choose the account and the repositories it may use.', 'In this tab, on GitHub · about two minutes', '<button type="button" class="btn" data-github>Connect GitHub</button>')}
          </div>${foot}${repoSheet('welcome')}`;
      }
    } else if (at === 'model') {
      const way = params.get('way');
      const status = f ? forgeStatus(f) : '';
      if (m && m.state === 'away') {
        body = `${status}${head('Connect a model')}<div class="away-note"><span class="spin" aria-hidden="true"></span><div><p><b>ChatGPT’s sign-in, in this tab</b></p><p class="muted">You sign in and approve temper; ChatGPT sends this tab back to temper’s listener on this machine, and on to Ready.</p><p class="mock-inline">Mockup: ChatGPT’s own pages are skipped; back in a moment.</p></div></div>`;
      } else if (m && ['finishing', 'checking'].includes(m.state)) {
        body = `${status}${head('Connect a model')}<div class="away-note"><span class="spin" aria-hidden="true"></span><div><p><b>${m.state === 'finishing' ? 'Finishing your sign-in…' : 'Checking Claude Code’s sign-in on this machine…'}</b></p></div></div>`;
      } else if (way === 'paste') {
        body = `${status}${head('Signing in to ChatGPT from another computer')}
          <section class="panel">
            <ol class="numbered"><li><a class="link" href="#" data-later-note="ChatGPT’s sign-in opens in a new tab; never a live sign-in in this mockup.">Open ChatGPT’s sign-in ›</a> and approve temper.</li><li>When ChatGPT is done, its tab shows a page that can’t be reached. Copy that page’s whole address from the address bar.</li><li>Paste it here.</li></ol>
            <label class="field-v"><span>Address of the page that couldn’t be reached</span><input class="text" type="password" id="paste-address" autocomplete="off" data-enables="paste-finish"></label>
            <div class="acts"><button type="button" class="btn primary" id="paste-finish" data-paste-finish disabled>Finish connecting</button><a class="btn quiet" href="welcome.html?at=model">All ways</a></div>
          </section>${foot}`;
      } else if (way === 'key') {
        body = `${status}${head('An OpenAI API key')}
          <section class="panel">
            <p>From OpenAI’s platform, under API keys. What temper’s agents use is billed to the key’s organization.</p>
            <label class="field-v"><span>Key, pasted once</span><input class="text" type="password" id="api-key" autocomplete="off" data-enables="key-add"></label>
            <div class="acts"><button type="button" class="btn primary" id="key-add" data-add-key disabled>Add key</button><a class="btn quiet" href="welcome.html?at=model">All ways</a></div>
            <p class="facts">Nothing checks a key when it is added: a wrong one shows once OpenAI refuses a run.</p>
          </section>${foot}`;
      } else if (way === 'claude') {
        body = `${status}${head('Your Claude subscription')}
          <section class="panel">
            <p>temper sets Claude Code up on this machine in an environment of its own: its settings, sessions and sign-in stay apart from your own Claude Code. In a terminal here:</p>
            <pre class="cmd">temper program-login claude-owner</pre>
            <p class="muted">It makes that environment, then Claude Code asks you to sign in to your subscription there. temper never reads its login.</p>
            <div class="acts"><button type="button" class="btn primary" data-claude-check>I’ve signed in</button><a class="btn quiet" href="welcome.html?at=model">All ways</a></div>
          </section>${foot}`;
      } else if (m?.ready) {
        body = `${status}${head('Connect a model')}<div class="forge-status done"><p><span class="ok">✓</span><b>${esc(m.name)}</b><span class="sep">·</span><span class="muted">${esc(m.who)}</span></p></div><div class="acts"><a class="btn primary" href="welcome.html?at=ready">Continue</a></div>${foot}`;
      } else {
        body = `${status}${head('Connect a model', 'temper’s agents run on a plan you sign in to.')}
          <div class="choices-grid">
            ${choice('ChatGPT', 'Your ChatGPT plan: sign in, and temper’s agents use its included usage.', 'Sign in, in this tab · about a minute', '<button type="button" class="btn primary" data-chatgpt>Continue with ChatGPT</button>', ' lead')}
            ${choice('Claude', 'Your Claude subscription, through Claude Code in an environment of its own on this machine, apart from yours.', 'One command in a terminal, on this machine', '<a class="btn" href="welcome.html?at=model&amp;way=claude">Use your Claude subscription</a>')}
          </div>${foot}`;
      }
    } else {
      const ready = setupReady();
      const making = !!S.setup.making;
      const row = (cls, label, what, extra = '') => `<li class="${cls}"><span class="mark" aria-hidden="true">${cls === 'done' ? '✓' : ''}</span><div><p><b>${label}</b> <span class="muted">${what}</span></p>${extra}</div></li>`;
      const forgeRow = !f ? row('todo', 'Forge', 'none yet · <a class="link" href="welcome.html?at=forge">Connect a forge</a>')
        : f.done ? row('done', 'Forge', `${esc(f.name)} · ${esc(f.line)}`)
        : row('current', 'Forge', `${esc(f.name)} · ${esc(f.doing)}…`, f.kind === 'forgejo' ? forgeSteps(f) : '');
      const modelRow = !m ? row('todo', 'Model', 'none yet · <a class="link" href="welcome.html?at=model">Connect a model</a>')
        : m.ready ? row('done', 'Model', `${esc(m.name)} · ${esc(m.who)}`)
        : row('current', 'Model', `${esc(m.name)} · signing in…`);
      const into = startRepos(S.setup.forge);
      const intoWords = into.length === 1 ? into[0] : `${into.length} repositories`;
      const readyLine = into.length ? `Everything is connected. Create your first project, with ${intoWords} in it, and start working.` : 'Everything is connected. Create your first project and start working.';
      // The first project: named here, "Default" unless changed, and
      // made by its button; a rename later is the project's own.
      const projectForm = `<form class="first-project" data-first-project>
          <label for="first-project-name">Your first project</label>
          <div class="first-project-row"><input class="text" id="first-project-name" value="${esc(FR.project.name)}" autocomplete="off" spellcheck="false"${making ? ' disabled' : ''}><button type="submit" class="btn primary"${ready && !making ? '' : ' disabled'}>${making ? `Creating ${esc(S.setup.name || FR.project.name)}…` : 'Create your project'}</button></div>
          <p class="sheet-error" id="first-project-error" role="alert" hidden></p>
          <p class="facts">Rename it whenever you like.</p>
        </form>`;
      body = `${head(ready ? 'Ready' : 'Almost ready', ready ? esc(readyLine) : 'Your first project can be created once your forge and model are ready.')}
        <ul class="checklist">${forgeRow}${modelRow}</ul>
        ${projectForm}
        ${ready || !(f && !f.done) ? '' : '<p class="facts">Forgejo is still setting up; this page moves on by itself.</p>'}${foot}`;
    }
    const server = '<a class="link" href="#" data-later-note="temper help server: a long-lived deployment, the engine behind a proxy and its workers on another host. Opens in a new tab.">Run temper on a server ↗</a>';
    return `<div class="setup"><header class="setup-top"><a class="brand" href="welcome.html">temper</a><p class="box-line"><b>temper box</b> · on this machine, to try it · more later, from Settings · ${server}</p>${steps}</header><main class="setup-main">${body}</main></div>`;
  }
  // "Run on this machine" opens it: what Forgejo starts with, an empty
  // repository by default. From the project's home, the same sheet adds
  // a repository later, made or copied, into the project at once.
  function repoSheet(ctx) {
    const welcome = ctx === 'welcome';
    const opt = (v, title, text, checked) => `<label class="option"><input type="radio" name="start" value="${v}"${checked ? ' checked' : ''}><b>${title}</b><small>${text}</small></label>`;
    return `<dialog class="sheet" id="repo-sheet" aria-labelledby="repo-title"><form method="dialog" data-repo-form="${ctx}" data-pick="empty">
      <div class="sheet-head"><h2 id="repo-title">${welcome ? 'Forgejo on this machine' : `Add a repository to ${esc(P().name)}`}</h2><p class="muted">${welcome ? 'temper starts it here and connects to it; you connect your model meanwhile.' : 'Made or copied on your Forgejo, then in this project at once.'}</p></div>
      <fieldset><legend>${welcome ? 'Start it with' : 'A repository'}</legend>
        ${opt('empty', 'An empty repository', 'A README and a starter CI check.', true)}
        <div class="option-more more-empty fields"><div><label for="fj-org">Organization</label><input class="text" id="fj-org" value="${esc(FR.forgejo.org)}" autocomplete="off"></div><div><label for="fj-repo">Name</label><input class="text" id="fj-repo" value="${esc(FR.forgejo.repo)}" autocomplete="off"></div></div>
        ${opt('copy', 'Copies of public repositories', 'Copied into your Forgejo from GitHub or another git host, to play around with. Nothing is sent back.', false)}
        <div class="option-more more-copy"><label for="fj-copies">Their addresses, one per line</label><textarea class="text addresses" id="fj-copies" rows="3" placeholder="${esc(FR.forgejo.copyHint)}" spellcheck="false"></textarea></div>
        ${welcome ? opt('none', 'Nothing for now', 'Your first project opens with no repository. Make or copy one later from temper’s UI, or adopt one you push yourself.', false) : ''}
      </fieldset>
      <p class="sheet-error" id="repo-error" role="alert" hidden></p>
      <div class="foot"><button type="button" class="btn quiet" data-close>Cancel</button><button type="submit" class="btn primary" autofocus>${welcome ? 'Run on this machine' : 'Add it'}</button></div>
    </form></dialog>`;
  }
  // Ready's Create: the project as named, then its home.
  function firstProject(f) {
    const name = f.querySelector('#first-project-name').value.trim();
    const err = f.querySelector('#first-project-error');
    if (!name) { err.textContent = 'A name for it: Default, or one of yours.'; err.hidden = false; return; }
    S.setup.making = Date.now(); S.setup.name = name; save(); render();
    setTimeout(() => {
      S = load(); Object.assign(S.setup, { made: true, making: 0 }); S.project = 'default';
      if (name !== FR.project.name) S.names.default = name;
      const r = startRepos(S.setup.forge); if (r.length) S.adopted.default = r.join(', ');
      save(); location.href = 'home.html';
    }, 1300);
  }
  // An address per line: host/owner/name, or owner/name on GitHub; a
  // copy keeps the owner and the name on the local Forgejo.
  function parseCopies(text) {
    const out = [];
    for (const raw of text.split('\n')) {
      const line = raw.trim().replace(/^https?:\/\//, '').replace(/\.git$/, '').replace(/\/+$/, '');
      if (!line) continue;
      const parts = line.split('/');
      if (parts.length < 2 || parts.length > 3 || parts.some(x => !x)) return { bad: raw.trim() };
      const host = parts.length === 3 ? parts[0] : 'github.com';
      const [o, n] = parts.slice(-2);
      out.push({ host, from: `${host}/${o}/${n}`, to: `${o}/${n}` });
    }
    return { copies: out };
  }
  function repoChosen(f) {
    const start = f.querySelector('input[name="start"]:checked').value;
    const err = f.querySelector('#repo-error');
    const say = t => { err.textContent = t; err.hidden = false; };
    const v = id => f.querySelector(`#${id}`).value.trim();
    let pick = { start };
    if (start === 'empty') {
      if (!v('fj-org') || !v('fj-repo')) return say('Both an organization and a name.');
      pick = { start, org: v('fj-org'), repo: v('fj-repo') };
    } else if (start === 'copy') {
      const r = parseCopies(v('fj-copies'));
      if (r.bad) return say(`Not an address: ${r.bad}. Write it as ${FR.forgejo.copyHint}.`);
      if (!r.copies.length) return say(`At least one address, as ${FR.forgejo.copyHint}.`);
      pick = { start, copies: r.copies };
    }
    f.closest('dialog').close();
    if (f.dataset.repoForm === 'welcome') {
      S.setup.forge = { kind: 'forgejo', at: Date.now(), ...pick };
      save();
      location.href = 'welcome.html?at=model';
      return;
    }
    const r = pick.start === 'copy' ? pick.copies.map(c => c.to) : [`${pick.org}/${pick.repo}`];
    toast(pick.start === 'copy' ? `Copying ${r.join(', ')}…` : `Making ${r[0]}…`);
    setTimeout(() => { S = load(); S.adopted[pid()] = r.join(', '); S.copiedLater = pick.start === 'copy'; save(); render(); toast(`${r.join(', ')} ${r.length === 1 ? 'is' : 'are'} in ${P().name}.`); }, 1500);
  }
  function forgeSteps(f) {
    return `<details class="each" id="forge-steps"><summary>Each step</summary><ol>${f.steps.map((s, i) => `<li class="${i < f.i ? 'done' : i === f.i ? 'current' : ''}">${esc(i < f.i ? s.done : s.doing)}</li>`).join('')}</ol></details>`;
  }
  function forgeStatus(f) {
    return `<div class="forge-status${f.done ? ' done' : ''}"><p>${f.done ? '<span class="ok">✓</span>' : '<span class="spin" aria-hidden="true"></span>'}<b>${esc(f.name)}</b><span class="sep">·</span><span class="muted">${esc(f.done ? f.line : `${f.doing}…`)}</span></p>${!f.done && f.kind === 'forgejo' ? forgeSteps(f) : ''}</div>`;
  }

  // The phone frame: any page at a phone's size, beside the laptop view.
  function phonePage() {
    const src = params.get('p') || (FIRST ? 'welcome.html' : 'home.html');
    document.body.style.background = '#e4e4e0';
    const links = FIRST ? [['welcome.html', 'Welcome'], ['welcome.html?at=forge', 'Forge'], ['welcome.html?at=model', 'Model'], ['welcome.html?at=ready', 'Ready'], ['home.html', 'Home, once made'], ['chats.html', 'Chats'], ['settings.html', 'Settings']]
      : [['home.html', 'Home'], ['needs.html', 'Needs you'], ['goals.html', 'Goals'], ['goal.html', 'A goal'], ['chats.html', 'Chats'], ['chat.html?id=T4', 'Draft session'], ['settings.html', 'Settings']];
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
    const everyday = '<a href="home.html">Home</a><a href="needs.html">Needs you</a><a href="goals.html">Goals</a><a href="goal.html?g=first-run-projects">A goal</a><a href="chat.html?id=T4">Draft session</a><a href="chats.html">Chats</a><a href="changes.html">Changes</a><a href="settings.html">Settings</a>';
    const first = `<a href="box.html">Terminal</a><a href="welcome.html?at=welcome">Welcome</a><a href="welcome.html?at=forge">Forge</a><a href="welcome.html?at=model">Model</a><a href="welcome.html?at=ready">Ready</a>${S.setup.made ? '<a href="home.html">Home</a><a href="chats.html">Chats</a><a href="settings.html">Settings</a>' : ''}`;
    return `<button type="button" class="mock-toggle" data-mock aria-expanded="false">Mockup</button>
      <div class="mock-panel" id="mock-panel">
        <h3>Scenario</h3>
        <label><input type="radio" name="mode" value="everyday"${FIRST ? '' : ' checked'}> Everyday: fabrig and hello</label>
        ${FIRST ? '' : `<div class="sub"><label><input type="radio" name="scenario" value="morning"${S.scenario === 'morning' ? ' checked' : ''}> Morning check-in, after 9 hours</label>
        <label><input type="radio" name="scenario" value="away"${S.scenario === 'away' ? ' checked' : ''}> Back from away</label></div>`}
        <label><input type="radio" name="mode" value="first-run"${FIRST ? ' checked' : ''}> First run: a fresh temper box, real backends</label>
        <p style="color:#52616f;font-size:12px">${FIRST ? 'Its own state: fabrig and hello are kept as you left them.' : 'Scenarios are fabrig’s; hello is a small new project. Switch projects with the project menu.'}</p>
        <h3>Pages</h3>
        <div class="pages">${FIRST ? first : everyday}</div>
        <a href="phone.html?p=${encodeURIComponent(here)}">Open this page in a phone frame</a>
        ${FIRST ? '<button type="button" class="reset" data-first-run>Start the first run over</button>' : '<button type="button" class="reset" data-reset>Reset everything</button>'}
      </div>`;
  }

  // ---- Drawing and wiring -----------------------------------------------
  const PAGES = { home, needs: needsPage, goals: goalsPage, goal: goalPage, chats: chatsPage, changes: changesPage, settings: settingsPage, chat: chatPage, phone: phonePage, box: boxPage, welcome: welcomePage };
  const TITLES = { home: 'Home', needs: 'Needs you', goals: 'Goals', goal: params.get('g') || 'Goal', chats: 'Chats', changes: 'Changes', settings: 'Settings', chat: 'Session', phone: 'Phone frame', box: 'Terminal', welcome: 'Welcome' };
  const APP_PAGES = ['home', 'needs', 'goals', 'goal', 'chats', 'chat', 'changes', 'settings'];
  // The first run has no project until Ready makes one: its pages are the
  // welcome's until then, as the link is.
  function guard() {
    if (FIRST && !S.setup.made && APP_PAGES.includes(page)) { location.replace(S.setup.forge || S.setup.model ? 'welcome.html?at=ready' : 'welcome.html'); return true; }
    if (FIRST && S.setup.made && page === 'welcome' && !params.get('at')) { location.replace('home.html'); return true; }
    return false;
  }
  let renderTimer;
  function render() {
    const scrollY = window.scrollY;
    const thread = document.getElementById('thread');
    const threadAtEnd = !thread || thread.scrollHeight - thread.scrollTop - thread.clientHeight < 40;
    const panelOpen = document.getElementById('mock-panel')?.classList.contains('open');
    // What the viewer typed or opened survives a redraw.
    const typed = [...document.querySelectorAll('input[id]:not([type=radio]):not([type=checkbox]), textarea[id]')].map(el => [el.id, el.value]);
    const opened = [...document.querySelectorAll('details[id][open]')].map(el => el.id);
    const focused = document.activeElement?.id;
    document.body.innerHTML = (PAGES[page] || home)() + mockCorner();
    for (const [id, v] of typed) { const el = document.getElementById(id); if (el) { el.value = v; if (el.dataset.enables) document.getElementById(el.dataset.enables).disabled = !v.trim(); } }
    for (const id of opened) { const el = document.getElementById(id); if (el) el.open = true; }
    if (focused) document.getElementById(focused)?.focus();
    if (panelOpen) document.getElementById('mock-panel')?.classList.add('open');
    document.title = `${TITLES[page] || 'temper'} · ${P()?.name || 'first run'} · temper`;
    window.scrollTo(0, scrollY);
    const t = document.getElementById('thread');
    if (t && threadAtEnd) t.scrollTop = t.scrollHeight;
    if (renaming) { const i = [...document.querySelectorAll('[data-rename-input]')].find(x => x.offsetParent); if (i && document.activeElement !== i) { i.focus(); i.select(); } }
    // A goal still planning, a question not shown yet, or a forge setting
    // up draws again when due.
    const due = P() ? [...goals().filter(g => g.state === 'planning' && g.created).map(g => g.created + PLAN_MS), ...(S.extraNeeds[pid()] || []).filter(n => n.showAfter > Date.now()).map(n => n.showAfter)] : [];
    const f = FIRST && forgeInfo();
    if (f && f.nextAt) due.push(f.nextAt);
    clearTimeout(renderTimer);
    if (due.length) renderTimer = setTimeout(render, Math.max(50, Math.min(...due) - Date.now() + 50));
  }

  function sendChat(form, text) {
    let cid = form.dataset.chatSay;
    const writes = form.dataset.writes || null;
    if (!cid) {
      const n = Math.max(P().chatBase ?? 6, ...chats().map(c => parseInt(c.id.slice(1), 10) || 0)) + 1;
      cid = `T${n}`;
      const title = text.length > 42 ? `${text.slice(0, 40)}…` : text;
      const isDraft = writes === 'untitled.md';
      const docName = isDraft ? `${slug(text)}.md` : writes;
      const configure = !!form.dataset.configure;
      list(S.newChats).unshift({ id: cid, title: configure ? 'Tell temper how you work' : isDraft ? `${title}, draft` : title, configure, status: 'working', live: true, when: 'now', writes: docName, model: modelLine(), thread: [], replies: ['Here’s a first outline; it’s in the draft beside us.', 'Done; the change is highlighted in the draft.', 'Noted, it’s in.'] });
      if (isDraft) { list(S.newDrafts).unshift({ id: docName, chat: cid, status: 'writing', updated: 'just now', path: `docs/design/draft/${docName}`, doc: [] }); if (S.panel['untitled.md']) S.panel[docName] = S.panel['untitled.md']; }
      else if (writes) { S.draftChat[writes] = cid; S.panel[writes] = 'split'; }
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
      if (c.configure) configureAnswer(c, th, text);
      else th.push({ who: 'agent', text: c.replies[said % c.replies.length] });
      const w = c.writes;
      if (w) {
        let secs = S.newDocs[cid];
        const d = drafts().find(x => x.id === w);
        if (!d?.doc?.length && !secs) {
          secs = S.newDocs[cid] = [
            { id: 'top', html: `<h1>${esc(c.title.replace(/, draft$/, ''))}</h1><p class="doc-muted">Draft, 2026-11-04, with the user.</p>` },
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

  // The wizard's acts that leave and come back (a sign-in, GitHub): the
  // mockup skips the other side and returns after a moment.
  function modelStep(m, ms) {
    S.setup.model = { ...m, at: Date.now() };
    save();
    render();
    setTimeout(() => { S = load(); S.setup.model = { ...S.setup.model, state: 'ready' }; save(); location.href = 'welcome.html?at=ready'; }, ms);
  }
  function firstRunClick(d) {
    const v = id => document.getElementById(id)?.value.trim();
    if ('firstRun' in d) setMode('first-run', 'welcome.html');
    else if ('playground' in d) toast('The playground isn’t drawn in this experiment; the current mockups have it.');
    else if ('forgejoOpen' in d || d.repoOpen) { const s = document.getElementById('repo-sheet'); if (d.repoOpen) { s.querySelector(`input[name="start"][value="${d.repoOpen}"]`).checked = true; s.querySelector('form').dataset.pick = d.repoOpen; } s.showModal(); if (d.repoOpen === 'copy') s.querySelector('#fj-copies').focus(); }
    else if ('github' in d) { S.setup.forge = { kind: 'github', at: Date.now() }; save(); render(); setTimeout(() => { location.href = 'welcome.html?at=model'; }, FR.github.ms); }
    else if ('chatgpt' in d) modelStep({ way: 'chatgpt', state: 'away' }, 2200);
    else if ('pasteFinish' in d) modelStep({ way: 'chatgpt', state: 'finishing' }, 1000);
    else if ('claudeCheck' in d) modelStep({ way: 'claude', state: 'checking' }, 1300);
    else if ('addKey' in d) { S.setup.model = { way: 'key', state: 'ready', at: Date.now() }; save(); location.href = 'welcome.html?at=ready'; }
    else if ('adopt' in d) { const repo = d.adopt || v('adopt-pick'); S.adopted[pid()] = repo; save(); render(); toast(`${repo} is in ${P().name}. Its CI ran on main and passed.`); }
    else if ('rename' in d) { renaming = true; render(); }
    else if ('newProject' in d) toast('+ New project: a name, then Create makes it on the stock policy and opens it, with no repository. Not drawn yet.');
    else if (d.starter) { const ta = document.querySelector('.convo-foot textarea'); ta.value = d.starter; ta.closest('form').requestSubmit(); }
    else if (d.accept) { const said = acceptProposal(d.accept, d.proposal); save(); render(); toast(said); }
    else if ('focusComposer' in d) document.querySelector('.convo-foot textarea')?.focus();
    else return false;
    return true;
  }
  document.addEventListener('click', e => {
    const t = e.target.closest('button, a, summary');
    if (!t) return;
    const d = t.dataset;
    if (firstRunClick(d)) return;
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
    else if ('addDraft' in d) {
      const c = params.get('id') !== 'new' && chat(params.get('id'));
      if (!c) { S.panel['untitled.md'] = 'split'; save(); location.href = 'chat.html?id=new&draft=1'; return; }
      const name = `${slug(c.title)}.md`;
      list(S.newDrafts).unshift({ id: name, chat: c.id, status: 'writing', updated: 'just now', path: `docs/design/draft/${name}`, doc: [] });
      S.chatPatch[c.id] = { ...(S.chatPatch[c.id] || {}), writes: name };
      S.panel[name] = 'split';
      save(); render();
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
    if (t.name === 'start' && t.form) { t.form.dataset.pick = t.value; t.form.querySelector('#repo-error').hidden = true; }
    else if (t.name === 'mode') setMode(t.value, t.value === 'first-run' ? 'box.html' : 'home.html');
    else if (t.name === 'scenario') { S.scenario = t.value; S.project = 'fabrig'; S.away.fabrig = t.value === 'away'; save(); render(); }
    else if (d.part) { (S.parts[pid()] = S.parts[pid()] || {})[d.part] = t.checked; save(); render(); }
    else if (d.way) { mine().defaultWay = d.way; save(); }
    else if (d.set) { mine()[d.set] = t.value; save(); }
    else if (d.kind) { const k = { ...setting('kinds', { impl: true, retry: true, words: true, deps: false }) }; k[d.kind] = t.checked; mine().kinds = k; save(); }
  });
  document.addEventListener('input', e => {
    if (e.target.dataset?.enables) { document.getElementById(e.target.dataset.enables).disabled = !e.target.value.trim(); return; }
    const ta = e.target.closest('.composer textarea');
    if (!ta) return;
    ta.style.height = 'auto';
    ta.style.height = `${Math.min(ta.scrollHeight, 220)}px`;
    ta.closest('.composer').querySelector('button').disabled = !ta.value.trim();
  });
  document.addEventListener('keydown', e => {
    const ta = e.target.closest && e.target.closest('.composer textarea');
    if (ta && e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); ta.closest('form').requestSubmit(); }
    if (renaming && e.key === 'Escape') { renaming = false; render(); }
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
    if (f.dataset.repoForm) { e.preventDefault(); repoChosen(f); return; }
    if ('firstProject' in f.dataset) { e.preventDefault(); firstProject(f); return; }
    if ('renameForm' in f.dataset) {
      e.preventDefault();
      const v = f.querySelector('input').value.trim();
      renaming = false;
      if (v) S.names[pid()] = v;
      save(); render();
      if (v) toast(`Renamed: ${v}.`);
      return;
    }
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
  window.addEventListener('storage', e => {
    if (e.key === MODE_KEY) location.reload();
    else if (e.key === KEY) { S = load(); if (!guard()) render(); }
  });
  matchMedia('(max-width: 760px)').addEventListener('change', render);
  if (guard()) return;
  render();
})();
