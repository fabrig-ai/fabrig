// temper's client, drawn from a world at the step in the address
// (?world=, ?step=), with the mockups' bar above each page. Every page
// reads the same world, so the pages agree at each step.
//
// Every value a page shows goes through src(): an engine fact, a name
// from the configuration, words as a person or an agent wrote them, or
// an ops application's report. What the page code writes itself is the
// UI's own words, which name no workflow: a workflow's words come from
// its world's data, with their writer.
(function () {
  const C = window.MOCKUPS;
  const params = new URLSearchParams(location.search);
  const page = document.body.dataset.page;

  function stored(key) { try { return localStorage.getItem(key); } catch { return null; } }
  function store(key, value) { try { localStorage.setItem(key, value); } catch { /* no storage in previews */ } }

  // ---- The world, and the step ----------------------------------------

  let worldKey = params.get('world') || stored('mockups-world') || 'reference';
  if (!C.worlds[worldKey]) worldKey = 'reference';
  store('mockups-world', worldKey);
  const W = C.worlds[worldKey];

  const ids = W.steps.map(s => s.id);
  let stepId = params.get('step') || stored(`mockups-step-${worldKey}`) || ids[0];
  if (!ids.includes(stepId)) stepId = ids[0];
  store(`mockups-step-${worldKey}`, stepId);
  const stepIndex = ids.indexOf(stepId);
  const step = W.steps[stepIndex];

  const world = W.initial();
  for (const s of W.steps.slice(0, stepIndex + 1)) {
    world.now = s.hour * 60;
    s.apply(world, s.id);
  }
  // A world may take choices the person made on its way from the address,
  // such as the repositories given to GitHub's installation.
  if (W.choices) W.choices(world, params);

  // Annotations show unless hidden; sources and the future show when
  // asked. `?bare` draws the product alone, for a capture: no bar, no
  // marks; `?future` draws its future pieces too. View's Hide this bar
  // does the same from page to page, keeping the future as it was set,
  // until Esc or the corner's handle brings the bar back.
  const FLAGS = { plain: 'mock-plain', sources: 'mock-sources', future: 'mock-future' };
  const BARLESS = 'mock-barless';
  const bare = params.has('bare');
  const barless = !bare && stored(BARLESS) === '1';
  if (bare || barless) document.body.classList.add(FLAGS.plain);
  if (barless) {
    document.body.classList.add(BARLESS);
    if (stored(FLAGS.future) === '1') document.body.classList.add(FLAGS.future);
  } else if (!bare) {
    for (const [flag, cls] of Object.entries(FLAGS)) if (stored(cls) === '1') document.body.classList.add(cls);
    document.body.classList.add('mock');
  }
  if (params.has('future')) document.body.classList.add(FLAGS.future);

  // The pages under the README's rules (README.md, section 5; STATUS.md,
  // section 1): realistic, with their future pieces marked.
  const REVIEWED = { welcome: '2026-10-25', github: '2026-10-25' };

  // ---- Words and links ------------------------------------------------

  const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  function esc(s) { return String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c])); }
  function date(min) { return new Date(W.HOUR0 + min * 60000); }
  function dayOf(min) { return Math.floor((W.HOUR0 + min * 60000) / 86400000); }
  function hm(min) { const d = date(min); return `${String(d.getUTCHours()).padStart(2, '0')}:${String(d.getUTCMinutes()).padStart(2, '0')}`; }
  function clock(min) { return dayOf(min) === dayOf(world.now) ? hm(min) : `${DAY[date(min).getUTCDay()]} ${hm(min)}`; }
  function span(minutes) {
    const m = Math.round(minutes);
    if (m < 60) return `${m} min`;
    const h = Math.floor(m / 60), r = m % 60;
    return r ? `${h} h ${r} min` : `${h} h`;
  }
  function ago(min) {
    const d = world.now - min;
    if (d < 1) return 'just now';
    if (d < 60 * 12) return `${span(d)} ago`;
    const days = dayOf(world.now) - dayOf(min);
    if (days === 0) return `at ${hm(min)}`;
    if (days === 1) return 'yesterday';
    return DAY[date(min).getUTCDay()];
  }
  function ordinal(n) { return `${n}${['th', 'st', 'nd', 'rd'][(n % 100 - 20) % 10] || ['th', 'st', 'nd', 'rd'][n % 100] || 'th'}`; }
  function plural(n, one, many) { return `${n} ${n === 1 ? one : (many || `${one}s`)}`; }
  const cap = s => s.charAt(0).toUpperCase() + s.slice(1);

  // A link within the mockups keeps the world and the step, or names a
  // later step, where an act's outcome is drawn.
  function u(target, at = stepId) {
    if (/^(https?:|#)/.test(target)) return target;
    const [path, hash] = target.split('#');
    const [file, query] = path.split('?');
    const p = new URLSearchParams(query || '');
    if (!p.has('world')) p.set('world', worldKey);
    if (!p.has('step')) p.set('step', at);
    for (const kept of ['project', 'granted', 'pick']) if (params.has(kept) && !p.has(kept)) p.set(kept, params.get(kept));
    if (bare) p.set('bare', '');
    if (params.has('future') && !p.has('future')) p.set('future', '');
    return `${file}?${p}${hash ? `#${hash}` : ''}`;
  }

  const T = id => world.tasks[id];
  const kids = id => world.children[id] || [];
  const requesterOf = t => t.requester || t.parent;
  const isChat = t => t.template === world.config.templates.chat;
  function isLive(id) {
    const t = T(id);
    return t.kind === 'change' ? t.stage !== 'landed' : !['done', 'failed', 'cancelled'].includes(t.phase);
  }
  const liveKids = id => kids(id).filter(isLive);
  function descendants(id) { return kids(id).flatMap(k => [k, ...descendants(k)]); }
  function href(id) {
    const t = T(id);
    if (!t) return '#';
    return u(isChat(t) ? `chat.html?task=${id}` : `task.html?task=${id}`);
  }
  function who(by) { const t = T(by); return t ? `${by} ${t.title}` : by; }
  function liveRuns(worker) { return Object.keys(world.tasks).filter(k => T(k).worker === worker && isLive(k) && T(k).kind !== 'change' && (T(k).phase === 'running' || T(k).run)); }
  const fresh = thing => (thing && thing.at === stepId ? ' fresh' : '');

  // A subset of Markdown, as jig's client renders what agents write.
  function inline(s) { return esc(s).replace(/`([^`]+)`/g, '<code>$1</code>').replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>'); }
  // A person's words keep their line breaks; an agent's lines join.
  function md(text, br = ' ') {
    const lines = String(text).split('\n');
    let out = '';
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];
      if (!line.trim()) { i++; continue; }
      if (line.startsWith('|')) {
        const rows = [];
        while (i < lines.length && lines[i].startsWith('|')) rows.push(lines[i++]);
        const cells = r => r.slice(1, -1).split('|').map(c => c.trim());
        const [head, , ...body] = rows;
        const h = cells(head);
        out += `<table class="md-table">${h.some(Boolean) ? `<thead><tr>${h.map(c => `<th>${inline(c)}</th>`).join('')}</tr></thead>` : ''}<tbody>${body.map(r => `<tr>${cells(r).map(c => `<td>${inline(c)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
        continue;
      }
      if (line.startsWith('- ')) {
        const items = [];
        while (i < lines.length && lines[i].startsWith('- ')) items.push(lines[i++].slice(2));
        out += `<ul>${items.map(x => `<li>${inline(x)}</li>`).join('')}</ul>`;
        continue;
      }
      const para = [];
      while (i < lines.length && lines[i].trim() && !lines[i].startsWith('|') && !lines[i].startsWith('- ')) para.push(lines[i++]);
      out += `<p>${para.map(inline).join(br)}</p>`;
    }
    return `<div class="md">${out}</div>`;
  }
  const firstLine = text => String(text).split('\n').find(l => l.trim()).replace(/\*\*|`/g, '');

  // ---- Sources --------------------------------------------------------

  const KINDS = {
    engine: 'Engine fact',
    config: 'Configuration',
    written: 'As written',
    ops: 'Ops application',
  };
  // A value of blocks is marked as a block; anything else, inline.
  function src(kind, where, html) {
    const tag = /^\s*<(div|table|ul|ol|p|dl)\b/.test(html) ? 'div' : 'span';
    return `<${tag} class="src src-${kind}" data-src="${esc(where)}">${html}</${tag}>`;
  }
  const fact = (where, html) => src('engine', where, html);
  const conf = (where, html) => src('config', where, html);
  function said(by, html, at) { return src('written', `Written by ${who(by)}${at != null ? `, ${clock(at)}` : ''}`, html); }

  // Annotations, beside the product's words: the reference case's names,
  // and the gaps temper lacks for what a page draws (the README lists
  // them).
  function ref(name) { return name ? `<i class="case-ref" title="The reference case's name">${esc(name)}</i>` : ''; }
  function gap(id, what) { return `<i class="gap" title="temper-on-temper.md ${esc(id)}: ${esc(what)}. Drawn ahead of temper's design.">gap ${esc(id)}</i>`; }

  // A piece the engine can't supply yet, shown only under View → Future,
  // marked with what would supply it: a mission, a draft, or "not
  // designed". `variant` gives what the page shows today in its place.
  // A need is its mark's few words and what would supply it.
  const NEEDS = {
    display: ['the account’s email', 'the account’s display, its email, which jig’s sign-in leaves empty (jig’s client/domain/contract.md, 7.1, OwnerAccountLine.display)'],
    preview: ['the resolved default', 'the model a new chat’s first run takes, before it runs: jig’s Template.resolved_default (jig’s client/domain/contract.md, 5.3)'],
    ci: ['a CI read at main', 'the verdict of CI at a repository’s main: a later mission (design STATUS.md, first-repo)'],
    projectless: ['no project at first', 'temper init seeding no project, which the playground’s start or the first repository makes (draft first-run.md, 2.4; design STATUS.md, first-run-ux-2)'],
    eligible: ['jig’s not-eligible end', 'a sign-in’s ending for an account not eligible for plan usage (client README, section 11)'],
    port: ['jig’s port-taken end', 'a sign-in’s ending for the listener’s port held by another program (client README, section 11)'],
    login: ['the person’s login', 'the person’s Forgejo login on the owned forge’s row (client forge.md, 5.4)'],
    grant: ['not designed', 'an installation’s choice of repositories, all or those selected, and their count'],
  };
  function future(need, html) {
    const [label, detail] = NEEDS[need] || [need, need];
    const tag = /^\s*<(div|section|table|ul|ol|p|dl|details|h[1-6])\b/.test(html) ? 'div' : 'span';
    return `<${tag} class="future" data-need="${esc(label)}"><i class="future-mark" title="${esc(`Future: needs ${detail}`)}">future · ${esc(label)}</i>${html}</${tag}>`;
  }
  function variant(need, today, then) {
    const tag = /^\s*<(div|section|table|ul|ol|p|dl|details|h[1-6])\b/.test(today) ? 'div' : 'span';
    return `${today ? `<${tag} class="today">${today}</${tag}>` : ''}${future(need, then)}`;
  }

  function ahead(label, what) { return `<i class="gap" title="Drawn ahead of temper’s design: ${esc(what)}">ahead · ${esc(label)}</i>`; }

  // ---- A task's facts, in words ---------------------------------------

  // When jig computes news, what would wake the task, from its wake
  // policy: a party's words, for a task a party requested; another task's
  // state, by a subscription. A timer alone is a time (phaseWords).
  function newsWords(t) {
    const w = t.wake || {};
    const req = requesterOf(t);
    if (w.words && req && !T(req)) return req === world.person ? 'your words' : `${req}’s words`;
    if (w.on) return `${w.on}’s end`;
    return null;
  }
  function waitsWords(id) {
    const t = T(id);
    switch (t.waitsOn) {
      case 'delegates': return plural(liveKids(id).length, 'delegate');
      case 'news': return newsWords(t) || 'news';
      case 'party': return t.waitsFor === world.person ? 'an answer from you' : `an answer from ${t.waitsFor}`;
      default: return 'nothing';
    }
  }
  function wakeWords(t, short) {
    if (!t.wake) return '';
    if (t.wake.every) return short ? `wakes ${hm(t.wake.next)}` : `wakes ${hm(t.wake.next)}, every ${span(t.wake.every)}`;
    return '';
  }
  function changeWords(id, t, short) {
    const pos = world.repos[t.repo].queue.indexOf(id);
    const repo = short ? '' : ` in ${t.repo}`;
    switch (t.stage) {
      case 'checking': return short ? 'checking' : 'checking · CI at its head';
      case 'queued': return `queued · ${ordinal(pos + 1)}${repo}`;
      case 'first': return `first${repo}${t.updating ? (short ? ' · updating' : ' · updating from main') : ''}`;
      case 'landed': return `landed ${clock(t.ended)}`;
      default: return t.stage;
    }
  }
  function phaseWords(id, short) {
    const t = T(id);
    if (t.kind === 'change') return changeWords(id, t, short);
    switch (t.phase) {
      case 'idle': {
        if (t.waitsOn === 'news' && !newsWords(t) && t.wake && t.wake.every) return `waiting until ${hm(t.wake.next)}${short ? '' : `, every ${span(t.wake.every)}`}`;
        const w = wakeWords(t, short);
        return `waiting on ${waitsWords(id)}${w ? ` · ${w}` : ''}`;
      }
      case 'running': return `running · ${t.run.doing}${short ? '' : ` · ${span(world.now - t.run.since)}`}`;
      case 'done': return `done ${clock(t.ended)}`;
      default: return t.phase;
    }
  }
  function phaseWhere(id) {
    const t = T(id);
    if (t.kind === 'change') return `temper: ${id}'s stage, in its forge connector's words`;
    const parts = [`jig: ${id}'s phase`];
    if (t.phase === 'idle') parts.push(`what the engine computes it waits on: ${t.waitsOn === 'party' ? 'a party, for an entry in its subtree' : t.waitsOn}`);
    if (t.waitsOn === 'news' && t.wake && t.wake.words && !T(requesterOf(t))) parts.push(`narrowed by its wake policy: a party's words wake it, and ${requesterOf(t) === world.person ? 'you' : requesterOf(t)} requested it`);
    if (t.waitsOn === 'news' && t.wake && t.wake.on) parts.push(`narrowed by its subscription to ${t.wake.on}'s state: done, held or its result`);
    if (t.wake && t.wake.every) parts.push(`its wake policy's timer, every ${span(t.wake.every)}`);
    if (t.phase === 'running') parts.push("its host's last report");
    return parts.join('; ');
  }
  const phase = (id, short) => fact(phaseWhere(id), esc(phaseWords(id, short)));
  function stateClass(id) {
    const t = T(id);
    if (t.kind === 'change') return t.stage === 'landed' ? 'done' : 'queued';
    return t.phase === 'idle' ? 'waiting' : t.phase;
  }

  function kind(id) {
    const t = T(id);
    if (t.kind === 'change') return fact(`temper: a change, its forge connector's procedure, into ${world.repos[t.repo].name}`, esc(`change · ${t.repo}`));
    return conf(`${t.template ? `template "${t.template}" and ` : ''}charter "${t.charter}", named in the deployment's configuration`, esc(t.template ? `${t.template} · ${t.charter}` : t.charter));
  }
  function chip(id) {
    const t = T(id);
    if (!t) return esc(id);
    return fact(`${id}'s chip: its number, and its title as ${who(t.by)} wrote it`, `<a class="task-chip" href="${href(id)}"><b>${id}</b> ${esc(t.title)}</a>`) + ref(t.ref);
  }
  // A task's card, in temper's words (client README, 5.7): its kind is its
  // result contract's, its state the document's.
  function contract(id) { const t = T(id); return t.kind === 'change' ? 'CHANGE' : (t.contract || 'report').toUpperCase(); }
  function cardState(id) {
    const t = T(id);
    if (t.kind === 'change') return t.stage === 'landed' ? 'Landed' : 'Running';
    if (t.phase === 'held') return 'Held';
    return { done: 'Completed', failed: 'Failed', cancelled: 'Cancelled' }[t.phase] || (isLive(id) ? 'Running' : 'Ended');
  }
  const spentThisPeriod = () => Object.values(world.tasks).reduce((n, t) => n + (t.spend || 0), 0) + world.ended.reduce((n, e) => n + (e.spend || 0), 0);

  // An account and the line naming what a run takes, in temper's words
  // (client README, 5.5).
  const account = name => world.accounts.find(a => a.name === name);
  function accountName(a) {
    if (a.display) return a.display;
    const program = a.sources.find(s => s.kind === 'program');
    const signIn = a.sources.find(s => s.kind === 'sign-in');
    if (program) return `${a.plan} · ${program.name}`;
    if (signIn) return `${a.name} · ${signIn.name}`;
    return `${a.provider} API key`;
  }
  function usingWords(a, modelName) {
    const first = a.sources[0];
    return first.kind === 'program' ? `Using your ${a.plan}, through ${first.program} · ${modelName}`
      : first.kind === 'sign-in' ? `Using ${a.plan} · ${modelName} · Manage usage`
      : `Using your ${a.provider} API key · ${modelName}`;
  }
  // A plan's Manage usage is an external link (6.5).
  function usingHtml(a, modelName) {
    const words = esc(usingWords(a, modelName));
    return a.sources[0].kind === 'sign-in' ? words.replace(/Manage usage$/, `<a class="text-link" href="#" data-later="${esc(MANAGE)}">Manage usage ↗</a>`) : words;
  }
  // While the demo's model holds, its line names it, and "Default" for
  // the model while none is requested (5.6).
  const DEMO_LINE = () => fact('temper: the demo’s model half holds, and no model is requested (client README, 5.6)', 'Using Scripted model · Default');
  function usingLine(id) {
    if (modelHalf()) return DEMO_LINE();
    const t = T(id);
    if (!account(t.account)) return noModelLine();
    return fact(`jig: the account and model ${id}'s next run takes; the account's first source, in temper's words`, usingHtml(account(t.account), t.model));
  }
  function newChatLine() {
    if (modelHalf()) return DEMO_LINE();
    const n = world.config.next;
    return n ? fact('jig: the account and model a new chat’s first run takes: its charter’s first model with a usable account', usingHtml(account(n.account), n.model)) : noModelLine();
  }

  // The demo (client README, 4.8): its model's half holds while no
  // account of another provider is, and its forge's while no forge but
  // its own is connected. Its words are 5.6's.
  const demo = world.demo;
  const demoRunning = () => !!demo && (!demo.state || demo.state === 'running');
  const usableModel = () => world.accounts.some(a => a.sources.length);
  const ownForge = () => world.forges.some(f => f.mode !== 'fake' && f.state === 'connected');
  const tryDemo = (label = 'Try the playground') => `<a href="${esc(u('chats.html?world=first-run&step=1'))}">${label}</a>`;
  const noModelLine = () => `No model yet · <a class="text-link" href="${esc(u(demo?.state === 'not started' ? 'welcome.html?at=model' : 'connect-model.html'))}">Connect a model</a>`;
  const modelHalf = () => demoRunning() && !world.accounts.some(a => a.provider !== demo.provider);
  const forgeHalf = () => demoRunning() && !world.forges.some(f => f.n !== demo.forge && f.state === 'connected');
  function demoBanner() {
    const m = modelHalf(), f = forgeHalf();
    if (!m && !f) return '';
    const words = m && f ? 'This is the playground: a scripted model and a playground forge.' : m ? 'This is the playground: a scripted model.' : 'This is the playground: a playground forge.';
    return `<aside class="first-run-banner" aria-label="Playground"><p>${fact('temper: the demo, as the forge’s side reads it, and which of its halves hold (client README, 4.8)', words)}</p><div class="terminal-proposal-actions">${m ? `<a href="${u('connect-model.html')}">Connect a model</a>` : ''}${f ? forgeBannerAct() : ''}</div></aside>`;
  }
  function demoStrip() {
    const setup = demo?.state === 'not started' && (!ownForge() || !usableModel());
    const words = !ownForge() && !usableModel() ? 'temper has no forge and no model yet.' : !ownForge() ? 'temper has no forge yet.' : 'temper has no model yet.';
    const settingUp = world.forges.some(f => f.mode !== 'fake' && ['setting up', 'connecting'].includes(f.state));
    // Continue setting up names its step (client README, 5.6).
    const started = world.forges.some(f => f.mode !== 'fake' && ['setting up', 'connecting', 'connected'].includes(f.state));
    const resume = !started ? 'forge' : !usableModel() ? 'model' : 'ready';
    const banner = setup ? `<aside class="first-run-banner" aria-label="Setup"><p>${words}</p><div class="terminal-proposal-actions">${settingUp ? forgeBannerAct() : ''}<a href="${esc(u(`welcome.html?at=${resume}`))}">Continue setting up</a>${!ownForge() ? tryDemo() : ''}</div></aside>` : demoBanner();
    return banner ? `<div class="demo-strip">${banner}</div>` : '';
  }
  function forgeBannerAct(target) {
    const f = world.forges.find(f => f.mode !== 'fake' && ['setting up', 'connecting'].includes(f.state));
    return f ? `<span class="banner-status">${fact('temper: the forge’s name and set-up state', esc(`${forgeName(f)} · ${f.state}${f.setup?.step ? `: ${f.setup.step}` : ''}`))}</span> <a href="${esc(u(target || `connect-forge.html?forge=${f.n}`))}">View</a>` : `<a href="${esc(u('connect-forge.html'))}">Connect a forge</a>`;
  }
  function demoContext() {
    if (!demoRunning()) return '';
    return `<div class="compact-context first-run-context"><span>${project()} · ${fact('temper: the demo’s repository, and its forge', `<strong>${esc(world.repos[demo.repo].name)}</strong> on Playground forge`)}</span><span>${fact('jig: your role on the deployment', 'Signed in as the deployment’s owner')}</span></div>`;
  }

  // The forges the configuration names (client forge.md, 5.4); the demo's
  // in the phrasebook's name (client README, 5.6).
  const forgeOf = n => world.forges.find(f => f.n === n);
  const forgeName = f => f.name || (f.mode === 'owned' ? 'Forgejo on this machine' : f.kind);

  const title = id => said(T(id).by, esc(T(id).title));
  const spent = id => fact(`jig: spent by ${id} itself`, esc(T(id).spend ?? ''));
  const model = id => fact(`jig: ${id}'s LLM choice, given by ${who(T(id).by)}, among the configuration's models`, esc(T(id).model || ''));
  const shownProject = () => params.get('project') || world.project;
  const inProject = t => (t.project || world.project) === shownProject();
  const projectTasks = () => Object.keys(world.tasks).filter(k => inProject(T(k)));
  // A first run's deployment has the project `temper init` seeds; with
  // no project until the playground's start or the first repository
  // makes one (draft first-run.md, 2.4), its place reads "No project yet".
  const seeded = () => world.seeded && shownProject() === world.seeded;
  const project = () => seeded()
    ? variant('projectless', conf('engine.projects, its first, unnamed, so “Project 1” (protocol/setup.md, 8.3)', esc(world.seeded)), 'No project yet')
    : fact('jig: the project', esc(params.get('project') || world.project || 'No project yet'));
  const noProject = () => !(params.get('project') || world.project);
  const PROJECT_NAV = ['board', 'changes', 'tasks', 'adopt'];
  const navShown = () => NAV.filter(e => !noProject() || !PROJECT_NAV.includes(e[0]));

  // ---- The step bar ---------------------------------------------------

  function mockbar() {
    const prev = W.steps[stepIndex - 1];
    const next = W.steps[stepIndex + 1];
    const upcoming = W.upcoming.find(([, id]) => !ids.includes(id));
    const nextTitle = next ? `Next: ${next.id} ${next.title}` : upcoming ? `Next in the case: ${upcoming[1]} ${upcoming[2]} (not drawn yet)` : 'The last step';
    const when = `${step.label} · ${DAY[date(step.hour * 60).getUTCDay()]} ${hm(step.hour * 60)}`;
    const legend = Object.entries(KINDS).map(([k, name]) => `<span class="legend-${k}">${esc(name)}</span>`).join('');
    return `<div class="mockbar" role="region" aria-label="The mockups’ controls">
      <a class="mockbar-mark" href="${u('index.html')}">${esc(W.name.toUpperCase())}</a>
      <div class="mockbar-step">
        <button type="button" data-go="${prev ? prev.id : ''}" ${prev ? '' : 'disabled'} title="${prev ? `Back to ${prev.id} ${esc(prev.title)}` : 'The first step'}" aria-label="Previous step">‹</button>
        <button type="button" data-pop="step" aria-expanded="false">${esc(step.label)} · ${esc(step.title)} ⌄</button>
        <button type="button" data-go="${next ? next.id : ''}" ${next ? '' : 'disabled'} title="${esc(nextTitle)}" aria-label="Next step">›</button>
      </div>
      <div class="mockbar-now"><b>${esc(when)}</b><span>${esc(W.acts[step.act])}</span></div>
      ${page === 'index' ? '' : REVIEWED[page] ? `<span class="mockbar-rules" title="${esc(`Under the README’s rules since ${REVIEWED[page]}: what the main drawing shows, the engine has. STATUS.md says what the client lacks.`)}">Under the rules</span>` : '<span class="mockbar-rules not" title="Not under the README’s rules yet: it may draw what the engine lacks (STATUS.md, section 2).">Not reviewed</span>'}
      <div class="mockbar-right">
        <div class="mockbar-legend" aria-hidden="true">${legend}</div>
        <button type="button" data-pop="steps" aria-expanded="false">Steps ⌄</button>
        <button type="button" data-pop="view" aria-expanded="false">View ⌄</button>
      </div>
    </div>`;
  }

  function stepPop() {
    return `<div class="mockbar-pop" data-popover="step" hidden style="left:120px">
      <h4>${esc(step.label)} · ${esc(step.title)}</h4>
      <p>${esc(step.summary)}</p>
      <h4>Where to look</h4>
      <ol>${step.look.map(([h, text]) => `<li><a href="${u(h)}">${esc(text)}</a></li>`).join('')}</ol>
    </div>`;
  }

  function stepsPop() {
    const drawn = W.steps.map(s => [s.act, s.id, s.title, s.label]);
    const later = W.upcoming.filter(([, id]) => !ids.includes(id));
    let act = 0;
    const items = [...drawn, ...later].map(([a, id, stepTitle, label]) => {
      const head = a !== act ? `<li class="act">${esc(W.acts[a])}</li>` : '';
      act = a;
      const isDrawn = ids.includes(id);
      const cls = id === stepId ? ' current' : isDrawn ? '' : ' later';
      const body = `<span>${esc(label || id)}</span><span>${esc(stepTitle)}${isDrawn ? '' : ' · not drawn yet'}</span>`;
      if (isDrawn && id !== stepId) {
        const p = new URLSearchParams(location.search);
        p.set('world', worldKey);
        p.set('step', id);
        return `${head}<li><a class="step-item" href="${location.pathname.split('/').pop()}?${p}">${body}</a></li>`;
      }
      return `${head}<li class="step-item${cls}">${body}</li>`;
    }).join('');
    return `<div class="mockbar-pop" data-popover="steps" hidden style="right:12px"><ul>${items}</ul></div>`;
  }

  function viewPop() {
    const file = location.pathname.split('/').pop() || 'index.html';
    const worlds = Object.entries(C.worlds).map(([key, w]) => {
      const p = new URLSearchParams(location.search);
      p.set('world', key);
      p.delete('step');
      if (!['chat', 'task'].includes(page)) p.delete('task');
      const target = ['chat', 'task'].includes(page) && key !== worldKey ? 'index.html' : file;
      return key === worldKey
        ? `<li class="step-item current"><span>●</span><span>${esc(w.name)}</span></li>`
        : `<li><a class="step-item" href="${target}?${p}"><span>○</span><span>${esc(w.name)}</span></a></li>`;
    }).join('');
    const toggle = (flag, label, what) => {
      const on = flag === 'plain' ? !document.body.classList.contains(FLAGS.plain) : document.body.classList.contains(FLAGS[flag]);
      return `<label class="view-toggle"><input type="checkbox" data-flag="${flag}" ${on ? 'checked' : ''}><span><b>${label}</b> ${what}</span></label>`;
    };
    const legend = Object.entries(KINDS).map(([k, name]) => `<li><span class="legend-${k}">${esc(name)}</span></li>`).join('');
    return `<div class="mockbar-pop view-pop" data-popover="view" hidden style="right:12px">
      <h4>World</h4><ul>${worlds}</ul>
      <h4>Show</h4>
      ${toggle('plain', 'Annotations', 'the reference case’s names, and the gaps temper lacks for what a page draws.')}
      ${toggle('sources', 'Sources', 'each value underlined by where it comes from; hover one for the detail. Unmarked text is the UI’s own words.')}
      ${toggle('future', 'Future', 'what the engine can’t supply yet, each marked with what it needs, in place of what the page shows today.')}
      <button type="button" class="view-hide" data-hide-bar><b>Hide this bar</b> and the marks: each page as it will look, from page to page. Esc, or the bottom-right corner, brings it back.</button>
      <h4>Sources</h4><ul class="view-legend">${legend}</ul>
    </div>`;
  }

  function bindCasebar() {
    document.querySelectorAll('[data-go]').forEach(b => b.addEventListener('click', () => {
      if (!b.dataset.go) return;
      const p = new URLSearchParams(location.search);
      p.set('world', worldKey);
      p.set('step', b.dataset.go);
      location.search = p.toString();
    }));
    document.querySelectorAll('[data-pop]').forEach(b => b.addEventListener('click', e => {
      e.stopPropagation();
      const pop = document.querySelector(`[data-popover="${b.dataset.pop}"]`);
      const open = pop.hidden;
      document.querySelectorAll('[data-popover]').forEach(p => { p.hidden = true; });
      document.querySelectorAll('[data-pop]').forEach(x => x.setAttribute('aria-expanded', 'false'));
      pop.hidden = !open;
      b.setAttribute('aria-expanded', String(open));
    }));
    document.addEventListener('click', e => {
      if (e.target.closest('.mockbar-pop')) return;
      document.querySelectorAll('[data-popover]').forEach(p => { p.hidden = true; });
    });
    const showBar = on => { store(BARLESS, on ? '0' : '1'); location.reload(); };
    document.querySelector('[data-hide-bar]')?.addEventListener('click', () => showBar(false));
    document.querySelector('[data-show-bar]')?.addEventListener('click', () => showBar(true));
    if (barless) document.addEventListener('keydown', e => { if (e.key === 'Escape') showBar(true); });
    document.querySelectorAll('[data-flag]').forEach(box => box.addEventListener('change', () => {
      const cls = FLAGS[box.dataset.flag];
      const on = box.dataset.flag === 'plain' ? !box.checked : box.checked;
      document.body.classList.toggle(cls, on);
      store(cls, on ? '1' : '0');
    }));
  }

  // The detail of a source, on hover, while sources show.
  function bindSources() {
    const tip = document.createElement('div');
    tip.className = 'src-tip';
    tip.hidden = true;
    document.body.append(tip);
    document.addEventListener('mouseover', e => {
      const s = document.body.classList.contains(FLAGS.sources) && e.target.closest('.src');
      if (!s) { tip.hidden = true; return; }
      const k = [...s.classList].find(c => c.startsWith('src-')).slice(4);
      tip.innerHTML = `<b class="legend-${k}">${esc(KINDS[k])}</b><span>${esc(s.dataset.src)}</span>`;
      tip.hidden = false;
      const r = s.getBoundingClientRect();
      const x = Math.min(Math.max(8, r.left), window.innerWidth - tip.offsetWidth - 8);
      const below = r.bottom + 6 + tip.offsetHeight < window.innerHeight;
      tip.style.left = `${x}px`;
      tip.style.top = `${below ? r.bottom + 6 : r.top - tip.offsetHeight - 6}px`;
    });
  }

  // ---- The product's chrome -------------------------------------------

  // In client README 5.3's order. Tasks and Workers are jig's pages
  // (pages.md, 6.3 and 7), not temper's yet; Adopt has no mockup (client
  // README, 4.4).
  const NAV = [
    ['inbox', 'inbox.html', 'Inbox'],
    ['chats', 'chats.html', 'Chats'],
    ['board', 'board.html', 'Board'],
    ['changes', 'changes.html', 'Changes'],
    ['tasks', 'tasks.html', 'Tasks'],
    ['adopt', '', 'Adopt a repository', 'No mockup: the adoption form is client README 4.4.'],
    ['forges', 'forges.html', 'Forges'],
    ['workers', 'workers.html', 'Workers'],
    ['accounts', 'accounts.html', 'Accounts'],
  ];
  function forgeNavMark() {
    const forges = world.forges.filter(f => f.mode !== 'fake');
    const needs = forges.some(f => ['set up failed', 'stopped'].includes(f.state));
    return needs || forges.some(f => ['setting up', 'connecting'].includes(f.state)) ? `<i class="nav-mark ${needs ? 'red' : 'amber'}" aria-label="${needs ? 'a forge needs you' : 'a forge is setting up'}"></i>` : '';
  }
  const navLink = ([key, h, label, later], active, n = '') => h
    ? `<a href="${u(h)}"${key === active ? ' class="active" aria-current="page"' : ''}>${label}${key === 'forges' ? forgeNavMark() : key === 'accounts' ? accountNavMark() : ''}${n}</a>`
    : `<a href="#" data-later="${esc(later)}">${label}</a>`;

  function topbar(active, { bare = false } = {}) {
    const links = navShown().map(e => navLink(e, active, e[0] === 'inbox' ? ` <span class="terminal-count">${world.inbox.length || ''}</span>` : '')).join('');
    return `<header class="terminal-topbar${bare ? ' terminal-topbar-bare' : ''}">
      <a class="terminal-brand" href="${u('chats.html')}" aria-label="temper chats"><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 1.5c1.5 5.6 2.9 7 8.5 8.5-5.6 1.5-7 2.9-8.5 8.5C10.5 12.9 9.1 11.5 3.5 10 9.1 8.5 10.5 7.1 12 1.5Z"/><circle cx="18.5" cy="18.5" r="2" fill="currentColor"/></svg><span>temper</span></a>
      ${bare ? '' : `<span class="terminal-divider" aria-hidden="true"></span>
      <span class="terminal-project">${project()} <span aria-hidden="true">⌄</span></span>
      <nav class="terminal-nav" aria-label="Main navigation">${links}</nav>`}
      <div class="terminal-top-right">${bare ? '' : `<span class="terminal-live"><i class="status-dot"></i> ${fact('jig-client: the link’s state', 'Live')}</span>`}<span class="terminal-user">${fact('jig: the party signed in', esc(world.person))}</span>${world.site ? `<a class="terminal-user" href="${esc(u('sign-in.html?state=signed-out'))}">Sign out</a>` : '<a class="terminal-user" href="#" data-later="Mockup: signing out is not drawn.">Sign out</a>'}</div>
      ${bare ? '' : `<details class="terminal-mobile-menu"><summary aria-label="Open navigation">☰</summary><nav aria-label="Mobile navigation">${navShown().map(e => navLink(e, active)).join('')}</nav></details>`}
    </header>`;
  }

  // The frame draws the demo's banner under the navigation, on every page
  // but the accounts and the forges pages (client README, 5.1), a chat's
  // and the demo's start among them; a page with a chatbar gives it as
  // its head, under the banner, and no crumbs.
  function listPage(active, crumbs, body, actions = '', head = null) {
    const trail = (crumbs || []).map((c, i) => i === crumbs.length - 1 ? `<strong>${c[0]}</strong>` : `<a href="${u(c[1])}">${c[0]}</a><span class="sep">/</span>`).join('');
    const strip = ['welcome', 'accounts', 'forges', 'connect-forge', 'connect-model'].includes(page) ? '' : demoStrip();
    document.body.classList.add('terminal-pages');
    return `${topbar(active)}${strip}${head ?? ''}<div class="app"><main class="main">
      ${crumbs ? `<header class="topbar"><div class="breadcrumbs">${trail}</div><div class="top-actions">${actions}<button class="icon-button" aria-label="More" title="Visual mockup"></button></div></header>` : ''}
      <div class="page">${body}</div>
    </main></div>`;
  }

  // ---- The tree -------------------------------------------------------

  function treeRows(roots, withRoots) {
    const rows = [];
    function walk(id, depth, lasts, ancestors) {
      const t = T(id);
      const children = kids(id).filter(k => inProject(T(k)));
      const glyph = depth === 0 ? '' : lasts.slice(0, -1).map(l => (l ? '   ' : '│  ')).join('') + (lasts[lasts.length - 1] ? '└─ ' : '├─ ');
      const running = t.worker && isLive(id);
      const where = t.kind === 'change'
        ? fact('temper: a change has no host of its own', '—')
        : fact(running ? `jig: the worker its run is on` : `jig: its charter's host kind; no run now`, esc(t.worker || t.hostKind));
      rows.push(`<div class="tree-row${isLive(id) ? '' : ' is-done'}${fresh(t)}" role="treeitem" style="--d:${depth}" data-id="${id}" data-anc="${ancestors.join(' ')}" data-live="${isLive(id)}" ${children.length ? 'aria-expanded="true"' : ''}>
        <span class="tree-name"><span class="tree-glyph">${glyph}</span><button type="button" class="tree-toggle" ${children.length ? `data-toggle="${id}" aria-label="Fold ${id}"` : 'disabled tabindex="-1" aria-hidden="true"'}>${children.length ? '▾' : ''}</button><a href="${href(id)}"><b>${id}</b>${title(id)}</a>${ref(t.ref)}</span>
        <span class="muted">${kind(id)}</span>
        <span class="muted">${model(id)}</span>
        <span class="${running ? '' : 'muted'}">${where}</span>
        <span class="tree-state ${stateClass(id)}">${phase(id, true)}</span>
        <span class="tree-spend">${spent(id)}</span>
      </div>`);
      children.forEach((k, i) => walk(k, depth + 1, [...lasts, i === children.length - 1], [...ancestors, id]));
    }
    if (withRoots) roots.forEach(r => walk(r, 0, [], []));
    else roots.forEach((r, i) => walk(r, 1, [i === roots.length - 1], []));
    return rows.join('');
  }

  function tree(roots, withRoots) {
    return `<div class="tree" role="tree">
      <div class="tree-row tree-head" aria-hidden="true"><span>Task</span><span>Kind</span><span>Model</span><span>On</span><span>State</span><span class="tree-spend">Spent</span></div>
      ${treeRows(roots, withRoots)}
    </div>`;
  }

  function bindTrees() {
    const folded = new Set();
    let liveOnly = false;
    function apply() {
      document.querySelectorAll('.tree-row[data-id]').forEach(row => {
        const anc = row.dataset.anc ? row.dataset.anc.split(' ') : [];
        const endedLeaf = row.dataset.live === 'false' && descendants(row.dataset.id).every(k => !isLive(k));
        row.hidden = anc.some(a => folded.has(a)) || (liveOnly && endedLeaf);
      });
    }
    document.querySelectorAll('[data-toggle]').forEach(b => b.addEventListener('click', () => {
      const id = b.dataset.toggle;
      const row = b.closest('.tree-row');
      if (folded.has(id)) { folded.delete(id); b.textContent = '▾'; row.setAttribute('aria-expanded', 'true'); }
      else { folded.add(id); b.textContent = '▸'; row.setAttribute('aria-expanded', 'false'); }
      apply();
    }));
    const set = on => {
      liveOnly = on;
      document.querySelectorAll('[data-live-only]').forEach(x => x.classList.toggle('active', on));
      document.querySelectorAll('[data-all]').forEach(x => x.classList.toggle('active', !on));
      apply();
    };
    document.querySelectorAll('[data-live-only]').forEach(b => b.addEventListener('click', () => set(!liveOnly)));
    document.querySelectorAll('[data-all]').forEach(b => b.addEventListener('click', () => set(false)));
  }

  // ---- Shared parts ---------------------------------------------------

  function slotsGlyph(used, total) { return `<span class="slots" role="img" aria-label="${used} of ${total} slots in use">${'<i class="on"></i>'.repeat(used)}${'<i></i>'.repeat(Math.max(0, total - used))}</span>`; }

  function repoHealth(r) {
    const red = r.ci.state !== 'passing' || (r.after && r.after.state !== 'passing');
    return `<span class="tag ${red ? 'red' : 'green'}">${fact(`temper: CI at ${r.name}'s tip, and what runs after a landing`, red ? `${r.branch} red` : `${r.branch} healthy`)}</span>`;
  }

  function findingRow(f) {
    const fix = T(f.fix);
    return `<div class="finding${fresh(f)}">
      <span class="finding-id">${esc(f.id)}</span>
      <div>
        <h3>${esc(f.title)} <span class="muted" style="font-weight:400">· ${esc(world.repos[f.repo].name)}</span></h3>
        <p><code>${esc(f.signature)}</code></p>
        <p>${esc(f.rate)} · matched at heads ${f.seen.heads} · at the tip ${f.seen.tip} · opened ${ago(f.opened)} from ${chip(f.from)}</p>
        <p>Goal ${chip(f.goal)} · fix ${chip(f.fix)} · ${phase(f.fix)}</p>
      </div>
      <span class="tag amber">${f.state === 'open' ? 'Open' : esc(f.state)}</span>
    </div>`;
  }

  function usageLine(label, pct, reset, pace) {
    const over = pace != null && pct > pace;
    return `<div class="usage-line"><div><span>${esc(label)}</span><span>${pct}%${reset ? ` · resets ${esc(reset)}` : ''}</span></div>
      <div class="usage-bar"><span class="${over ? 'over' : ''}" style="width:${pct}%"></span>${pace != null ? `<i style="left:${pace}%" title="Pace: where the week would be if spent evenly"></i>` : ''}</div></div>`;
  }

  function event(e, body) { return `<div class="event${fresh(e)}"><time>${fact('jig: when it was committed', clock(e.at))}</time><div>${body}</div></div>`; }

  function toast(message) {
    const el = document.createElement('div');
    el.className = 'demo-toast';
    el.textContent = message;
    document.body.append(el);
    setTimeout(() => el.remove(), 2600);
  }
  function bindLaterActs() {
    document.querySelectorAll('[data-later]').forEach(b => b.addEventListener('click', () => toast(b.dataset.later || 'Mockup: this act is not drawn.')));
  }

  // ---- A conversation, its cards and proposals ------------------------

  // A delegation's card, at the call that made it (client README, 5.7).
  function taskCard(k) {
    const t = T(k);
    const result = isLive(k) ? '' : t.kind === 'change' ? (t.merge ? `CI passed. Landed · merge commit ${t.merge.slice(0, 7)}` : '') : (t.result || '');
    const steps = t.kind === 'change'
      ? fact('temper: the steps, a stand-in, all ticked once landed (client README, 5.7)', `<div class="steps" role="list" aria-label="Steps">${['Branch pushed', 'Pull request opened', 'CI passed', 'Landed'].map(w => `<div class="step${t.stage === 'landed' ? ' done' : ''}" role="listitem">${w}</div>`).join('')}</div>`)
      : '';
    return `<article class="terminal-proposal task-card${fresh(t)}" aria-label="Task ${k.slice(1)}">
      <div class="terminal-proposal-heading"><span class="terminal-proposal-type">${fact(`jig: ${k}'s number, and its result contract`, `${k} · ${contract(k)}`)}</span><span>${fact(`jig: ${k}'s document, in temper's words`, cardState(k))}</span></div>
      <h2>${title(k)}</h2>
      ${result ? `<div class="task-card-result">${said(k, md(result), t.ended)}</div>` : ''}${steps}
      <a class="text-link" href="${href(k)}">Task ${k.slice(1)} ↗</a>
    </article>`;
  }

  // A proposal (jig's client objects.md, section 4), in the inbox or at
  // the call that made it.
  const number = n => String(+n.toFixed(1));
  function proposalCard(e, where) {
    const b = e.batch[0];
    const left = world.config.allotment - spentThisPeriod();
    const items = [
      ['Why', said(e.task, `“${esc(e.why)}”`, e.at)],
      ['Goal', `${e.own ? 'yours · ' : ''}${conf('the template and charter it names, in the configuration', esc(`${b.template} · ${b.charter}`))}`],
      ['Landing', fact('jig: the grant it asks for, in words', esc(`${b.landing} · priority ${b.priority}`))],
      ['Budget', fact('jig: what it needs, and who would fund it, from the proposal’s document', esc(`Funds ${b.budget} from your pool · ${number(left)} left, ${number(left - b.budget)} after`))],
    ];
    const waiting = w => fact('jig: since it was proposed', `${w} ${span(world.now - e.at)}`);
    if (where === 'chat') {
      return `<article class="terminal-proposal${fresh(e)}" aria-label="Proposal from ${e.task}">
        <div class="terminal-proposal-heading"><span class="terminal-proposal-type">PROPOSAL</span><span class="terminal-proposal-wait">${waiting('waiting')}</span></div>
        <h2>${said(e.task, esc(e.title), e.at)}</h2><p>${said(e.task, inline(e.text), e.at)}</p>
        <dl>${items.map(([k, v]) => `<div><dt>${k}</dt><dd>${v}</dd></div>`).join('')}</dl>
        <div class="terminal-proposal-actions"><button type="button" class="terminal-primary" data-later>Accept goal</button><button type="button" data-later>Reject with reason</button><a href="${u('inbox.html')}">Open in inbox ↗</a></div>
      </article>`;
    }
    return `<article class="card inbox-card${fresh(e)}"><div class="card-top"><div class="meta-row"><span class="tag accent">Proposal</span><span>${lineage(e.task)}</span></div><span class="muted tiny">${waiting('Waiting')}</span></div>
      <h3>${said(e.task, esc(e.title), e.at)}</h3><p>${said(e.task, inline(e.text), e.at)}</p>
      <div class="proposal-items">${items.map(([k, v]) => `<div class="proposal-item"><span class="label">${k}</span><span>${v}</span></div>`).join('')}</div>
      <div class="button-row"><button class="button primary" data-later>Accept goal</button><button class="button" data-later>Reject with reason</button><a class="button ghost" href="${href(e.task)}">Open ${esc(e.task)} ↗</a></div></article>`;
  }

  // A conversation's lines (jig's client objects.md, section 10), its
  // turns named as client README 5.1 names them: "You", "Temper", or the
  // party's name. A delegation shows as its tasks' cards, and a proposal
  // as its card, at the call that made them.
  function conversation(id, shown) {
    const t = T(id);
    return (world.transcripts[id] || []).map(x => {
      if (x.day) return `<div class="terminal-day">${esc(x.day)}${x.earlier ? ` · <button type="button" data-later="Mockup: the earlier turns are not drawn.">${x.earlier} earlier turns</button>` : ''}</div>`;
      if (x.who === 'you') return `<section class="terminal-turn terminal-turn-user${fresh(x)}"><div class="terminal-role">You <time>${clock(x.at)}</time></div><div class="terminal-turn-content"><span class="terminal-prompt">›</span>${said(world.person, md(x.text, '<br>'), x.at)}</div></section>`;
      if (x.who === 'agent' && x.live) return `<section class="terminal-turn provisional${fresh(x)}"><div class="terminal-role">Temper <span class="provisional-mark">${fact(`jig: ${id}'s provisional text, before its turn commits`, 'writing')}</span></div><div class="terminal-turn-content"><span class="terminal-prompt agent">✳</span>${said(id, md(x.text).replace(/<\/p><\/div>$/, '<span class="stream-caret" aria-hidden="true"></span></p></div>'), x.at)}</div></section>`;
      if (x.who === 'agent') return `<section class="terminal-turn${fresh(x)}"><div class="terminal-role">Temper <time>${clock(x.at)}</time></div><div class="terminal-turn-content"><span class="terminal-prompt agent">✳</span>${said(id, md(x.text), x.at)}</div></section>`;
      if (x.who === 'tools') {
        const made = (x.made || []).filter(k => T(k));
        const calls = x.calls.filter(([name]) => !(name === 'delegate' && made.length) && !(name === 'propose' && x.proposes));
        const n = calls.reduce((sum, [, c]) => sum + c, 0);
        const work = n ? `<details class="terminal-work${fresh(x)}"><summary>${fact(`jig: ${id}'s run, its tool calls folded`, plural(n, 'tool call'))} <span>${calls.map(([name, c]) => `${esc(name)}${c > 1 ? ` ×${c}` : ''}`).join(' · ')}</span></summary><div class="terminal-work-detail">${calls.map(([name, c, g]) => `<div>${fact('jig: a tool call, by its tool’s name', `${esc(name)}${c > 1 ? ` ×${c}` : ''}`)}${g ? ` ${gap(g, GAPS[g])}` : ''}</div>`).join('')}</div></details>` : '';
        made.forEach(k => shown.add(k));
        const proposal = x.proposes ? world.inbox.find(e => e.id === x.proposes) : null;
        return work + made.map(taskCard).join('') + (proposal ? proposalCard(proposal, 'chat') : '');
      }
      return '';
    }).join('');
  }
  const GAPS = { '6.3': 'plan usage', '6.7': 'fleet occupancy' };

  // ---- Pages ----------------------------------------------------------

  const pages = {};

  // Every state each world's steps name, page by page.
  function statesIndex() {
    const byPage = {};
    for (const [key, w] of Object.entries(C.worlds)) for (const s of w.steps) for (const [h, text] of s.look) {
      const [path, hash] = h.split('#');
      const [file, query] = path.split('?');
      const p = new URLSearchParams(query || '');
      if (!p.has('world')) p.set('world', key);
      if (!p.has('step')) p.set('step', s.id);
      const target = `${file}?${p}${hash ? `#${hash}` : ''}`;
      (byPage[file] = byPage[file] || []).push(`<li><a href="${esc(u(target))}">${esc(text)}</a> <span class="muted">· ${esc(w.name)}, step ${esc(s.label)}</span></li>`);
    }
    return Object.entries(byPage).sort(([a], [b]) => a.localeCompare(b)).map(([file, items]) => `<details class="mock-states"><summary><b>${esc(file)}</b> · ${plural(items.length, 'state', 'states')}${REVIEWED[file.replace('.html', '')] ? ' · under the rules' : ''}</summary><ul>${items.join('')}</ul></details>`).join('');
  }

  pages.index = () => {
    const look = step.look.map(([h, text]) => `<li><a href="${u(h)}">${esc(text)}</a></li>`).join('');
    const worlds = Object.entries(C.worlds).map(([key, w]) => key === worldKey
      ? `<li><b>${esc(w.name)}</b> · ${esc(w.about)}</li>`
      : `<li><a href="index.html?world=${key}">${esc(w.name)}</a> · ${esc(w.about)}</li>`).join('');
    const pagesList = [['welcome.html', 'Welcome'], ['inbox.html', 'Inbox'], ['chats.html', 'Chats'], ['board.html', 'Board'], ['changes.html', 'Changes'], ['tasks.html', 'Tasks'], ['forges.html', 'Forges'], ['connect-forge.html', 'Connect a forge'], ['workers.html', 'Workers'], ['accounts.html', 'Accounts'], ['connect-model.html', 'Connect a model']]
      .map(([h, label]) => `<a href="${u(h)}">${label}</a>`).join(' · ');
    return `<div class="mock-index">
      <h1>temper's client, in mockups</h1>
      <p>Each page is drawn from a world at a step, in the bar above: ‹ and › move the step, <b>Steps</b> lists them, and <b>View</b> switches the world and what is marked. The README says what the mockups are and their rules; STATUS.md, how far the client is from them. The bar says whether a page is under the rules yet.</p>
      <h2>Worlds</h2>
      <ul>${worlds}</ul>
      <h2>${esc(W.name)} · ${esc(step.label)} · ${esc(step.title)}</h2>
      <p>${esc(step.summary)}</p>
      <ol>${look}</ol>
      <h2>Every page's states</h2>
      ${statesIndex()}
      <h2>Pages</h2>
      <p>${pagesList}; a chat, a task and a change open from them. <a href="first-run.html">first-run.html</a> opens the first run at its first step: the link <code>temper box</code> prints. <a href="${u(`github.html?world=${GH_WORLD}`, '2')}">github.html</a> draws GitHub’s own pages, in the world First run, on GitHub. <a href="${u(`sign-in.html?world=${SIGN_IN_WORLD}`, '1')}">sign-in.html</a> draws the stock front page, signed out, and its sign-in’s endings, in the world Google sign-in.</p>
      <h2>Marks</h2>
      <p>Slate marks are annotations, not the product: the reference case's names (<i class="case-ref">B</i>, <i class="case-ref">f1</i>) and the gaps temper lacks for what a page draws (<i class="gap">gap 6.5</i>). <b>Sources</b>, in View, underlines each value by where it comes from: <span class="legend-engine">an engine fact</span>, <span class="legend-config">a name from the configuration</span>, <span class="legend-written">words as a person or an agent wrote them</span>, or <span class="legend-ops">an ops application</span>. Unmarked text is the UI's own words, and names no workflow. <b>Future</b>, in View, shows what the engine can’t supply yet, each piece marked with what it needs (<i class="future-mark">future · not designed</i>), in place of what the page shows today.</p>
    </div>`;
  };

  function lineage(id) {
    const chain = [];
    for (let k = id; k && T(k); k = T(k).parent) chain.unshift(k);
    return `${project()} › ${chain.map(k => fact(`${k}: its number and title`, `${k} ${esc(T(k).title)}`)).join(' › ')}`;
  }

  function entryCard(e) {
    if (e.kind === 'proposal') return proposalCard(e, 'inbox');
    if (e.kind === 'question') {
      return `<article class="card inbox-card${fresh(e)}"><div class="card-top"><div class="meta-row"><span class="tag amber">Question</span><span>${lineage(e.task)}</span></div><span class="muted tiny">${fact('jig: since it was asked', `Waiting ${span(world.now - e.at)}`)}</span></div>
        ${said(e.task, md(e.text), e.at)}
        <div class="button-row">${e.options.map(o => `<button class="button" data-later>${said(e.task, esc(o), e.at)}</button>`).join('')}<button class="button ghost" data-later>Answer in words</button><a class="button ghost" href="${href(e.task)}">Open ${esc(e.task)} ↗</a></div></article>`;
    }
    return '';
  }

  // Replies in the person's chats, and results of what they asked for.
  function updates() {
    const list = [];
    for (const [id, lines] of Object.entries(world.transcripts)) {
      const t = T(id);
      if (!t || requesterOf(t) !== world.person) continue;
      const replies = lines.filter(x => x.who === 'agent');
      if (!replies.length) continue;
      const last = replies[replies.length - 1];
      const unread = replies.filter(x => !x.read).length;
      list.push({ at: last.at, unread, html: `<a class="card update" href="${href(id)}"><span class="update-icon">✳</span><div><h3>${fact(`${id}: its number and title`, `${id} ${esc(t.title)}`)} replied</h3><p>${said(id, esc(firstLine(last.text)), last.at)}</p><p>${fact("jig: your read position in it", `${unread ? `${unread} new · ` : ''}${ago(last.at)}`)}</p></div></a>` });
    }
    for (const [id, t] of Object.entries(world.tasks)) {
      if (requesterOf(t) !== world.person || isChat(t) || isLive(id)) continue;
      list.push({ at: t.ended, unread: 1, html: `<a class="card update" href="${href(id)}"><span class="update-icon">✓</span><div><h3>${fact(`${id}: its number and title`, `${id} ${esc(t.title)}`)} is done</h3><p>${said(id, esc(firstLine(t.result || '')), t.ended)}</p></div></a>` });
    }
    return list.sort((a, b) => b.at - a.at);
  }

  pages.inbox = () => {
    const needs = world.inbox.map(entryCard).join('') || '<div class="empty-state">Nothing needs you.</div>';
    const ups = updates();
    const unread = ups.filter(x => x.unread).length;
    return listPage('inbox', [['Inbox']], `<div class="inbox-split"><div>
      <div class="section-heading" style="margin-top:0"><h2>Needs you <span class="subtle-count">${world.inbox.length}</span></h2><div class="filter-row"><a class="button" href="${u('chats.html')}">+ New chat</a><button class="filter active">All goals</button><button class="filter">Kind ⌄</button></div></div>
      ${needs}
    </div><aside><div class="section-heading" style="margin-top:0"><h2>Updates <span class="subtle-count">${unread ? `${unread} new` : ups.length}</span></h2></div><div class="updates-list">${ups.map(x => x.html).join('') || '<div class="empty-state">No updates.</div>'}</div></aside></div>`);
  };

  // No project yet: the composer and the list give way (client README,
  // 3.5); the frame keeps the setup strip.
  function noProjectChats() {
    const act = ownForge()
      ? '<a class="button primary" href="#" data-later="Mockup: the adoption form into a new project, named after the repository chosen (client README, 4.4).">Adopt a repository</a>'
      : `<a class="button primary" href="${esc(u('welcome.html?at=forge'))}">Set up temper</a>`;
    const playground = demo?.state === 'not started' ? tryDemo() : '';
    return `<section class="first-run-empty no-project"><h1>No project yet</h1><p>A project starts with its first repository.</p><div class="button-row">${act}${playground}</div></section>`;
  }

  pages.chats = () => {
    if (noProject()) return listPage('chats', [['Chats']], noProjectChats());
    const live = projectTasks().filter(k => isChat(T(k)) && requesterOf(T(k)) === world.person && isLive(k));
    const rows = live.map(id => {
      const t = T(id);
      const lines = world.transcripts[id] || [];
      const replies = lines.filter(x => x.who === 'agent');
      const last = replies[replies.length - 1];
      const activity = Math.max(...lines.filter(x => x.at != null).map(x => x.at), t.made);
      return `<a class="card chat-preview${fresh(t)}" href="${href(id)}"><div><div class="meta-row"><span class="tag ${stateClass(id) === 'running' ? 'green' : 'amber'}">${phase(id, true)}</span><span>${project()} · ${id}</span>${ref(t.ref)}</div><h3>${title(id)}</h3>${last ? `<p>${said(id, esc(firstLine(last.text)), last.at)}</p>` : ''}</div><div class="meta-row"><span>${spent(id)} spent${kids(id).length ? ` · ${fact('jig: its delegates, live or ended', liveKids(id).length ? plural(liveKids(id).length, 'delegate') : `${plural(kids(id).length, 'delegate')}, ended`)}` : ''}</span><span>${fact('jig: its last activity', ago(activity))}</span></div></a>`;
    });
    // A chat that ended itself, by its agent's finish, is still drawn;
    // one closed before these mockups' time is a summary.
    const finished = projectTasks().filter(k => isChat(T(k)) && requesterOf(T(k)) === world.person && !isLive(k)).map(id => {
      const t = T(id);
      const replies = (world.transcripts[id] || []).filter(x => x.who === 'agent');
      const last = replies[replies.length - 1];
      return `<a class="card chat-preview${fresh(t)}" href="${href(id)}"><div><div class="meta-row"><span class="tag">${fact(`jig: ${id}'s phase`, 'Ended')}</span><span>${project()} · ${id}</span></div><h3>${title(id)}</h3>${last ? `<p>${said(id, esc(firstLine(last.text)), last.at)}</p>` : ''}</div><div class="meta-row"><span>${spent(id)} spent</span><span>${fact('jig: when it ended', ago(t.ended))}</span></div></a>`;
    });
    const ended = world.ended.filter(inProject).filter(e => e.by === world.person).map(e => `<a class="card chat-preview" href="#"><div><div class="meta-row"><span class="tag">Closed</span><span>${project()} · ${e.task}</span></div><h3>${said(e.by, esc(e.title))}</h3><p>${fact('jig: its retention class, kept', 'Closed. Its transcript is kept.')}</p></div><div class="meta-row"><span>${fact('jig: what it spent', esc(e.spend))} spent</span><span>${fact('jig: when it ended', ago(e.ended))}</span></div></a>`);
    const all = [...rows, ...finished, ...ended];
    const line = newChatLine();
    const composer = `${line ? `<div class="composer-line">${line}</div>` : ''}<div class="terminal-entry-composer"><span class="entry-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Start a new chat"></textarea><span class="entry-hint">temper · Enter to start · Shift+Enter for newline</span><button class="send-button" aria-label="Start chat"${usableModel() ? ' data-later' : ' disabled'}>↑</button></div>`;
    const list = `<div class="section-heading"><h2>Your chats</h2><div class="filter-row"><button class="filter active">All</button><button class="filter">Live</button><button class="filter">Closed</button></div></div>${all.length ? `<div class="chat-list">${all.join('')}</div>` : '<div class="empty-state">No chats yet.</div>'}`;
    if (!modelHalf()) {
      const body = `${params.has('project') ? `<div class="compact-context"><span>${project()} · ${fact('temper: the project’s repositories, and their forges', Object.values(world.repos).filter(r => !r.retired && inProject(r)).map(r => `${esc(r.name)} on ${esc(forgeName(forgeOf(r.forge)))}`).join(' · '))}</span></div>` : ''}<div class="welcome"><div class="section-heading" style="margin-top:0"><h2>Start a chat</h2></div>${composer}</div>${list}${planVariants()}`;
      return listPage('chats', [['Chats']], seeded() ? variant('projectless', body, noProjectChats()) : body);
    }
    // The demo's start, in place of the page's heading (client README,
    // 5.6): a chat starts with its first message, so no task, no number.
    const chatbar = `<div class="terminal-chatbar"><div class="terminal-chat-identity"><strong class="terminal-title">New chat</strong><span class="terminal-phase" role="status">Ready</span></div><div class="terminal-chat-meta"><span>${fact('temper: the demo’s model half holds', 'Scripted model')}</span></div></div>`;
    const start = `<section class="first-run-empty" aria-labelledby="demo-start"><h1 id="demo-start">What shall we change?</h1>
      <p>Try a request in ${fact('temper: the demo’s repository', esc(world.repos[demo.repo].name))}. Follow the task from its branch to a landed change.</p>
      <ul class="suggestions" aria-label="Suggested requests">${demo.cues.map(suggestion).join('')}</ul>
      <p class="first-run-hint">The scripted model plays these requests from its script.</p></section>`;
    return listPage('chats', null, `${demoContext()}${start}<div class="welcome">${composer}</div>${list}`, '', chatbar);
  };

  // A plan's first use: a card in the conversation of the run that first
  // used it, from the document's fact (client README, 5.5).
  function firstUseCard(a) {
    return `<article class="terminal-proposal plan-card" aria-label="Your ${esc(a.plan)}"><div class="terminal-proposal-heading"><span class="terminal-proposal-type">${conf('the provider’s plan, in temper’s words', esc(a.plan.toUpperCase()))}</span><span>${fact('jig: the account’s first use, a fact of its document', 'First use')}</span></div><p>${fact('jig: a plan’s first use (jig’s accounts.md, section 5), in temper’s words', esc(`You’re using your ${a.plan}. Eligible requests from temper use your plan’s included usage. You can see it, and set a weekly cap for temper, in ${a.name}’s settings.`))}</p><div class="terminal-proposal-actions">${manageUsage()}</div></article>`;
  }
  function planVariants() {
    const a = ownAccounts().find(isPlan);
    if (worldKey !== 'first-run' || !a) return '';
    const limited = chatgptLike({ source: { state: 'limited' } });
    return `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: a chat as the plan’s first use shows it, and a chat whose plan is limited.</p>
      <h3>A plan’s first use, in the conversation of the run that first used it</h3>
      <div class="plan-sample"><section class="terminal-turn terminal-turn-user"><div class="terminal-role">You</div><div class="terminal-turn-content"><span class="terminal-prompt">›</span>${conf('a cue of the demo’s scripts, for the sample', `<div class="md"><p>${esc(demo.cues[1])}</p></div>`)}</div></section>${firstUseCard(a)}<p class="mock-note">The run’s turns follow. Nothing is kept to dismiss the card: it is part of the conversation.</p></div>
      <h3>A chat whose plan is limited ${ahead('limited chat', 'where a chat shows its account’s limit: client README 5.5 words the account’s state, not the chat’s')}</h3>
      <div class="plan-sample">${stateNotice(limited)}<div class="composer-line">${fact('jig: the account and model the chat’s next run takes', usingHtml(a, 'gpt-6-luna'))}</div></div>
    </section>`;
  }

  // A suggested request: the cue's words, copied in from the demo's
  // configuration. Where a later step draws what it starts, it leads there.
  function suggestion(cue) {
    const play = demo.plays[cue];
    const words = conf('a cue of the demo’s scripts, copied in (setup.md, 8.4)', esc(cue));
    if (play && ids.indexOf(play[0]) > stepIndex) return `<li><a class="suggestion" href="${u(`chat.html?task=${play[1]}`, play[0])}">${words}</a></li>`;
    const why = play ? 'Mockup: asking again is not drawn.' : 'Mockup: not drawn. Its script is the dark theme’s: one answer in words, then the chat ends.';
    return `<li><button type="button" class="suggestion" data-later="${esc(why)}">${words}</button></li>`;
  }

  pages.board = () => {
    const goals = projectTasks().filter(k => T(k).tracked && isLive(k)).sort((a, b) => T(a).priority - T(b).priority);
    const changes = projectTasks().filter(k => T(k).kind === 'change' && isLive(k));
    const ready = changes.filter(k => ['queued', 'first'].includes(T(k).stage)).length;
    const total = spentThisPeriod();
    const held = projectTasks().filter(k => T(k).phase === 'held').length;
    const rows = goals.map(g => {
      const t = T(g);
      const sub = descendants(g).filter(k => inProject(T(k)));
      const queued = sub.filter(k => T(k).kind === 'change' && ['queued', 'first'].includes(T(k).stage)).length;
      const running = sub.filter(k => T(k).kind !== 'change' && T(k).phase === 'running').length;
      const landed = sub.filter(k => T(k).kind === 'change' && T(k).stage === 'landed').length;
      const findings = world.findings.filter(f => f.goal === g && f.state === 'open').map(f => f.id);
      const pct = Math.round((t.spend / t.budget) * 100);
      const req = requesterOf(t);
      return `<article class="card board-row${fresh(t)}"><div class="rank">${fact('jig: its priority among the project’s goals', t.priority)}</div><div><div class="meta-row"><span class="tag ${stateClass(g) === 'running' ? 'green' : 'amber'}">${phase(g, true)}</span><span class="tiny">${g} · ${kind(g)}</span>${ref(t.ref)}</div><h3 style="margin-top:7px"><a href="${href(g)}">${title(g)}</a></h3><p>${fact('jig and temper: the tasks and changes in its tree', `${plural(landed, 'change')} landed · ${queued} queued · ${plural(running, 'delegate')} running`)}${findings.length ? ` · finding ${findings.join(', ')} open` : ''} · requested by ${req === world.person ? 'you' : chip(req)}</p></div><div><div class="progress"><span style="width:${pct}%"></span></div><p>${fact('jig: its budget, and what its tree spent', `${t.spend} of ${t.budget} spent`)} · ${fact('jig: its deadline', t.deadline ? `due ${esc(t.deadline)}` : 'no deadline')} · ${fact(`jig: ${t.worker ? 'the worker its run is on' : 'its host kind'}`, `on ${esc(t.worker || t.hostKind)}`)}</p></div><div class="right"><a class="text-link" href="${href(g)}">View goal ↗</a></div></article>`;
    }).join('') || '<div class="empty-state">No live goal.</div>';
    const repos = Object.values(world.repos).filter(r => !r.retired && inProject(r)).map(r => `<div class="kv"><span>${esc(r.branch)} in ${esc(r.name)}</span><strong>${fact(`temper: CI at ${r.name}'s tip`, r.ci.state === 'passing' ? '<span class="ok">CI passing at the tip</span>' : '<span class="bad">CI failing at the tip</span>')}${r.after ? ` · ${fact('temper: the check that runs after a landing, by the forge’s name', `${esc(r.after.name)} ${esc(r.after.state)}`)}` : ''} · ${fact('temper: its landing queue', plural(r.queue.length, 'change'))} queued</strong></div>`).join('');
    const open = world.findings.filter(f => inProject(world.repos[f.repo]) && f.state === 'open');
    const workers = world.workers.map(w => `${w.name} ${liveRuns(w.name).length}/${w.slots}`).join(' · ');
    return listPage('board', [[project(), 'board.html'], ['Board']], `
      <div class="stats-row"><div class="card stat-card"><span class="muted tiny">Live goals</span><strong class="value">${fact('jig: the project’s live goals', goals.length)}</strong><small>${fact('jig: held tasks in their trees', held ? `${held} held` : 'none held')}</small></div><div class="card stat-card"><span class="muted tiny">Changes in flight</span><strong class="value">${fact('temper: live changes', changes.length)}</strong><small>${fact('temper: changes in landing queues', `${ready} ready, in queues`)}</small></div><div class="card stat-card"><span class="muted tiny">Needs you</span><strong class="value">${fact('jig: your inbox’s entries', world.inbox.length)}</strong><small><a class="text-link" href="${u('inbox.html')}">Open inbox ↗</a></small></div><div class="card stat-card"><span class="muted tiny">Spent this ${conf('the configuration’s period', esc(world.config.period))}</span><strong class="value">${fact('jig: spent under your authority this period', total.toFixed(1))}</strong><small>${conf('your allotment per period, in the configuration', `of ${world.config.allotment} allotted`)}</small></div></div>
      <div class="section-heading"><h2>Priority order</h2><div class="filter-row"><button class="button primary" data-later>+ Set a goal</button><button class="filter active">Live</button><button class="filter">Waiting</button><button class="filter">Recently ended</button></div></div>
      <div class="stack">${rows}</div>
      <div class="section-heading"><h2>Project health</h2><a class="text-link" href="${u('changes.html')}">See changes and queues ↗</a></div>
      <div class="card card-pad">${repos}<div class="kv"><span>Open findings</span><strong>${open.map(f => `${esc(f.id)} · ${esc(f.title)}`).join(', ') || 'none'} ${gap('6.5', 'an open finding at the landing tip')}</strong></div><div class="kv"><span>Workers</span><strong><a class="text-link" href="${u('workers.html')}">${fact('jig: workers’ slots and live runs', `${esc(workers)} slots`)} ↗</a></strong></div></div>`);
  };

  pages.tasks = () => {
    const roots = projectTasks().filter(k => requesterOf(T(k)) === world.person && isLive(k)).sort((a, b) => T(a).made - T(b).made);
    const all = roots.flatMap(r => [r, ...descendants(r).filter(k => inProject(T(k)))]);
    const running = all.filter(k => T(k).phase === 'running').length;
    const waiting = all.filter(k => T(k).phase === 'idle').length;
    const queued = all.filter(k => ['queued', 'first'].includes(T(k).stage)).length;
    return listPage('tasks', [[project(), 'board.html'], ['Tasks']], `
      <div class="section-heading" style="margin-top:0"><div><h2>Every task <span class="subtle-count">${all.length}</span></h2><p class="muted tiny" style="margin:5px 0 0">What you requested, and what each delegated: ${fact('jig: the tasks’ phases', `${running} running · ${waiting} waiting · ${queued} in queues`)}.</p></div><div class="filter-row"><button class="filter active" data-all>All</button><button class="filter" data-live-only>Live only</button></div></div>
      <section class="card card-pad">${tree(roots, true)}</section>`);
  };

  pages.task = () => {
    const id = params.get('task') || Object.keys(world.tasks).find(k => T(k).tracked);
    const t = T(id);
    if (!t) return listPage('tasks', [[project(), 'board.html'], ['Tasks', 'tasks.html'], [esc(id)]], `<div class="empty-state">${esc(id)} does not exist at this step.</div>`);
    if (isChat(t)) { location.replace(href(id)); return ''; }
    if (t.tracked) return goalPage(id, t);
    if (t.kind === 'change') return changePage(id, t);
    return agentPage(id, t);
  };

  function goalPage(id, t) {
    const all = descendants(id);
    const pct = Math.round((t.spend / t.budget) * 100);
    const holders = all.filter(k => T(k).pool && isLive(k));
    const findings = world.findings.filter(f => f.goal === id);
    const notes = world.notes.filter(n => n.scope === 'goal' && n.goal === id);
    const worker = world.workers.find(w => w.name === t.worker);
    const req = requesterOf(t);
    const reqWords = req === world.person ? 'you' : chip(req);
    const activity = (world.activity[id] || []).map(e => event(e, `<strong>${fact(`jig: ${e.task}, ${e.what}`, `${e.task} ${esc(T(e.task) ? T(e.task).title : '')} · ${esc(e.what)}`)}</strong>${e.gap ? ` ${gap(e.gap, 'an open finding')}` : ''}<p>${fact('jig and temper: what the commit records', esc(e.text))}</p>`)).join('');
    const history = (world.history[id] || []).map(e => event(e, `<strong>${fact('jig: the task’s history, by whom', `${esc(who(e.by))} ${esc(e.what)}`)}</strong>${e.reason ? `<p>${said(e.by, `“${esc(e.reason)}”`, e.at)}</p>` : ''}`)).join('');
    const changes = all.filter(k => T(k).kind === 'change' && isLive(k));
    const run = t.run ? fact(`jig: run ${t.run.n} of ${id}`, `Run ${t.run.n}${t.run.waiting ? ' · waits, holding its slot' : ''}`) : fact('jig: no live run', 'No run now');
    return listPage('tasks', [[project(), 'board.html'], ['Board', 'board.html'], [id]], `
      <div class="page-header"><div class="task-heading">
        <div class="meta-row"><span class="tag accent">${fact('jig: tracked, a goal, with its priority', `Goal · priority ${t.priority}`)}</span><span class="task-chip"><b>${id}</b> ${kind(id)}</span>${ref(t.ref)}<span>${project()}</span></div>
        <h1>${title(id)}</h1>
        <div class="meta-row" style="margin-top:12px"><span class="tag ${stateClass(id) === 'running' ? 'green' : 'amber'}"><i class="pill-dot"></i> ${phase(id)}</span><span>Requested by ${reqWords} · ${fact('jig: when it was made', `made ${ago(t.made)}`)}</span></div>
        <div class="facts-line"><span>Model <b>${model(id)}</b></span><span>Account <b>${fact('jig: the account its runs use, kept from run to run', esc(t.account))}</b></span><span>Host kind <b>${conf('its charter’s host kind, in the configuration', esc(t.hostKind))}</b></span><span>${run}</span></div>
      </div><div class="task-toolbar"><button class="button" data-later>Amend</button><button class="button" data-later>Stop</button><button class="button danger" data-later>Cancel</button></div></div>
      <div class="tabbar"><a class="active" href="#overview">Overview</a><a href="#plan">Plan</a><a href="#findings">Findings</a><a href="#notes">Notes</a><a href="#activity">Activity</a><a href="#history">History</a></div>
      <div class="two-col"><div class="stack">
        <section id="overview" class="card card-pad"><div class="card-top"><h2>What was asked</h2><span class="tag outline">${fact('jig: its result contract', 'Result · report')}</span></div>
          <div style="margin:12px 0 0;font-size:12px;line-height:1.7">${said(t.by, md(t.brief), t.made)}</div>
          <div class="rule"></div>
          <div class="kv"><span>Input</span><strong>${fact('jig: an input its batch named', esc(t.input))}</strong></div>
          <div class="kv"><span>Writes</span><strong>${fact('jig: the resources it writes, held alone', esc(t.writes))}</strong></div>
          <div class="kv"><span>Wakes on</span><strong>${fact('jig: its wake policy, by kind of message', esc(t.wakesOn.join(', ')))}</strong></div>
        </section>
        <section id="plan" class="card card-pad"><div class="card-top"><div><h2>Plan <span class="subtle-count">${plural(all.length, 'task')}</span></h2><p class="muted tiny" style="margin:5px 0 0">Its delegates: ${fact('jig: its delegates’ phases', `${all.filter(k => T(k).phase === 'running').length} running, ${all.filter(k => ['queued', 'first'].includes(T(k).stage)).length} in queues, ${all.filter(k => !isLive(k)).length} ended`)}.</p></div><div class="filter-row"><button class="filter active" data-all>All</button><button class="filter" data-live-only>Live only</button></div></div>
          <div style="margin-top:10px">${tree(kids(id), false)}</div>
          ${t.pool ? `<div class="rule"></div><div class="meta-row"><span>Pool <b style="color:var(--term-ink)">${fact(`jig: a pool ${id} holds, its name as ${who(t.by)} gave it`, esc(t.pool.name))}</b></span>${slotsGlyph(holders.length, t.pool.size)}<span>${fact('jig: the pool’s holders', `${holders.length} of ${t.pool.size} held: ${holders.join(', ') || 'none'}`)}</span><span>· a new delegate waits for a slot</span></div>` : ''}
        </section>
        <section id="findings" class="card card-pad"><div class="card-top"><h2>Findings on this goal <span class="subtle-count">${findings.length}</span></h2>${gap('6.5', 'an open finding at the landing tip')}</div>${findings.map(findingRow).join('') || '<p class="muted">None.</p>'}</section>
        <section id="notes" class="card card-pad"><div class="card-top"><div><h2>Notes <span class="subtle-count">${notes.length}</span></h2><p class="muted tiny" style="margin:5px 0 0">In its scope: each brief in its tree carries their index.</p></div></div>${notes.length ? `<ul class="note-list">${notes.map(n => `<li>${said(n.by, `<b>${esc(n.name)}</b> · ${inline(n.text)}`, n.at)}</li>`).join('')}</ul>` : '<p class="muted">None.</p>'}</section>
        <section id="activity" class="card card-pad"><div class="card-top"><h2>Latest activity</h2><span class="tag green">Live</span></div><div class="timeline">${activity || '<p class="muted">None.</p>'}</div></section>
        <section id="history" class="card card-pad"><div class="card-top"><h2>History</h2><button class="filter">Whole subtree ⌄</button></div><div class="timeline">${history}</div></section>
      </div>
      <aside class="stack">
        <section class="card side-card"><h3>Budget & authority</h3><div class="progress"><span style="width:${pct}%"></span></div><p class="mini-stat">${fact('jig: its budget, and what its tree spent', `${t.spend} spent · ${(t.budget - t.spend).toFixed(1)} left of ${t.budget}`)}</p><div class="rule"></div><div class="kv"><span>Funded by</span><strong>${req === world.person ? fact('jig: who funded its budget', esc(world.person)) : chip(req)}</strong></div><div class="kv"><span>Deadline</span><strong>${fact('jig: its deadline', esc(t.deadline || 'none'))}</strong></div></section>
        <section class="card side-card"><h3>Placement</h3><div class="kv"><span>Host kind</span><strong>${conf('its charter’s host kind, in the configuration', esc(t.hostKind))}</strong></div><div class="kv"><span>Its run</span><strong>${worker ? `<a class="text-link" href="${u('workers.html')}">${fact('jig: the worker its run is on', `${esc(worker.name)} · ${liveRuns(worker.name).length} of ${worker.slots} slots`)} ↗</a>` : fact('jig: no live run', 'none now')}</strong></div>${t.pool ? `<div class="kv"><span>Pool</span><strong>${fact('jig: the pool’s holders', `${holders.length} of ${t.pool.size} ${esc(t.pool.name)}`)}</strong></div>` : ''}</section>
        <section class="card side-card"><h3>Its changes</h3><div class="aside-list">${changes.map(k => `<a href="${href(k)}"><span class="task-chip"><b>${k}</b> ${esc(T(k).title)}</span>${ref(T(k).ref)}<small>${phase(k, true)} · ${fact('temper: its pull request', T(k).pr ? `PR #${T(k).pr}` : 'no pull request yet')}</small></a>`).join('') || '<p class="muted tiny" style="margin:0">None live.</p>'}</div></section>
      </aside></div>`);
  }

  // A repository's page on its forge, or a pull request's.
  function forgeLink(repo, pr) {
    const f = forgeOf(repo.forge);
    return `${f.web}/${repo.name}${pr ? `/${f.kind === 'GitHub' ? 'pull' : 'pulls'}/${pr}` : ''}`;
  }

  function changePage(id, t) {
    const repo = world.repos[t.repo];
    const pos = repo.queue.indexOf(id);
    const order = ['producing', 'opening', 'checking', 'gating', 'queued', 'landed'];
    const at = { first: 4, landing: 5 }[t.stage] ?? order.indexOf(t.stage);
    const names = order.map((s, i) => (i === 4 && pos >= 0 ? `Queued · ${ordinal(pos + 1)}` : cap(s)));
    const steps = names.map((n, i) => `<div class="step ${i < at || t.stage === 'landed' ? 'done' : i === at ? 'current' : ''}">${esc(n)}</div>`).join('');
    const fixes = t.fixes ? world.findings.find(f => f.id === t.fixes) : null;
    const goal = [id, ...[]].reduce(k => { let g = T(k).parent; while (g && !T(g).tracked) g = T(g).parent; return g; }, id);
    const ci = at > 2 ? 'Passed' : at === 2 ? 'Running' : 'Not yet';
    const gates = repo.gates || [];
    return listPage('changes', [[project(), 'board.html'], ['Changes', 'changes.html'], [id]], `
      <div class="page-header"><div class="task-heading"><div class="meta-row"><span class="tag accent">${fact('temper: a change', `Change · ${id}`)}</span>${ref(t.ref)}<span>Goal ${chip(goal)}</span></div><h1>${title(id)}</h1><p class="intro">${fact('temper: its pull request, branch and landing branch', `${t.pr ? `PR #${t.pr}` : 'No pull request yet'} · ${esc(repo.name)} · temper/${id.slice(1)}/${esc(t.title.toLowerCase().replace(/[^a-z0-9]+/g, '-'))} → ${esc(repo.branch)}`)}</p></div>
      <div class="task-toolbar"><a class="button" href="${esc(forgeLink(repo, t.pr))}" target="_blank" rel="noreferrer">View on forge ↗</a></div></div>
      <section class="card card-pad">${fact(`temper: ${id}'s stage (forge.md, section 8)`, `<div class="steps">${steps}</div>`)}
        ${fixes ? `<div class="notice"><span>◇</span><div><strong>Fixes finding ${esc(fixes.id)}.</strong> ${esc(fixes.title)}: ${esc(fixes.rate)}. ${gap('6.5', 'an open finding at the landing tip')}</div></div>` : ''}
        <div class="section-heading"><h3>Its tasks</h3><span class="muted tiny">made by the change procedure: producing, repairs, reviews, resolving</span></div>
        ${kids(id).map(k => `<div class="gate"><strong><a href="${href(k)}">${k} ${title(k)}</a> <span class="muted" style="font-weight:400">${kind(k)}</span></strong><span>${phase(k, true)}</span></div>`).join('') || '<p class="muted">None yet.</p>'}
        <div class="section-heading"><h3>Gates at its head</h3></div>
        <div class="gate"><strong>CI at its head</strong><span>${fact('temper: CI on the change’s exact head', ci)}</span></div>
        ${gates.map(g => `<div class="gate"><strong>${conf(`a gate ${repo.name}'s landing rules add`, esc(g))}</strong><span>Not yet</span></div>`).join('') || `<div class="gate"><strong class="muted">No other gate</strong><span>${conf(`${repo.name}'s landing rules, in the project’s configuration`, 'its landing rules add none')}</span></div>`}
      </section>`);
  }

  function agentPage(id, t) {
    document.body.classList.add('terminal-variant', 'agent-task-page');
    const parent = T(t.parent);
    const live = t.phase === 'running';
    const body = !isLive(id)
      ? `<div class="agent-run-heading"><strong>Run 1 · ended</strong></div><div class="agent-result"><strong>RESULT</strong>${said(id, md(t.result), t.ended)}</div>`
      : `<div class="agent-run-heading"><strong>${t.run ? fact(`jig: run ${t.run.n} of ${id}`, `Run ${t.run.n} · ${live ? 'live now' : 'waiting'}`) : 'No run now'}</strong></div>
         <div class="agent-run-meta">${t.run ? fact('jig: when its run started, and on which worker', `Started ${clock(t.run.since)}${t.worker ? ` on ${esc(t.worker)}` : ''}`) : conf('its charter’s host kind', `Host kind ${esc(t.hostKind)}`)} · ${spent(id)} spent so far</div>
         ${world.transcripts[id] ? conversation(id, new Set()) : `<div class="agent-event"><strong>${phase(id)}</strong> <span class="mock-note">Its transcript is not drawn in these mockups.</span></div>`}`;
    return `<div class="terminal-shell">${topbar('tasks')}${demoStrip()}
      <div class="agent-lineage"><a href="${href(t.parent)}">← ${t.parent} ${esc(parent.title)}</a><span>/</span><span class="agent-lineage-current">${id} ${esc(t.title)}</span></div>
      <div class="terminal-chatbar agent-taskbar"><div class="terminal-chat-identity"><h1 class="chatbar-heading"><span class="terminal-number">${id}</span><span class="visually-hidden"> · </span><strong class="terminal-title">${title(id)}</strong></h1>${ref(t.ref)}<span class="terminal-phase${stateClass(id) === 'waiting' ? ' waiting' : stateClass(id) === 'done' ? ' closed' : ''}"><i class="pill-dot"></i> ${phase(id, true)}</span></div>
        <div class="terminal-chat-meta"><span class="agent-charter">${kind(id)} · ${model(id)}</span><span class="agent-spend">${spent(id)} spent</span>${isLive(id) ? '<button type="button" data-later>Amend</button><button type="button" data-later>Stop</button>' : ''}</div></div>
      <main class="terminal-thread" aria-label="Agent task"><div class="terminal-thread-inner">
        <div class="agent-brief"><span class="agent-kicker">TASK BRIEF</span><div class="agent-brief-text">${said(t.briefBy || t.by, md(t.brief), t.made)}</div><div class="agent-brief-meta">Asked by ${esc(t.parent)}${t.briefBy && t.briefBy !== t.parent ? `, from ${esc(t.briefBy)}’s spec` : ''} · ${fact('jig: the account its runs use', esc(t.account))} · ${model(id)}${t.pool ? ` · ${fact(`jig: a slot of ${t.parent}'s pool`, `holds a slot of ${t.parent}’s pool`)}` : ''}</div></div>
        ${body}
      </div></main>
      ${isLive(id) ? `<footer class="terminal-composer-area agent-steer-area"><div class="agent-steer-note">Steer this task · words reach its live run at the next turn.</div><div class="terminal-composer"><span class="terminal-input-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Write to agent task" placeholder="Write to ${esc(id)}…"></textarea><span class="terminal-input-hint">Enter to send · Shift+Enter for newline</span><button type="button" class="terminal-send" aria-label="Send" data-later>↑</button></div></footer>` : ''}
    </div>`;
  }

  pages.chat = () => {
    const id = params.get('task') || Object.keys(world.tasks).find(k => isChat(T(k)));
    const t = T(id);
    if (!t) return listPage('chats', [['Chats', 'chats.html'], [esc(id || 'A chat')]], `<div class="empty-state">${id ? `${esc(id)} does not exist at this step.` : 'No chat at this step.'}</div>`);
    document.body.classList.add('terminal-variant');
    const shown = new Set();
    const lines = conversation(id, shown);
    // A live delegate whose call the window does not hold shows after the
    // conversation (client README, 5.7).
    const after = liveKids(id).filter(k => !shown.has(k)).map(taskCard).join('');
    const notes = world.notes.filter(n => n.by === id);
    const scopes = [...new Set(notes.map(n => n.scope))];
    const live = isLive(id);
    // A chat its agent finished has ended, with its report; it takes no
    // more words, so its page has no composer.
    const end = live ? '' : `<div class="terminal-day terminal-ended">${fact(`jig: ${id}'s run ended, and its result`, `Ended ${clock(t.ended)}`)}${t.endedWhy ? ` · ${fact('temper: why the chat ended', esc(t.endedWhy))}` : t.result ? ` · ${said(id, `its report, “${esc(t.result)}”`, t.ended)}` : ''}</div>`;
    return `<div class="terminal-shell">${topbar('chats')}${demoStrip()}
      <div class="terminal-chatbar"><div class="terminal-chat-identity"><h1 class="chatbar-heading"><span class="terminal-number">${id}</span><span class="visually-hidden"> · </span><strong class="terminal-title">${title(id)}</strong></h1>${ref(t.ref)}<span class="terminal-phase ${live ? 'waiting' : 'closed'}"><i class="pill-dot"></i> ${phase(id)}</span></div>
        <div class="terminal-chat-meta"><span>${kind(id)} · ${model(id)} · ${conf('its charter’s host kind', esc(t.hostKind))}</span><span>${fact('jig: what it spent, and its budget', `${t.spend} spent · ${t.budget ? `of ${t.budget}` : 'as allowed'}`)}</span>${notes.length ? '<button type="button" data-notes aria-expanded="false">Notes</button>' : ''}${kids(id).length ? `<a href="${u('tasks.html')}" class="chatbar-link">${fact('jig: its delegates, live or ended', liveKids(id).length ? plural(liveKids(id).length, 'delegate') : `${plural(kids(id).length, 'delegate')}, ended`)}</a>` : ''}${live ? '<button type="button" data-later>Stop</button><button type="button" data-later>Close</button>' : ''}</div></div>
      <main class="terminal-thread" aria-label="Conversation"><div class="terminal-thread-inner">${demoContext()}${lines}${after}${end}</div></main>
      ${live ? `<footer class="terminal-composer-area"><div class="composer-line">${usingLine(id)}</div><div class="terminal-composer"><span class="terminal-input-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Message ${id}" placeholder="Message ${esc(id)}…"></textarea><span class="terminal-input-hint">Enter to send · Shift+Enter for newline</span><button type="button" class="terminal-send" aria-label="Send message"${usableModel() ? ' data-later' : ' disabled'}>↑</button></div></footer>` : ''}
    </div>
    ${notes.length ? `<aside class="drawer" id="notes" hidden aria-label="Notes ${id} wrote">
      <div class="drawer-head"><h2>${id} · the notes it wrote</h2><button type="button" data-notes-close>Close ✕</button></div>
      <p class="muted" style="margin:0">Notes are hints: where a note and the facts disagree, the facts are right.</p>
      ${scopes.map(s => `<h3>${fact('jig: the note’s scope', `${esc(s)} scope${s === 'project' ? ` · ${esc(world.project)}` : ''}`)}</h3><ul>${notes.filter(n => n.scope === s).map(n => `<li>${said(n.by, `<b>${esc(n.name)}</b>${md(n.text)}`, n.at)}</li>`).join('')}</ul>`).join('')}
      <p class="drawer-foot">Each an entry: a one-line description, a body, a scope and its writer.</p>
    </aside>` : ''}`;
  };

  pages.changes = () => {
    const projectRepos = Object.entries(world.repos).filter(([, r]) => (r.project || world.project) === (params.get('project') || world.project));
    const repos = projectRepos.filter(([, r]) => !r.retired);
    const cards = repos.map(([key, r]) => `<section class="card card-pad repo-card"><div class="card-top"><div><h2>${fact('temper: an adopted repository, and its landing branch', `${esc(r.branch)} · ${esc(r.name)}`)}</h2><p class="muted tiny" style="margin:4px 0 0">${fact('temper: its landing queue', plural(r.queue.length, 'change'))} queued</p></div>${repoHealth(r)}</div>
      <div class="repo-lines"><div class="kv"><span>Tip</span><strong>${fact('temper: the branch’s tip', esc(r.tip))}</strong></div><div class="kv"><span>CI at the tip</span><strong class="${r.ci.state === 'passing' ? 'ok' : 'bad'}">${fact('temper: CI at the tip', `${r.ci.state === 'passing' ? 'Passing' : 'Failing'} · ${clock(r.ci.at)}`)}</strong></div>${r.after ? `<div class="kv"><span>After landing</span><strong class="${r.after.state === 'passing' ? 'ok' : 'bad'}">${fact('temper: the check that runs after a landing, by the forge’s name', `${esc(r.after.name)} ${esc(r.after.state)} at ${esc(r.after.head)} · ${clock(r.after.at)}`)}</strong></div>` : ''}</div></section>`).join('');
    const changeRow = (k, i, tag) => {
      const t = T(k);
      let g = t.parent;
      while (g && !T(g).tracked) g = T(g).parent;
      return `<div class="queue-row${fresh(t)}"><span class="queue-no">${i == null ? '·' : i + 1}</span><div><h3><a href="${href(k)}">${k} ${title(k)}</a>${ref(t.ref)}</h3><p>${g ? `Goal ${fact('jig: its goal, and the goal’s priority', `${g} ${esc(T(g).title)} · priority ${T(g).priority}`)}` : ''}${t.ready != null ? ` · ${fact('temper: since it was ready', `ready ${span(world.now - t.ready)}`)}` : ''}${t.fixes ? ` · fixes ${esc(t.fixes)}` : ''}</p></div><span class="tag ${tag}">${phase(k, true)}</span></div>`;
    };
    const queues = repos.map(([key, r]) => {
      const rows = r.queue.map((k, i) => changeRow(k, i, i === 0 ? 'accent' : '')).join('') || '<div class="empty-state">Nothing queued.</div>';
      return `<section class="card queue-col"><div class="card-pad" style="padding-bottom:8px"><div class="card-top"><div><h2>${esc(key)} · landing queue</h2><p class="muted tiny" style="margin:5px 0 0">By goal priority, then by when ready.</p></div><span class="tag ${r.queue.length ? 'green' : ''}">${fact('temper: its landing queue', `${r.queue.length} ready`)}</span></div></div>${rows}</section>`;
    }).join('');
    const notReady = projectTasks().filter(k => T(k).kind === 'change' && ['producing', 'opening', 'checking', 'gating'].includes(T(k).stage));
    const landings = world.landings.filter(l => projectRepos.some(([key]) => key === l.repo)).map(l => `<a href="${href(l.task)}">${fact('temper: a landing', `${l.task} ${esc(T(l.task) ? T(l.task).title : l.title)}`)}<small>${fact('temper: where and when it landed', `${esc(world.repos[l.repo].name)}${world.repos[l.repo].retired ? ' · the playground’s, ended' : ''} · ${clock(l.at)}${T(l.task) && T(l.task).parent ? ` · goal ${T(l.task).parent}` : ''}`)}</small></a>`).join('');
    const open = world.findings.filter(f => f.state === 'open' && repos.some(([key]) => key === f.repo));
    return listPage('changes', [[project(), 'board.html'], ['Changes']], `
      <div class="compact-context"><span>${fact('temper: the forges and the repositories adopted on each', world.forges.filter(f => repos.some(([, r]) => r.forge === f.n)).map(f => `Forge · ${esc(forgeName(f))} · ${repos.filter(([, r]) => r.forge === f.n).map(([, r]) => esc(r.name)).join(', ')}`).join(' · '))}</span></div>
      ${repos.length ? `<div class="repo-grid${repos.length === 1 ? ' one' : ''}">${cards}</div>
      <div class="section-heading"><h2>Landing queues</h2></div>
      <div class="repo-grid${repos.length === 1 ? ' one' : ''}">${queues}</div>` : `<div class="empty-state">No repository in ${project()}.</div>`}
      <div class="section-heading"><h2>Not ready yet</h2><span class="muted tiny">${plural(notReady.length, 'change')}</span></div>
      ${notReady.length ? `<section class="card queue-col">${notReady.map(k => changeRow(k, null, '')).join('')}</section>` : '<div class="empty-state">No change is being produced, opened, checked or gated.</div>'}
      <div class="section-heading"><h2>Open findings <span class="subtle-count">${open.length}</span></h2>${gap('6.5', 'a failure that matches an open finding attaches to it and leaves the queue running')}</div>
      <section class="card card-pad">${world.findings.map(findingRow).join('') || '<p class="muted" style="margin:0">No open finding.</p>'}${world.findings.length ? '<p class="gap-note">Gap 6.5, as the case wants it: a failure at a tip or a head that matches an open finding is attached to it; the queue keeps landing, and the finding’s goal hears.</p>' : ''}</section>
      <div class="section-heading"><h2>Recent landings</h2></div>
      ${landings ? `<section class="card side-card"><div class="aside-list">${landings}</div></section>` : '<div class="empty-state">Nothing has landed yet.</div>'}`);
  };

  pages.workers = () => {
    const used = world.workers.reduce((n, w) => n + liveRuns(w.name).length, 0);
    const total = world.workers.reduce((n, w) => n + w.slots, 0);
    const cards = world.workers.map(w => {
      const runs = liveRuns(w.name);
      const h = w.health;
      const runRows = runs.map(k => {
        const t = T(k);
        const what = t.run && t.run.waiting
          ? fact(`jig: run ${t.run.n} of ${k}, waiting`, `run ${t.run.n} waits, holding its slot`)
          : `${kind(k)}${t.pool ? ` · ${fact(`jig: a slot of ${t.parent}'s pool`, `${t.parent}’s pool`)}` : ''}`;
        return `<div class="worker-run"><span>${chip(k)}</span><small>${what} · ${model(k)}</small></div>`;
      }).join('') || '<p class="muted tiny" style="margin:4px 0 0">No run here now.</p>';
      const idle = Object.keys(world.tasks).filter(k => T(k).hostKind === w.kind && !runs.includes(k) && isLive(k) && T(k).kind !== 'change');
      return `<section class="card card-pad worker-card"><div class="card-top"><h2>${conf('the worker’s name, in its configuration', esc(w.name))} <small>${conf('the worker’s description and host kind, in its configuration', `${esc(w.what)} · host kind ${esc(w.kind)}`)}</small></h2><span class="tag green">${fact('jig: the worker’s connection', 'Connected')}</span></div>
        <div class="meta-row" style="margin-top:10px">${slotsGlyph(runs.length, w.slots)}<span>${fact('jig: the slots it declares, and its live runs', `${runs.length} of ${w.slots} slots in use · ${w.slots - runs.length} free`)}</span>${w.note ? `<span>· ${conf('the worker’s description, in its configuration', esc(w.note))}</span>` : ''}</div>
        <div class="worker-runs">${runRows}</div>
        ${idle.length ? `<p class="muted tiny" style="margin:8px 0 0">Of host kind ${esc(w.kind)}, with no run now: ${idle.map(k => `<a class="text-link" href="${href(k)}">${fact(`${k}: its number and title`, `${k} ${esc(T(k).title)}`)}</a>`).join(', ')}.</p>` : ''}
        <div class="rule"></div>
        <div class="worker-logins"><span>Programs’ logins on it</span>${w.logins.map(([n, s]) => `<span>${fact('temper: a login the worker declares, and its last check', `<b>${esc(n)}</b> · <span class="ok">${esc(s)}</span>`)}</span>`).join('') || `<span>${fact('temper: the logins the worker declares', 'none')}</span>`}</div>
        ${world.config.ops && h ? src('ops', 'the ops application: a later application feeding temper, not temper’s facts', `<div class="ops-strip"><div class="ops-strip-head"><span>From the ops application · not temper’s facts</span>${gap('6.4', 'worker health')}</div>
          <dl><div><dt>Load per core</dt><dd>${esc(h.load)}</dd></div><div><dt>Builds queued</dt><dd>${esc(h.builds)}</dd></div><div><dt>Disk free · floor</dt><dd>${esc(h.disk)} · ${esc(h.floor)}</dd></div><div><dt>Memory</dt><dd>${esc(h.memory)}</dd></div></dl></div>`) : ''}
      </section>`;
    }).join('');
    const waiting = Object.values(world.tasks).filter(t => t.phase === 'waiting-slot').length;
    return listPage('workers', [[project(), 'board.html'], ['Workers']], `
      <div class="stats-row"><div class="card stat-card"><span class="muted tiny">Workers</span><strong class="value">${fact('jig: the workers registered', world.workers.length)}</strong><small>${fact('jig: their connections', 'all connected')}</small></div><div class="card stat-card"><span class="muted tiny">Slots in use</span><strong class="value">${fact('jig: live runs over declared slots', `${used} of ${total}`)}</strong><small>${fact('jig: per worker', world.workers.map(w => `${w.name} ${liveRuns(w.name).length}/${w.slots}`).join(' · '))}</small></div><div class="card stat-card"><span class="muted tiny">Free slots</span><strong class="value">${fact('jig: declared slots with no run', total - used)}</strong><small>${fact('jig: per worker', world.workers.map(w => `${w.name} ${w.slots - liveRuns(w.name).length}`).join(' · '))} ${gap('6.7', 'fleet occupancy, read by a coordinator')}</small></div><div class="card stat-card"><span class="muted tiny">Waiting for a slot</span><strong class="value">${fact('jig: runs waiting for a slot', waiting)}</strong><small>runs placed by host kind, model and source</small></div></div>
      <div class="section-heading"><h2>Workers</h2><span class="muted tiny">Each declares its slots. A run goes to a worker of its task’s host kind.</span></div>
      <div class="worker-grid${world.workers.length === 1 ? ' one' : ''}">${cards}</div>`);
  };

  // The person's LLM accounts, in temper's words (client README, 5.5): a
  // row for each account, folded, with its sources in order, what it pays
  // for and who uses it, and the acts its state offers. A plan shows its
  // limit, never its windows: OpenAI reports none to temper, and Manage
  // usage reaches them. A program's windows are gap 6.3's.
  const MANAGE = 'Mockup: ChatGPT’s settings open in a new tab, at chatgpt.com/settings/usage: the plan’s usage, and a weekly cap for temper.';
  const manageUsage = (cls = 'text-link') => conf('the provider’s manage-usage address, in the configuration: chatgpt.com/settings/usage', `<a class="${cls}" href="#" data-later="${esc(MANAGE)}">Manage usage ↗</a>`);
  const displayMark = () => ahead('display', 'the account’s display, its email: jig carries none yet, so the heading drops it until it does (client README, 5.5)');
  const SOURCE_STATES = {
    usable: ['green', 'Ready'],
    limited: ['amber', 'Limited'],
    refused: ['red', 'Refused'],
    'needs sign-in': ['red', 'Signed out'],
    misconfigured: ['red', 'Misconfigured'],
    unloaded: ['red', 'Can’t be used'],
  };
  const sourceState = s => s.state || 'usable';
  // The source that decides, in this slice: the first.
  const accountState = a => sourceState(a.sources[0]);
  const ownAccounts = () => world.accounts.filter(a => a.provider !== demo?.provider);
  function sourceKind(a, s) {
    if (s.kind === 'sign-in') return `${a.name} sign-in`;
    if (s.kind === 'program') return `${s.program} login ${s.name}`;
    return 'API key';
  }
  function holderWords(a) {
    const holder = a.holder === 'the deployment' ? 'held by the deployment' : `held by ${a.holder}`;
    return a.sources.every(s => s.kind === 'key') ? holder : `${a.provider} · ${holder}`;
  }
  const isPlan = a => a.sources.some(s => s.kind === 'sign-in');
  // An account's heading (client README, 5.5, An account): its name, its
  // display where jig gives one, and its holder.
  const accountHeading = a => fact('client README, 5.5, An account: OwnerAccountLine.{provider, display, holder} (jig’s client/domain/contract.md, 7.1)', `${esc(a.name)}${isPlan(a) ? future('display', esc(` · ${a.sources[0].name}`)) : ''} · ${esc(holderWords(a))}`);
  const paysFor = a => (isPlan(a) ? 'Your own work, and projects where you’re the only member' : 'Your own work');
  const accountUsers = a => Object.keys(world.tasks).filter(k => T(k).account === a.name && isLive(k) && T(k).kind !== 'change').sort((x, y) => x.slice(1) - y.slice(1));

  // A source's state, in the words of 5.5's table, with its acts.
  function stateNotice(a) {
    const s = a.sources[0];
    const act = (label, why, cls = 'button') => `<button class="${cls}" data-later="${esc(why)}">${label}</button>`;
    const again = act('Continue with ChatGPT', 'Mockup: a sign-in starts, as Connect a model’s: the address to open, then this row reads Ready.', 'button primary');
    const key = act('Use an API key', 'Mockup: the key’s form, as Connect a model’s; the key joins this account.');
    const resume = act('Resume', 'Mockup: temper tries the plan again now; its tasks go on if OpenAI allows them.');
    const keyFirst = a.sources.some(x => x.kind === 'key') ? act('Use the API key first', 'Mockup: the key moves before the sign-in; the account’s waiting tasks go on, billed to the key.') : '';
    const [words, acts, red] = {
      limited: s.reset != null
        ? [`Limited by ${a.provider} until ${hm(s.reset)}. Tasks on it wait.`, resume + keyFirst]
        : [`temper reached its usage limit on your ${a.plan}. Tasks on it wait.`, manageUsage('button') + resume + keyFirst],
      refused: s.why === 'declined'
        ? ['You signed in without plan usage.', again + key, true]
        : [`This ${a.name} account can’t be used for plan usage in temper. Plan usage needs an eligible ${a.plan}.`, key + act('Sign in to another account', 'Mockup: sign out of ChatGPT in your browser first; then a sign-in starts, as Connect a model’s.'), true],
      'needs sign-in': s.unconfirmed
        ? [`temper removed its tokens, but ${a.name} didn’t confirm. Disconnect temper in ${a.name}’s settings, under Apps.`, again, true]
        : [`Signed out of ${a.name}.`, again, true],
      misconfigured: [`${a.provider} refused temper’s sign-in. Check the configuration’s registration.`, '', true],
      unloaded: [`Can’t be used on ${s.host}: ${s.why}.`, '', true],
    }[sourceState(s)] || [];
    if (!words) return '';
    return `<div class="notice${red ? ' red' : ''} account-notice"><div>${fact('jig: the deciding source’s state, in temper’s words', esc(words))}${acts ? `<div class="button-row">${acts}</div>` : ''}</div></div>`;
  }

  function sourceRow(a, s, i) {
    const [tone, word] = SOURCE_STATES[sourceState(s)];
    const what = s.kind === 'program' ? `on ${s.hosts.map(esc).join(', ')}`
      : s.kind === 'sign-in' ? `${esc(s.name)}${s.since != null ? ` · signed in ${clock(s.since)}` : ''}` : esc(s.name);
    const usage = s.kind === 'program' && s.usage
      ? `<div class="usage">${fact('gap 6.3: the plan’s usage per window, as the provider reports it', `${usageLine('5-hour window', s.usage.window, s.usage.windowReset != null ? hm(s.usage.windowReset) : null, null)}${usageLine('Week', s.usage.week, s.usage.weekReset, s.usage.pace)}`)}</div>`
      : '';
    return `<div class="source-row${usage ? '' : ' plain'}"><span class="queue-no">${i + 1}</span><div><h3>${fact('jig: the source’s kind, in temper’s words', esc(sourceKind(a, s)))} <span class="tag ${tone}">${fact('jig: the source’s state, in temper’s words', word)}</span>${s.use === 'In use' ? ` <span class="tag">${fact('jig: a run uses it now', 'In use')}</span>` : ''}</h3><p>${conf('the source’s name, and for a program’s, the hosts it is signed in on', what)}</p></div>${usage}</div>`;
  }

  function accountActs(a) {
    const first = a.sources[0];
    const remove = '<button class="button" data-later="Mockup: the account is removed, with its sources.">Remove</button>';
    if (first.kind === 'sign-in' && sourceState(first) === 'needs sign-in') return remove;
    const addKey = a.sources.some(s => s.kind === 'key') ? '' : '<button class="button" data-later="Mockup: a key joins this account, after its sign-in; a run takes it when the plan can’t serve it.">Add an API key</button>';
    if (first.kind === 'sign-in') return `${manageUsage()}${addKey}<button class="button" data-later="Mockup: temper revokes the sign-in at ChatGPT and removes its tokens; the row reads Signed out.">Sign out</button>`;
    if (first.kind === 'program') return `<button class="button" data-later="Mockup: temper program-login ${esc(first.name)}, to run on another host.">Sign in on another host</button><button class="button" data-later="Mockup: the login is removed on every host that has it.">Sign out on every machine</button>`;
    return '<button class="button" data-later="Mockup: a new key, pasted; the old is erased once the new is kept.">Replace</button><button class="button" data-later="Mockup: the key is erased, and the account with it.">Remove</button>';
  }

  function accountRow(a, open = false, signing = null) {
    const [tone, word] = SOURCE_STATES[accountState(a)];
    const waiting = (a.waiting || []).filter(k => T(k));
    const users = accountUsers(a).filter(k => !waiting.includes(k));
    const name = `${esc(accountName(a))}${isPlan(a) ? ` ${displayMark()}` : ''}`;
    const summary = [a.models ? a.models.join(', ') : '', a.models ? paysFor(a).replace(/^Y/, 'y') : paysFor(a), waiting.length ? `${plural(waiting.length, 'task')} waiting` : users.length ? `used by ${users.join(', ')}` : 'no task uses it now'].filter(Boolean).join(' · ');
    const kv = (k, v) => `<div class="kv"><span>${k}</span><strong>${v}</strong></div>`;
    const links = ks => ks.map(k => `<a class="text-link" href="${href(k)}">${k}</a>`).join(', ');
    return `<details class="card forge-row account-row${fresh(a)}"${open ? ' open' : ''}><summary><div class="forge-name">${fact('jig: the account’s provider and display name, its provider and holder, in temper’s words', `${name}<small>${esc(holderWords(a))}</small>`)}</div><span class="tag ${tone}">${fact('jig: the deciding source’s state, in temper’s words', word)}</span><div class="forge-summary-line"><p class="muted tiny">${fact('jig: its models, its uses, and the tasks on it', esc(summary))}</p></div></summary><div class="card-pad">
        ${signing ? signInCard(signing.state, false) : stateNotice(a)}
        <p class="muted tiny" style="margin:0 0 4px">Its sources in order: a run takes the first usable one.</p>
        ${a.sources.map((s, i) => sourceRow(a, s, i)).join('')}
        <div class="rule"></div>
        <div class="repo-lines">
          ${a.models ? kv('Models', conf('the models its provider’s endpoints serve, in the configuration', esc(a.models.join(', ')))) : ''}
          ${kv('Pays for', fact('jig: the account’s uses; a plan serves its holder alone', esc(paysFor(a))))}
          ${kv('Used by', users.length ? `${fact('jig: the live tasks whose runs use it', links(users))} · ${fact('jig: their runs now', `${users.filter(k => T(k).phase === 'running').length} running`)}` : 'none')}
          ${kv('Waiting for it', fact('jig: tasks waiting for the account; temper never moves them to another', waiting.length ? links(waiting) : 'none'))}
        </div>
        <div class="button-row account-acts">${accountActs(a)}</div>
      </div></details>`;
  }

  function accountNavMark() {
    const own = ownAccounts();
    const needs = own.some(a => ['refused', 'needs sign-in', 'misconfigured', 'unloaded'].includes(accountState(a)));
    const limited = own.some(a => accountState(a) === 'limited');
    if (!needs && !limited) return '';
    return `<i class="nav-mark ${needs ? 'red' : 'amber'}" aria-label="${needs ? 'an account needs you' : 'an account is limited'}"></i>`;
  }

  // The ChatGPT row in each other state, built on the world's own.
  function chatgptLike(sign) {
    const a = ownAccounts().find(a => a.name === 'ChatGPT');
    const base = a || { name: 'ChatGPT', plan: 'ChatGPT plan', provider: 'OpenAI', holder: world.person, models: ['gpt-6-luna'], sources: [{ name: 'sam@example.com', kind: 'sign-in' }] };
    return { ...base, at: null, waiting: sign.waiting, sources: [{ ...base.sources[0], use: 'Idle', ...sign.source }, ...(sign.more || [])] };
  }

  pages.accounts = () => {
    const own = ownAccounts();
    const play = demoRunning() ? world.accounts.find(a => a.provider === demo.provider) : null;
    const signing = params.has('paste') ? { state: params.get('paste') } : world.modelSignIn;
    const rows = own.map(a => accountRow(a, !!signing && a.name === 'ChatGPT', a.name === 'ChatGPT' ? signing : null));
    const pending = signing && !own.some(a => a.name === 'ChatGPT') ? flowWrap(signInCard(signing.state, false)) : '';
    const windows = own.some(a => a.sources.some(s => s.kind === 'program' && s.usage));
    const key = { name: 'sam-key', kind: 'key', use: 'Idle' };
    const variants = worldKey === 'first-run' ? `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: the ChatGPT row in each state of client README 5.5’s table, and an API key’s and a Claude subscription’s rows, to compare.</p><h3 id="other-states-signing">A sign-in under way, in its account’s row</h3>${accountRow(chatgptLike({ source: { state: 'needs sign-in' } }), true, { state: 'waiting' })}${[
      ['Limited: temper reached the plan’s limit, with a key after it', chatgptLike({ source: { state: 'limited' }, more: [key], waiting: ['T1'] })],
      ['Limited, until the reset OpenAI gave', chatgptLike({ source: { state: 'limited', reset: 5 * 60 }, waiting: ['T1'] })],
      ['Refused: signed in without plan usage', chatgptLike({ source: { state: 'refused', why: 'declined' } })],
      ['Refused: not eligible', chatgptLike({ source: { state: 'refused', why: 'not eligible' } })],
      ['Signed out', chatgptLike({ source: { state: 'needs sign-in' } })],
      ['Signed out, but ChatGPT didn’t confirm', chatgptLike({ source: { state: 'needs sign-in', unconfirmed: true } })],
      ['Misconfigured', chatgptLike({ source: { state: 'misconfigured' } })],
      ['Can’t be used on a host', chatgptLike({ source: { state: 'unloaded', host: 'worker', why: 'its record didn’t load' } })],
      ['An OpenAI API key, for comparison', { name: 'OpenAI', provider: 'OpenAI', holder: world.person, models: ['gpt-6-luna'], sources: [key] }],
      ['A Claude subscription, where the configuration offers it, for comparison', { name: 'Claude', plan: 'Claude subscription', provider: 'Anthropic', holder: world.person, sources: [{ name: 'claude-sam', kind: 'program', program: 'Claude Code', hosts: ['worker'], use: 'Idle' }] }],
    ].map(([heading, a]) => `<h3>${esc(heading)}</h3>${accountRow(a, true)}`).join('')}</section>` : '';
    const empty = `<div class="empty-state">No model of yours yet. <a class="button primary" href="${esc(u('connect-model.html'))}">Connect a model</a></div>`;
    const playground = play ? `<div class="section-heading"><h2>The playground</h2></div><details class="card forge-row forge-demo"><summary><span>${fact('temper: the demo’s model, the deployment’s account', 'Scripted model · the playground’s, until a model of yours connects · held by the deployment')}</span></summary><div class="card-pad"><p class="muted tiny" style="margin:0">It plays the playground’s scripts: it answers the suggested requests, and nothing else. It costs nothing, and nothing leaves this machine.</p></div></details>` : '';
    return listPage('accounts', [['Accounts']], `<div class="section-heading" style="margin-top:0"><div><h2>Your accounts <span class="subtle-count">${fact('jig: your LLM accounts, the demo’s excluded', esc(own.length))}</span></h2><p class="muted tiny">The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it.</p></div><a class="button" href="${esc(u('connect-model.html'))}">+ Connect a model</a></div>
      <div class="stack">${pending}${rows.length ? rows.join('') : pending ? '' : empty}</div>
      ${windows ? `<p class="gap-note">${gap('6.3', 'plan usage')} A program’s windows, as the account’s facts. The mark on a week is its pace: where usage would be if the week were spent evenly.</p>` : ''}
      ${playground}${variants}`);
  };

  // The paste shares the built client's words and wrappers (2ec1c6574,
  // accounts/words.rs and pages/accounts.rs). Values never enter a URL or store.
  const PASTE_WORDS = {
    elsewhere: "This browser isn't on temper's computer ›",
    link: "Open ChatGPT's sign-in ›",
    address: "Open this address in a browser on the engine's machine:",
    open: 'Open the sign-in page',
    disclosure: "Signed in, but the page couldn't be reached?",
    history: 'Go forward to that page, or find it in your history, and copy its address.',
    instructions: "When ChatGPT is done, its tab shows a page that can't be reached. Copy that page's whole address from the address bar, come back here, and paste it. If a page opens instead, such as another program's sign-in, close that program's sign-in and start again.",
    field: "Address of the page that couldn't be reached",
    finish: 'Finish connecting',
    sending: 'Finishing…',
    finishing: 'Finishing your sign-in…',
    'too-long': "That's longer than a sign-in's address. Copy the address bar alone.",
    'address-refused': "That isn't this sign-in's address. Copy the whole address of the page that couldn't be reached, from the address bar.",
    state: "That address is from an earlier sign-in. Paste the one from this sign-in's tab, or start again.",
    'too-many': "That's too many addresses from other sign-ins. Continue with ChatGPT to start again.",
    ended: 'This sign-in has ended.',
    again: "That address wasn't sent. Paste it again.",
  };
  const modelButton = (label, target, primary = false) => `<button type="button" class="button${primary ? ' primary' : ''}" data-model-go="${esc(u(target))}">${esc(label)}</button>`;
  function pasteTarget(state, elsewhere = true) {
    return `${page === 'welcome' ? 'welcome.html?at=model&' : 'connect-model.html?'}way=chatgpt&paste=${state}${elsewhere ? '&elsewhere=1' : ''}`;
  }
  function signInPaste(state, folded) {
    const restart = modelButton('Continue with ChatGPT', page === 'accounts' ? 'accounts.html?paste=waiting' : pasteTarget('waiting', false));
    const finish = worldKey === 'first-run' && stepId === 'model-pasted'
      ? `${page === 'accounts' ? 'accounts.html' : 'connect-model.html?way=chatgpt'}${page === 'accounts' ? '?' : '&'}step=5`
      : page === 'accounts' ? 'accounts.html?paste=finishing' : pasteTarget('finishing');
    let body;
    if (state === 'finishing') body = `<p role="status">${PASTE_WORDS.finishing}</p>`;
    else if (['not-waiting', 'expired'].includes(state)) body = `<p role="status">${PASTE_WORDS.ended}</p>${restart}`;
    else if (state === 'too-many') body = `<p class="forge-failure" role="alert">${PASTE_WORDS['too-many']}</p>${restart}`;
    else {
      const refusal = { address: 'address-refused', state: 'state', 'too-long': 'too-long', lost: 'again', unsent: 'again' }[state];
      // Written values are hidden, just as the built secret field is. The
      // mockup draws only the fact of a paste, never a callback's code/state.
      body = `<p>${esc(PASTE_WORDS.instructions)}</p>${refusal ? `<p class="forge-failure" role="alert">${esc(PASTE_WORDS[refusal])}</p>` : ''}
        <div data-model-paste><label class="forge-field"><span>${esc(PASTE_WORDS.field)}</span><input type="password" autocomplete="off" spellcheck="false"${state === 'written' ? ' placeholder="••••••••"' : ''}${state === 'sending' ? ' disabled' : ''}></label>
        <button type="button" class="button" data-model-finish="${esc(u(finish))}"${state === 'written' ? '' : ' disabled'}>${esc(state === 'sending' ? PASTE_WORDS.sending : PASTE_WORDS.finish)}</button></div>`;
    }
    return folded ? `<details><summary>${esc(PASTE_WORDS.disclosure)}</summary><p>${esc(PASTE_WORDS.history)}</p>${body}</details>` : body;
  }
  function signInCard(state = 'waiting', elsewhere = false) {
    const ended = ['expired', 'not-waiting'].includes(state);
    // A fake provider's start address: never a live sign-in.
    const link = `<a class="text-link" href="https://issuer.example/visit" target="_blank" rel="noopener noreferrer">${esc(elsewhere ? PASTE_WORDS.link : PASTE_WORDS.open)}</a>`;
    return `<h3>Signing in</h3>${ended ? '' : `${elsewhere ? '' : `<p>${esc(PASTE_WORDS.address)}</p>`}<p>${link}</p>`}${signInPaste(state, !elsewhere && !ended)}`;
  }
  const wayLabel = w => conf('engine.providers, offered as me.providers (jig’s client/domain/contract.md, 7.1, ProviderOffer); the label, client README 5.5, The ways', esc(w));
  function modelChoices(signIn, keyForm = 'connect-model.html?way=key') {
    const ways = world.config.ways || [];
    const chatgpt = ways.find(w => w.startsWith('Continue with'));
    const key = ways.find(w => w.startsWith('Use an') && w.endsWith('API key'));
    const elsewhere = worldKey === 'first-run' && page !== 'welcome' ? 'connect-model.html?way=chatgpt&step=model-elsewhere' : pasteTarget('waiting');
    return `<div class="forge-choices model-choices">${chatgpt ? `<section class="card card-pad"><h3>ChatGPT</h3><p>Your ChatGPT plan. Sign in with your ChatGPT account, and temper’s agents use your plan’s included usage.</p><p class="muted tiny">In a browser on the engine’s machine · about a minute</p><div class="button-row">${modelButton(chatgpt, signIn, true)}${modelButton(PASTE_WORDS.elsewhere, elsewhere)}</div></section>` : ''}${key ? `<section class="card card-pad"><h3>An OpenAI API key</h3><p>A key from OpenAI’s platform. What temper’s agents use is billed to the key’s organization.</p><p class="muted tiny">Pasted here · at once</p><a class="button" href="${esc(u(keyForm))}">${wayLabel(key)}</a></section>` : ''}</div>`;
  }
  function chatgptReady(a, wasPlayground) {
    const s = a.sources[0];
    const t = Object.keys(world.tasks).find(k => T(k).account === a.name && isLive(k) && isChat(T(k)));
    const ended = 'jig’s client/domain/contract.md, 7.1: OwnerAccounts.last_end, Made { account }';
    return `<h3>${fact(ended, `Signed in to ChatGPT${future('display', esc(` as ${s.name}`))}`)}</h3>
      <p>temper’s agents now run on your ChatGPT plan.</p>
      <div class="repo-lines">
        <div class="kv"><span>Account</span><strong>${accountHeading(a)}</strong></div>
        ${future('preview', `<div class="kv"><span>Model</span><strong>${esc(world.config.next?.model || 'gpt-6-luna')}</strong></div>`)}
        <div class="kv"><span>Pays for</span><strong>${fact('jig: the account’s uses; a plan serves its holder alone', esc(paysFor(a)))}</strong></div>
      </div>
      ${wasPlayground ? `<p>The playground’s scripted model stepped aside${t ? `: ${t}’s next run takes ${esc(T(t).model)}` : ''}.</p>` : ''}
      <div class="button-row"><a class="button primary" href="${esc(u('chats.html'))}">Start a chat</a>${manageUsage()}</div>
      <p class="muted tiny">${esc(accountName(a))} is on the <a class="text-link" href="${esc(u('accounts.html'))}">Accounts</a> page, with its details.</p>`;
  }
  function keyForm() {
    return `<div class="forge-setup welcome-model"><h3>An OpenAI API key</h3><p>From OpenAI’s platform, under API keys. What temper’s agents use is billed to the key’s organization.</p>
        <label class="forge-field"><span>Key (pasted once)</span><input type="password"></label>
        <button class="button primary" data-later="Mockup: the key makes an account, OpenAI API key, held by you, and reads Ready.">Add key</button>
        <p class="mock-note">A key reads Ready until a run is refused: nothing checks it when it is added, so a wrong key shows only once OpenAI refuses a request.</p></div>`;
  }
  function claudeSketch() {
    return `<div class="forge-setup"><h3>Your Claude subscription ${ahead('sketch', 'Claude’s subscription in Connect a model, in outline for the context: the words are client README 5.5’s, the command credentials.md’s')}</h3>
        <p>temper runs Claude Code, signed in to your Claude subscription on each host its agents run on. Name the login, then sign in on each host, in a terminal there:</p>
        <label class="forge-field"><span>Name</span><input type="text" value="claude-sam"></label>
        <pre><code>temper program-login claude-sam</code></pre>
        <p class="muted tiny">Claude Code asks you to sign in. temper never reads its login.</p>
        <button class="button" data-later="Mockup: the login is declared; its row on Accounts says, for each host, whether Claude Code is signed in there.">Use your Claude subscription</button></div>`;
  }
  function pasteGallery(wasPlayground) {
    return `<h3 id="the-paste">The paste, as built</h3>${[
      ['local', 'A sign-in under way on temper’s computer: paste folded', 'waiting', false],
      ['waiting', 'Waiting, from another computer: paste unfolded', 'waiting', true],
      ['written', 'The whole address pasted, hidden; ready to finish', 'written', true],
      ['sending', 'The finish request is out', 'sending', true],
      ['finishing', 'Finishing, until the accounts watch shows the end', 'finishing', true],
      ['address', 'Refused: address', 'address', true],
      ['state', 'Refused: state', 'state', true],
      ['too-many', 'Refused: too many; start again', 'too-many', true],
      ['too-long', 'Past the address bound; nothing sent', 'too-long', true],
      ['not-waiting', 'Refused: not waiting; sign-in gone', 'not-waiting', true],
      ['expired', 'Expired; start again', 'expired', true],
      ['lost', 'Lost; paste again', 'lost', true],
      ['unsent', 'Unsent; paste again', 'unsent', true],
      ['done', 'Done; the watch shows Ready', 'done', true],
    ].map(([id, heading, state, elsewhere]) => `<h3 id="paste-${id}"><a href="${esc(u(pasteTarget(state, elsewhere)))}">${esc(heading)}</a></h3>${modelSteps(state === 'done' ? 3 : 1, 'chatgpt')}${flowWrap(state === 'done' ? chatgptReady(chatgptLike({ source: {} }), wasPlayground) : signInCard(state, elsewhere))}`).join('')}`;
  }
  const modelSteps = (current, way) => flowSteps(current, '', ['Choose', way !== 'chatgpt' ? 'Details' : 'Signing in', 'Ready']);
  const endingCard = (words, acts, mark = '', failed = true) => `<h3>Signing in to ChatGPT ${mark}</h3><p${failed ? ' class="forge-failure"' : ''}>${words}</p>${acts ? `<div class="button-row">${acts}</div>` : ''}`;

  pages['connect-model'] = () => {
    const way = params.get('way');
    const a = ownAccounts().find(a => a.provider === 'OpenAI' && isPlan(a));
    const wasPlayground = demoRunning();
    // The ordinary departure leads to the return; elsewhere stays here.
    const signIn = worldKey === 'first-run-real' ? 'connect-model.html?way=chatgpt&step=3' : 'connect-model.html?way=chatgpt&step=5';
    let current = 0;
    let body = modelChoices(signIn);
    const signing = params.has('paste') ? { state: params.get('paste'), elsewhere: params.get('elsewhere') === '1' } : world.modelSignIn;
    if (way === 'chatgpt' && signing && signing.state !== 'done') {
      current = 1; body = flowWrap(signInCard(signing.state, signing.elsewhere));
    }
    else if (way === 'chatgpt' && (a || signing?.state === 'done')) { current = 3; body = flowWrap(chatgptReady(a || chatgptLike({ source: {} }), wasPlayground)); }
    else if (way === 'key') { current = 1; body = flowWrap(keyForm()); }
    else if (way === 'program') { current = 1; body = flowWrap(claudeSketch()); }
    const back = ['key', 'program'].includes(way) ? `<p><a class="text-link" href="${esc(u('connect-model.html'))}">‹ All ways</a></p>` : '';
    const again = '<button class="button primary" data-later="Mockup: a fresh sign-in, with a new address.">Continue with ChatGPT</button>';
    const key = '<button class="button" data-later="Mockup: the key’s form.">Use an API key</button>';
    const variants = worldKey === 'first-run' ? `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: the paste, each other ending of ChatGPT’s sign-in, back in temper, and the other ways. The external sign-in links here use a fake provider.</p>${pasteGallery(wasPlayground)}${[
      ['The same account again', 1, endingCard(fact('jig: a sign-in to a subject the person already holds', esc(`That’s ${a ? a.sources[0].name : 'sam@example.com'} again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.`)), again, '', false), undefined, 'other-states-renewed'],
      ['Signed in without plan usage', 1, endingCard(fact('jig: the sign-in refused, plan usage declined', 'You signed in without plan usage.'), again + key), undefined, 'other-states-declined'],
      ['Not eligible', 1, endingCard(fact('jig: the sign-in refused, not eligible', 'This ChatGPT account can’t be used for plan usage in temper. Plan usage needs an eligible ChatGPT plan.'), `${key}<button class="button" data-later="Mockup: sign out of ChatGPT in your browser first; then a fresh sign-in.">Sign in to another account</button>`), undefined, 'other-states-not-eligible'],
      ['Another person holds that account', 1, endingCard(fact('jig: the sign-in refused, a subject another holder holds', 'Another person here holds that ChatGPT account. To use one of yours, sign out of ChatGPT in your browser first, then Continue with ChatGPT.'), again, ahead('words', 'a sign-in refused for a subject another holder holds (jig’s accounts.md, 3.3): client README 5.5 has no words yet')), undefined, 'other-states-refused'],
      ['The port is taken, before ChatGPT opens', 1, endingCard(fact('temper: the loopback listener couldn’t bind the fixed port', 'temper can’t listen on 127.0.0.1:1455: another program has it, such as Codex’s own sign-in. Finish or close that sign-in, then Continue with ChatGPT.'), again, ahead('words', 'a sign-in whose fixed port is taken: client README 5.5 has no words yet')), undefined, 'other-states-port-taken'],
      ['Misconfigured', 1, endingCard(fact('jig: the sign-in refused by the provider, misconfigured', 'OpenAI refused temper’s sign-in. Check the configuration’s registration.'), ''), undefined, 'other-states-misconfigured'],
      ['An OpenAI API key: its details', 1, keyForm(), 'key', 'other-states-key-details'],
      ['Your Claude subscription, where the configuration offers it', 1, claudeSketch(), 'key', 'other-states-program-details'],
    ].map(([heading, at, content, w, id]) => `<h3 id="${id}">${esc(heading)}</h3>${modelSteps(at, w)}${flowWrap(content)}`).join('')}</section>` : '';
    return listPage('accounts', [['Accounts', 'accounts.html'], ['Connect a model']], `<div class="section-heading" style="margin-top:0"><h2>Connect a model</h2></div>${modelSteps(current, way)}${back}${body}${variants}`);
  };

  // The forges' part of the deployment's connectors, in temper's words
  // (client forge.md, 5.4): a row for each forge the person has acted on,
  // with its connection, its access, temper's App on GitHub and its load,
  // and the acts its kind, mode and state offer.
  function forgeConnection(f) {
    if (f.state === 'connected') return `Connected as ${f.login}`;
    if (f.state === 'setting up') return f.setup.step === 'connecting' ? 'Connecting' : f.setup.step === 'repository' ? 'Making the repository' : `Setting up: ${f.setup.step}`;
    return cap(f.state);
  }
  const PASTED = '<p class="muted tiny" style="margin:0">A pasted value is sent once and shown nowhere, then or after.</p>';
  // GitHub's ways (client forge.md, 5.4). Connect GitHub leaves for GitHub
  // in this tab at once, with no page of temper's before it, and GitHub
  // sends the tab back to the page it left; an App of your own is the one
  // GitHub way with Details. GitHub's side is drawn in its own world, at
  // the step it is at; `from` is the page the tab comes back to.
  const GH_WORLD = 'first-run-github';
  const toGithub = (at, from, step) => u(`github.html?${worldKey === GH_WORLD ? '' : `world=${GH_WORLD}&`}at=${at}&from=${from}`, step);
  const githubBack = from => ({ welcome: 'welcome.html?at=forge&forge=2', forges: 'forges.html' }[from] || 'connect-forge.html?forge=2');
  const githubAdopted = f => Object.values(world.repos).find(r => r.forge === f.n && !r.retired);
  function githubAppForm(f) {
    return `<div class="forge-setup"><h3>An App of your own</h3><p>Its id, and its private key as GitHub gives it.</p><p>On GitHub, set the App’s setup URL to ${conf('the forge’s GitHub setup URL', esc(`http://127.0.0.1:8367/forges/${f.n}/github/setup`))}, and give it the permissions and events temper’s guide lists.</p>
          <label class="forge-field"><span>App id</span><input type="text"></label>
          <label class="forge-field"><span>Private key</span><textarea rows="2"></textarea></label>
          <button class="button primary" data-later="Mockup: temper checks the App with its key; then Install on GitHub, as for temper’s own App.">Connect the App</button>${PASTED}</div>`;
  }
  // Connecting: the App is made, and installed nowhere. A connect before
  // the App is made shows nowhere: Connect GitHub again starts afresh.
  function githubConnecting(f, from = 'connect-forge') {
    return `<div class="forge-setup"><h3>Install temper’s App</h3><p>${fact('temper: the App made on GitHub, installed nowhere', esc(`${f.app.slug} is made on GitHub, under your account, and installed nowhere yet.`))} Install it, and choose the repositories temper may use.</p><a class="button primary" href="${esc(toGithub('install', from, '3'))}">Install on GitHub</a></div>`;
  }
  // Ready: installed. Choose the repository to start in, adopted into a new
  // project of its name; once one is adopted, it is ready.
  function githubReady(f, from = 'connect-forge') {
    const install = f.app.installs[0];
    const adopted = githubAdopted(f);
    if (adopted) return `<div class="forge-setup"><h3>${fact('client forge.md, 5.4, Ready: the forges watch’s Row.repositories[].{owner, name}', esc(`${adopted.name} is ready`))}</h3><p>In the new project ${fact('jig’s client/domain/contract.md, 3.2: MakeProject’s ProjectMade, and Row.repositories[].project', esc(adopted.project))}, on GitHub.</p><p>${fact('client README, 4.7, and client forge.md, 5.4: the adoption’s Adopted.kinds, a GitHub App’s status never set', 'temper may: read; push; open pull requests; land; review; comment; create issues; create and delete branches.')}</p><p>${fact('temper: on GitHub, always (client README, 4.7)', 'Commit statuses stay temper\'s own: GitHub\'s rules cannot require them.')}</p><p>${fact('temper: the branch rules the adoption read (domain forge.md, 5.1)', 'The branch has no rules.')}</p>
        <div class="button-row"><a class="button primary" href="${esc(u(`chats.html?project=${encodeURIComponent(adopted.project)}`))}">Open ${esc(adopted.project)}</a><a class="text-link" href="#" data-later="${esc(`Mockup: github.com/${adopted.name}, in a new tab.`)}">Open on GitHub ↗</a></div><p class="muted tiny">GitHub is on the <a class="text-link" href="${esc(u('forges.html'))}">Forges</a> page, with its details.</p></div>`;
    const repos = install.names.map(n => [n, (world.github?.repos[install.account] || []).find(([r]) => r === n)?.[1]]);
    const target = from === 'welcome' ? 'welcome.html?at=model' : 'connect-forge.html?forge=2';
    const first = repos[0][0];
    return `<div class="forge-setup"><h3>Choose a repository to start in</h3><p>${fact('client README, 4.6: the forges watch’s Row.installations[].account, and the App’s slug', variant('grant', esc(`${f.app.slug} is installed on ${install.account}.`), esc(`${f.app.slug} is installed on ${install.account}, with ${install.all ? `all its repositories, ${install.names.length}` : `${plural(install.names.length, 'repository', 'repositories')} you chose`}.`)))}</p>
        <div class="pick-list" data-pick="${esc(u(target, '5'))}" data-owner="${esc(install.account)}">${repos.map(([n, priv], i) => `<label class="pick-row"><input type="radio" name="pick" value="${esc(n)}"${i ? '' : ' checked'}><span>${fact('client forge.md, 4.3: the Reachable read’s rows, Reached.{owner, name, private}', esc(`${install.account}/${n}`))}</span>${priv ? '<span class="tag">private</span>' : ''}</label>`).join('')}</div>
        <label class="forge-field"><span>Into a new project, named</span><input type="text" value="${esc(first)}" data-pick-project></label>
        <div class="button-row"><a class="button primary" data-pick-go href="${esc(u(`${target}&pick=${encodeURIComponent(first)}`, '5'))}">Adopt ${esc(`${install.account}/${first}`)}</a></div>
        <p class="muted tiny">Not listed? <a class="text-link" href="${esc(toGithub('install', from, '4'))}">Choose its repositories on GitHub</a>, and GitHub sends you back here.</p></div>`;
  }
  // GitHub's steps: Connect GitHub has no Details; an App of your own does.
  function githubSteps(f) {
    const labels = f.mode === 'managed' ? ['Choose', 'Connecting', 'Ready'] : ['Choose', 'Details', 'Connecting', 'Ready'];
    const n = labels.length;
    return flowSteps(f.state === 'connected' ? (githubAdopted(f) ? n : n - 1) : f.state === 'connecting' ? n - 2 : 1, '', labels);
  }
  // What the clients' listener answers itself, where GitHub's way can't go
  // on: its own short page, with the way back (client forge.md, 9.1).
  function githubRefusal(why, from = 'connect-forge') {
    const [head, words, back, step] = {
      request: ['GitHub sent a request, not an installation', 'Only an owner of an organization can install temper’s App there, and temper can’t wait for an owner to approve it yet: an installation approved later isn’t used. Install it on your own account, or on an organization you own.', githubBack(from), '3'],
      ended: ['This link has ended', 'It was used already, it expired, or temper restarted since. Start again from temper.', 'connect-forge.html?forge=2', '1'],
    }[why];
    return `<div class="forge-setup"><h3>${esc(head)}</h3><p>${fact('client forge.md, 9.1: the clients’ listener’s own page, in its words', esc(words))}</p><a class="button primary" href="${esc(u(back, step))}">Back to temper</a></div>`;
  }
  function ownedCommands(f) { return `<div class="forge-setup"><h3>Set it up from a terminal</h3><p>temper was not started by temper box, so it does not start a Forgejo itself. Run these, keep the last two running (a service manager is best), then restart temper:</p><pre><code>${conf('the owned forge’s number in the configuration', esc(`temper forge fetch ${f.n}\ntemper forge prepare ${f.n}\ntemper forge run ${f.n} forgejo\ntemper forge run ${f.n} runner`))}</code></pre><p>Once it is connected, make your first repository here.</p></div>`; }
  function ownedForm(f, target = `connect-forge.html?forge=${f.n}`, at = '6') {
    const setup = f.setup;
    return `<div class="forge-setup">
        <h3>Forgejo on this machine</h3>
        <p>temper box downloads Forgejo ${fact('client forge.md, 5.4: the forges watch’s Row.owned.forgejo, its pin', esc(f.versions.forgejo))} and its runner, starts them, and makes your first repository, with a README and a CI workflow. No restart.</p>
        <div class="forge-fields"><label class="forge-field"><span>Organization</span><input type="text" value="${esc(setup.org)}"></label><label class="forge-field"><span>Repository</span><input type="text" value="${esc(setup.repo)}"></label></div>
        <div class="forge-fields"><label class="forge-field"><span>Into a new project, named</span><input type="text" value="${esc(setup.project)}"></label></div>
        <ul class="forge-facts"><li>Its CI jobs run on this machine, as you, with no isolation.</li><li>Forgejo stops when temper box stops. Its repositories are kept in temper’s data directory.</li></ul>
        <a class="button primary" href="${esc(u(target, at))}">Set up a Forgejo here</a>
      </div>`;
  }
  const ownedFailure = () => `<p class="forge-failure">${fact('temper box: the failed preparation', 'Preparing failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.')}</p>`;
  const stoppedFailure = 'Forgejo refused temper’s token. Restart temper box: its preparation mends temper’s admin, and the bot’s tokens with it.';
  const chosenValues = f => `<p class="muted tiny">${fact('temper: the set-up’s chosen repository and project', esc(`${f.setup.org}/${f.setup.repo}, into ${f.setup.newProject ? 'a new project' : 'the project'}, ${f.setup.project}`))}</p>`;
  function ownedProgress(f, { allDone = false, failedAt = null } = {}) {
    const order = ['fetching', 'preparing', 'starting', 'connecting', 'repository'];
    const current = allDone ? order.length : order.indexOf(failedAt || f.setup.step);
    const lines = [esc(`${current > 0 ? 'Fetched' : 'Fetching'} Forgejo ${f.versions.forgejo} and its runner ${f.versions.runner}`), `Preparing: its database, temper’s admin, your account${future('login', esc(` ${f.person}`))}`, 'Starting Forgejo and its runner', 'Connecting as temper', esc(`Making ${f.setup.org}/${f.setup.repo}, and adopting it into ${f.setup.project}`)];
    return fact('client forge.md, 5.4, Setting up: Row.connection (SettingUp.step, Connecting), Row.first.state, the adoption; Row.owned’s versions', `<ol class="forge-progress">${lines.map((line, i) => `<li${i < current ? ' class="done"' : i === current ? ` class="${failedAt ? 'failed' : 'current'}"` : ''}>${line}</li>`).join('')}</ol>`);
  }
  function forgeActs(f) {
    if (f.state === 'setting up') return `${ownedProgress(f)}<a class="text-link" href="${esc(u(`connect-forge.html?forge=${f.n}`))}">Open its set-up ›</a>`;
    if (f.state === 'set up failed') return `${ownedProgress(f, { failedAt: 'preparing' })}<a class="button" href="${esc(u(`connect-forge.html?forge=${f.n}`))}">Set up again</a>`;
    if (f.kind === 'GitHub' && f.state === 'connecting') return `<div class="rule"></div>${githubConnecting(f, 'forges')}`;
    if (f.kind === 'GitHub') return `<div class="rule"></div><div class="button-row"><button class="button" data-later="Mockup: GitHub’s install page, in this tab, as at step 3; GitHub sends the tab back to Forges, and the row lists the new installation.">Install on another account</button><button class="button" data-later="Mockup: a new App key, pasted; the old is erased once the new is kept.">Replace</button></div>`;
    if (f.mode === 'owned' && f.state === 'stopped') return '';
    if (f.mode === 'owned' && f.state === 'connected') return `<div class="rule"></div><div class="button-row">
        <a class="button primary" href="#" data-later="${esc(`Mockup: Forgejo opens in a new tab, signed in as ${f.person} through temper.`)}">Open Forgejo ↗</a>${ahead('sign-in', 'signing in to an owned Forgejo through temper, as its OpenID provider: no mission yet')}
        <button class="button" data-later="Mockup: the set-up’s three fields again; temper makes the repository, grants its bot, and adopts it.">New repository</button>${ahead('B4 M4', 'a repository made by temper, which holds the admin’s token')}
        <button class="button" data-later="Mockup: the adoption form (client README, 4.4) has no mockup.">Adopt a repository</button></div>
        <p class="muted tiny">Open Forgejo signs you in as ${conf('the person’s Forgejo login', esc(f.person))}, through temper.</p>
        <p class="gap-note">Until signing in through temper: sign in as ${conf('the person’s Forgejo login', esc(f.person))} with the password in ${conf('the owned forge’s person password file', esc(`~/.local/share/temper/forge-${f.n}/person_password`))}. Forgejo asks for a new one at once.</p>`;
    // A Forgejo whose people sign in through temper, as its OpenID
    // provider (draft sign-in.md, section 5).
    if (f.through && f.state === 'connected') return `<div class="rule"></div><div class="button-row">
      <a class="button primary" href="#" data-later="${esc(`Mockup: ${f.web}/user/oauth2/temper, in a new tab: Forgejo starts its sign-in with the temper source, temper answers at once, and Forgejo opens signed in as ${world.site.login}.`)}">Open Forgejo ↗</a>${ahead('provider', 'temper as Forgejo’s OpenID provider: draft sign-in.md, section 5')}
      <button class="button" data-later="Mockup: the set-up’s three fields again; temper makes the repository, grants its bot, and adopts it.">New repository</button>
      <button class="button" data-later="Mockup: the adoption form (client README, 4.4) has no mockup.">Adopt a repository</button>
      <button class="button" data-later="Mockup: the admin’s token erased; temper keeps its bot and tokens, and the forge becomes connected.">Forget the admin’s token</button></div>
      <p class="muted tiny">Open Forgejo signs you in as ${conf('the member’s login', esc(world.site.login))}, through temper: Forgejo’s sign-in is temper’s.</p>`;
    if (f.kind === 'Forgejo' && f.state === 'connected' && ['managed', 'connected'].includes(f.mode)) return `<div class="rule"></div><div class="button-row">
      <a class="text-link" href="#" data-later="Mockup: its web page; you sign in there your own way.">Open Forgejo ↗</a>
      ${f.mode === 'managed' ? '<button class="button" data-later="Mockup: the set-up’s three fields again; temper makes the repository, grants its bot, and adopts it.">New repository</button>' : ''}
      <button class="button" data-later="Mockup: the adoption form (client README, 4.4) has no mockup.">Adopt a repository</button>
      ${f.mode === 'managed' ? '<button class="button" data-later="Mockup: the admin’s token erased; temper keeps its bot and tokens, and the forge becomes connected.">Forget the admin’s token</button>' : '<button class="button" data-later>Replace</button><button class="button" data-later>Connect as its admin</button>'}</div>
      ${f.mode === 'connected' ? `<p class="muted tiny">Adopting grants temper nothing here: make its bot, ${fact('temper: the forge’s bot login', esc(f.login))}, a collaborator with write on each repository first.</p>` : ''}`;
    return '';
  }
  function forgeLoad(f) {
    const l = f.load;
    if (!l) return '';
    return `<div class="kv"><span>Requests</span><strong>${fact('temper: the forge’s request budget, the configuration’s rate per window (domain forge.md, 5.4)', esc(`${l.used.toLocaleString('en')} of ${l.rate.toLocaleString('en')} this ${l.window} · ${l.reserve} kept for keeping up and the slow pass`))}</strong></div>
      <div class="kv"><span>Used by</span><strong>${fact('temper: what used the budget, by queue', esc(l.uses.map(([q, n]) => `${q} ${n.toLocaleString('en')}`).join(' · ')))}</strong></div>
      <div class="kv"><span>Pause for its rate</span><strong>${fact('temper: a pause the forge asked for', 'none')}</strong></div>
      <div class="kv"><span>Outbox</span><strong>${fact('temper: the effects waiting, and those uncertain, being looked for', esc(`${plural(l.outbox.waiting, 'effect')} waiting · ${l.outbox.uncertain ? `${l.outbox.uncertain} uncertain` : 'none uncertain'}`))}</strong></div>
      <div class="kv"><span>Webhooks last heard</span><strong>${fact('temper: when each repository’s webhooks were last heard', esc(l.hooks.map(([r, at]) => `${r} ${ago(at)}`).join(' · ')))}</strong></div>`;
  }
  function forgeBody(f) {
    if (['setting up', 'set up failed'].includes(f.state)) return forgeActs(f);
    const ok = f.state === 'connected';
    const access = f.kind === 'Forgejo' ? { fake: 'temper’s own admin token · the playground’s forge', owned: 'Site admin, with temper’s own token', managed: 'Site admin, with a token you gave', connected: 'Its bot’s tokens, which you gave' }[f.mode] : { managed: 'temper’s App, made on GitHub from temper’s manifest', connected: 'An App of your own, with the key you gave' }[f.mode];
    const repos = Object.values(world.repos).filter(r => !r.retired && r.forge === f.n);
    const app = f.app ? `<div class="kv"><span>temper’s App</span><strong>${fact('temper: the App, and its installations', `${esc(f.app.slug)} · ${f.app.installs.length ? f.app.installs.map(i => `<a class="text-link" ${worldKey === GH_WORLD ? `href="${esc(toGithub('installation', 'forges', stepId))}"` : 'href="#" data-later="Mockup: the installation’s page, on GitHub."'} title="Choose its repositories on GitHub">${esc(i.account)}, ${esc(i.all ? 'all repositories' : plural(i.names.length, 'selected repository', 'selected repositories'))}</a>`).join(', ') : 'installed nowhere yet'}`)}</strong></div>` : '';
    return `<div class="repo-lines">${ok ? `<div class="kv"><span>Account</span><strong>${fact('temper: the user the forge’s records hold, by login and id', esc(`${f.login} · ${f.kind === 'GitHub' ? 'App user' : 'user'} ${f.id}`))}</strong></div>` : ''}
        ${ok || f.state === 'stopped' ? `<div class="kv"><span>Access</span><strong>${fact('temper: how temper holds the forge: its own admin token, one a person gave, or the bot’s tokens alone (setup.md, the modes)', esc(access))}</strong></div>` : ''}
        ${f.mode === 'owned' && f.state !== 'not set up' ? `<div class="kv"><span>Runs</span><strong>${fact('temper: the owned forge’s setup, from the configuration and the pins', esc(`On this machine, with temper box · Forgejo ${f.versions.forgejo} and its runner ${f.versions.runner} · jobs labelled ${f.label} run here, as you`))}</strong></div>` : ''}
        ${ok && repos.length ? `<div class="kv"><span>Repositories</span><strong>${fact('temper and jig: the repositories adopted from this forge, and their projects', repos.map(r => `${esc(r.name)} · project <a class="text-link" href="${esc(u(`changes.html?project=${encodeURIComponent(r.project || world.project)}`))}">${esc(r.project || world.project)}</a>`).join('<br>'))}</strong></div>` : ''}
        ${app}${forgeLoad(f)}</div>
      ${forgeActs(f)}`;
  }

  function elsewhereHow(next) {
    return `<div class="forge-elsewhere"><h3>A Forgejo elsewhere</h3><p>Add its address to the configuration and restart temper box. Then connect as its admin, and temper makes and keeps its bot, or paste its bot’s tokens.</p>

          <p>For a forge at forge.example, add this entry to application.forges in ~/.config/temper/temper.json. Replace the example host with yours; use an unused forge number.</p>
          <pre><code>{
  "number": ${conf('application.forges: one past the highest forge number (client forge.md, 5.4)', esc(next))},
  "kind": "forgejo",
  "api": {"host": "forge.example", "port": 443},
  "transport": {"tls": {
    "server_name": "forge.example", "trust": null
  }},
  "git": "https://forge.example",
  "web": null,
  "log_origins": null
}</code></pre>
          <p>Add its git identity to application.identities, using an unused identity number and the same forge number:</p>
          <pre><code>{
  "number": ${conf('application.forges: one past the highest forge number (client forge.md, 5.4)', esc(next))}, "forge": ${conf('application.forges: one past the highest forge number (client forge.md, 5.4)', esc(next))}, "login": null,
  "author": {
    "name": "temper",
    "email": "temper-bot@forge.example"
  }
}</code></pre>
          <p>To connect as its admin, add its setup entry to application.setup:</p>
          <pre><code>{
  "forge": ${conf('application.forges: one past the highest forge number (client forge.md, 5.4)', esc(next))},
  "bot": {"login": "temper", "email": "temper-bot@forge.example"},
  "mode": "managed"
}</code></pre>
          <p>Without it, temper takes its bot’s tokens only.</p>
          <p>Restart temper box, then connect it from this page.</p>
        </div>`;
  }
  function forgeRow(f, open = false) {
    const repos = Object.values(world.repos).filter(r => !r.retired && r.forge === f.n);
    const line = f.state === 'setting up' ? chosenValues(f) : f.state === 'set up failed' ? ownedFailure() : f.state === 'stopped' ? `<p class="muted tiny">${fact('temper: why the forge stopped', esc(stoppedFailure))}</p>` : `<p class="muted tiny">${fact('temper and jig: the forge’s repositories and their projects', esc(repos.map(r => `${r.name} · project ${r.project || world.project}`).join(' · ')))}</p>`;
    return `<details class="card forge-row"${open ? ' open' : ''}><summary><div class="forge-name">${conf('the forge’s name, number, kind and web address', `${esc(forgeName(f))}<small>forge ${esc(f.n)} · ${esc(f.kind)} · ${esc(f.web)}</small>`)}</div><span class="tag ${f.state === 'connected' ? 'green' : ['set up failed', 'stopped'].includes(f.state) ? 'red' : 'amber'}">${fact('temper: the forge’s connection', esc(forgeConnection(f)))}</span><div class="forge-summary-line">${line}</div></summary><div class="card-pad">${forgeBody(f)}</div></details>`;
  }
  function flowSteps(current, label3 = 'Setting up', labels = ['Choose', 'Details', label3, 'Ready']) {
    return `<div class="steps flow-steps" style="grid-template-columns:repeat(${labels.length}, minmax(0, 1fr))">${labels.map((label, i) => `<div class="step${i < current ? ' done' : i === current ? ' current' : ''}">${esc(label)}</div>`).join('')}</div>`;
  }
  function adminCard(f) {
    const s = f.setup;
    const stopped = f.state === 'not connected' && params.get('stop') || f.stop;
    if (f.state === 'connecting') return `<h2>Connecting forge ${esc(f.n)} as its admin.</h2><p>Making its bot, then ${esc(s.org)}/${esc(s.repo)}.</p>`;
    if (f.state === 'connected') return `<h2>Connected as ${esc(f.login)} (id ${esc(f.id)}), managed.</h2><p>Made ${esc(s.org)}/${esc(s.repo)}, and its project ${esc(s.project)}. Its CI runs on your forge’s runners labelled ${esc(s.label)}; with none, a change’s checks wait.</p><div class="button-row"><a class="button primary" href="${esc(u(`chats.html?project=${s.project}`))}">Open ${esc(s.project)}</a><a class="text-link" href="#" data-later="Mockup: the repository on Forgejo">Open ${esc(s.org)}/${esc(s.repo)} on Forgejo ↗</a></div>`;
    const stops = {
      'not-admin': 'The token you gave is not a site admin’s. Paste one a site admin made.',
      '401': 'Forgejo refused the token. Check that it was copied whole, and that it hasn’t been deleted.',
      '403-bot': 'The token lacks a scope: making the bot was refused. Make one with write:admin, read:user, write:organization and write:repository.',
      '403-org': 'The token lacks a scope: making the organization was refused. Make one with write:admin, read:user, write:organization and write:repository.',
      unmarked: 'A user temper didn’t make holds the bot’s login, without temper’s mark: add temper:bot to its full name on Forgejo, or change the bot’s login in the configuration.',
      other: 'The connection stopped: Forgejo answered 500.',
    };
    const stages = { admin: "checking the admin's token", org: 'its organization', repository: 'its repository', file: 'its starter workflow' };
    const stop = stopped?.startsWith('first-') ? `temper made its bot, but making ${s.org}/${s.repo} stopped at ${stages[stopped.slice(6)]}: Forgejo answered 500. Paste the token again to finish it.` : stops[stopped];
    const field = (label, value, type = 'text') => {
      const refusal = type === 'password' && stopped === 'required' || label === 'Organization' && stopped === 'org-required' || label === 'Repository' && stopped === 'repository-required' ? 'Required.' : type === 'password' && stopped === 'too-long' ? 'The pasted values are past 4 KiB, the most a request carries.' : label === 'Runner label' && stopped === 'label' ? 'Letters, digits, dots, dashes and underscores, at most 64.' : '';
      // An invalid label is kept as written; a required field was left empty.
      const shown = refusal && label !== 'Runner label' ? '' : label === 'Runner label' && stopped === 'label' ? 'docker!' : value;
      return `<label class="forge-field"><span>${label}</span><input type="${type}" value="${esc(shown)}"${refusal ? ' aria-invalid="true"' : ''}>${refusal ? `<span class="forge-failure">${refusal}</span>` : ''}</label>`;
    };
    return `${stop ? `<p class="forge-failure">${esc(stop)}</p><p class="muted tiny">temper kept nothing of the token.</p>` : ''}<h2>Connect as its admin</h2><p>Paste a site admin’s token with the scopes write:admin, read:user, write:organization and write:repository. temper keeps it, and uses it only to make its bot and your first repository, and to give the bot access to it. To stop temper using it, delete the token on Forgejo.</p>${field('The site admin’s token', '', 'password')}<fieldset><legend>Your first repository</legend>${field('Organization', s.org)}${field('Repository', s.repo)}${field('Into a new project, named', s.project)}${field('Runner label', s.label)}<p class="muted tiny">Its CI runs on your forge’s runners with this label. Forgejo’s runner registers as docker unless told otherwise.</p></fieldset><button class="button primary" data-later="Mockup: send the token once, then Connecting">Connect as its admin</button>`;
  }
  function flowCard(f, from = 'connect-forge') {
    if (f.admin) return adminCard(f);
    if (f.kind === 'GitHub') return f.state === 'connected' ? githubReady(f, from) : f.state === 'connecting' ? githubConnecting(f, from) : githubAppForm(f);
    if (f.state === 'connected') return `<h3>${fact('temper: the repository made at set-up', esc(`${f.setup.org}/${f.setup.repo} is ready`))}</h3><p>In the new project ${fact('jig’s client/domain/contract.md, 3.2: MakeProject’s ProjectMade, and Row.repositories[].project', esc(f.setup.project))}, on ${esc(forgeName(f))}.${future('ci', ' Its first CI run passed, on this machine.')}</p>${ownedProgress(f, { allDone: true })}<div class="button-row"><a class="button primary" href="${esc(u(`chats.html?project=${encodeURIComponent(f.setup.project)}`))}">Open ${esc(f.setup.project)}</a><a class="text-link" href="#" data-later="${esc(`Mockup: Forgejo opens in a new tab at its /user/oauth2/temper, and signs ${f.person} in through temper.`)}">Open Forgejo ↗</a></div><p class="muted tiny">${conf('the forge’s name', esc(forgeName(f)))} is on the <a class="text-link" href="${esc(u('forges.html'))}">Forges</a> page, with its details.</p>`;
    if (['setting up', 'set up failed'].includes(f.state)) return `<h3>Setting up Forgejo on this machine</h3>${chosenValues(f)}${ownedProgress(f, { failedAt: f.state === 'set up failed' ? 'preparing' : null })}${f.state === 'set up failed' ? `${ownedFailure()}<button class="button" data-later="Mockup: the box runs the steps again; a fetched binary is kept.">Set up again</button>` : '<p class="muted tiny">You may leave this page: the steps go on, and the banner says where they are.</p>'}`;
    return f.box ? ownedForm(f) : ownedCommands(f);
  }
  const flowCurrent = f => !f ? 1 : f.state === 'connected' ? 4 : ['setting up', 'connecting', 'set up failed'].includes(f.state) ? 2 : 1;
  const flowWrap = content => `<section class="card card-pad forge-flow">${content}</section>`;
  function forgeChoices(target = 'connect-forge.html?forge=') {
    const github = world.forges.find(f => f.kind === 'GitHub' && f.state === 'not connected');
    const offer = world.forges.find(f => f.mode === 'owned' && f.state === 'not set up');
    const admin = world.forges.filter(f => f.admin && f.state === 'not connected').map(f => `<section class="card card-pad"><h3>Forgejo at ${esc(f.web || `forge ${f.n}`)}</h3><p>temper connects as its site admin, makes its bot and your first repository.</p><a class="button" href="${esc(u(`${target}${f.n}`))}">Connect as its admin</a></section>`).join('');
    return `<div class="forge-choices">${github ? `<section class="card card-pad"><h3>GitHub</h3><p>Your repositories on GitHub. temper makes its own App there; you choose the account and the repositories it may use.</p><p class="muted tiny">In this tab, on GitHub · about two minutes</p><div class="button-row"><a class="button primary" href="${esc(toGithub('create', target.startsWith('welcome') ? 'welcome' : 'connect-forge', '2'))}">Connect GitHub</a></div></section>` : ''}${offer ? (target.startsWith('welcome') ? `<section class="card card-pad"><h3>Forgejo</h3><p>A forge of your own. temper can start it and manage it, or you can connect your own deployment.</p><p class="muted tiny">${offer.box ? 'Here · about a minute' : 'Needs four commands and a restart'}</p><div class="button-row"><a class="button" href="${esc(u(`${target}${offer.n}`))}">Run on this machine</a><a class="text-link" href="https://github.com/fabrig-ai/temper/blob/main/docs/help/server.md">Or connect your deployment</a></div></section>` : `<section class="card card-pad"><h3>Forgejo on this machine</h3><p>A forge of your own, run by temper box. No account needed: temper makes your first repository, and you start in it.</p><p class="muted tiny">${offer.box ? 'Here · about a minute' : 'Needs four commands and a restart'}</p><a class="button" href="${esc(u(`${target}${offer.n}`))}">Choose</a></section>`) : ''}${admin}${target.startsWith('welcome') ? '' : `<section class="card card-pad"><h3>A Forgejo elsewhere</h3><p>Your own Forgejo server. temper connects as its admin, or with its bot’s tokens.</p><p class="muted tiny">Needs a change to temper’s configuration and a restart</p><a class="button" href="${esc(u(`${target}elsewhere`))}">Show how</a></section>`}</div>`;
  }

  // The welcome (client README, 3.5): over, its link opens Chats in its
  // place; otherwise it shows Welcome, and its other steps only at their
  // addresses, `?at=`. The demo starts only across its link.
  const SIGN_IN_ENDS = {
    declined: () => ['You signed in without plan usage.', true],
    held: () => ['Another person here holds that ChatGPT account. To use one of yours, sign out of ChatGPT in your browser first, then Continue with ChatGPT.'],
    misconfigured: () => ['OpenAI refused temper’s sign-in. Check the configuration’s registration.'],
    'port-taken': () => [variant('port', 'Try later', 'temper can’t listen on 127.0.0.1:1455: another program has it, such as Codex’s own sign-in. Finish or close that sign-in, then Continue with ChatGPT.')],
  };
  function signInEnd(end) {
    const again = '<button class="button" data-later="Mockup: a fresh sign-in, in this tab; ChatGPT sends it back here.">Continue with ChatGPT</button>';
    if (end === 'not-eligible') return future('eligible', `<div class="welcome-end"><p class="forge-failure">This ChatGPT account can’t be used for plan usage in temper. Plan usage needs an eligible ChatGPT plan.</p><div class="button-row"><a class="button" href="${esc(u('welcome.html?at=model&way=key'))}">Use an API key</a><button class="button" data-later="Mockup: a fresh sign-in, after signing out of ChatGPT in the browser.">Sign in to another account</button></div></div>`);
    const [words, key] = SIGN_IN_ENDS[end]();
    return `<div class="welcome-end"><p class="forge-failure" role="alert">${fact('jig’s client/domain/contract.md, 7.1: OwnerAccounts.last_end, its Refused or Failed', words)}</p><div class="button-row">${again}${key ? `<a class="button" href="${esc(u('welcome.html?at=model&way=key'))}">Use an API key</a>` : ''}</div></div>`;
  }
  pages.welcome = () => {
    document.body.classList.add('terminal-pages');
    // The person's forge in play: the one acted on, else the owned offer.
    const f = world.forges.find(f => f.mode !== 'fake' && !['not connected', 'not set up', 'retired'].includes(f.state)) || world.forges.find(f => f.mode === 'owned');
    const startedForge = f && !['not connected', 'not set up'].includes(f.state);
    const adopted = startedForge ? Object.values(world.repos).find(r => r.forge === f.n && !r.retired) : null;
    const done = startedForge && f.state === 'connected' && !!adopted;
    // GitHub's way stays at Forge until a repository is adopted from it.
    const githubOpen = startedForge && f.kind === 'GitHub' && !adopted;
    const over = demoRunning() || (usableModel() && ownForge());
    const at = params.get('at') || (over ? 'over' : 'welcome');
    const views = ['welcome', 'forge', 'model', 'ready'];
    const current = views.includes(at) ? at : 'welcome';
    const way = params.get('way');
    const back = (current === 'forge' && params.has('forge')) || (current === 'model' && way) ? current : views[Math.max(0, views.indexOf(current) - 1)];
    const foot = `<div class="welcome-foot"><a class="text-link" href="${esc(u(`welcome.html?at=${back}`))}">‹ Back</a><a class="text-link" href="${esc(u('chats.html'))}">Later</a>${!ownForge() ? tryDemo('Try the playground instead') : ''}</div>`;
    const eachStep = () => f.mode === 'owned' ? `<details class="welcome-progress"><summary>Each step</summary>${ownedProgress(f)}</details>` : '';
    // The first repository: the adopted one, else the set up's, once asked.
    const repo = adopted ? adopted.name : startedForge && f.setup ? `${f.setup.org}/${f.setup.repo}` : null;
    const proj = adopted ? adopted.project : startedForge && f.setup ? f.setup.project : null;
    let content;
    if (at === 'over') {
      content = `<p class="mock-note">${demoRunning() ? 'The playground runs' : 'A model is usable, and a forge of Sam’s is connected'}: the link opens Chats, its address in the welcome’s place, so that back doesn’t return here. <a class="text-link" href="${esc(u('chats.html'))}">Open Chats</a></p>`;
    } else if (current === 'welcome') {
      content = `<h1>Welcome to temper</h1><p class="welcome-tagline">Reliable, scalable and flexible software factory.</p><p class="welcome-lead">You describe <strong>what</strong> you want and <strong>how</strong> you want it done, and it happens.</p>
        <div class="welcome-choices"><section class="card card-pad"><h2>Try the playground</h2><p>A built-in scripted model and forge. Get a taste of how temper feels.</p><p class="muted">Fully offline · Ready at once</p><a class="button primary" href="${esc(u('chats.html?world=first-run&step=1'))}">Try the playground</a></section>
        <section class="card card-pad"><h2>Set up temper</h2><p>Your forge: GitHub or Forgejo<br>Your model: ChatGPT or Claude</p><p class="muted">About five minutes</p><a class="button primary" href="${esc(u('welcome.html?at=forge'))}">Set up temper</a></section></div>
        <p class="muted welcome-after">Started in the playground? Connect your own forge and model whenever you like; the playground then steps aside.</p>`;
    } else if (current === 'forge') {
      const which = params.get('forge') || (githubOpen ? String(f.n) : null);
      const chosen = world.forges.find(f => String(f.n) === which && f.mode !== 'fake');
      const next = Math.max(...world.forges.map(f => f.n)) + 1;
      const details = which === 'elsewhere' ? elsewhereHow(next) : (chosen?.kind === 'GitHub' || chosen?.admin) ? flowCard(chosen, 'welcome') : chosen ? ownedForm(chosen, 'welcome.html?at=model&world=first-run-real&step=2') : '';
      content = `<h2>Connect a forge</h2><p class="muted welcome-lead">Where your code lives: you file issues, agents push and land code.</p>${which ? flowWrap(details) : forgeChoices('welcome.html?at=forge&forge=')}${foot}`;
    } else if (current === 'model') {
      // The forge's line, so that Model says what Forge did: under way,
      // with its steps, or done, ticked, with its first repository.
      const into = f?.setup ? `${repo}, ${done ? 'in' : 'into'} ${f.setup.newProject ? `the new project ${proj}` : proj}` : '';
      const words = [forgeName(f), done ? '' : f?.kind === 'GitHub' && f.state === 'connected' ? 'no repository chosen yet' : f?.state, f?.kind === 'GitHub' && !done ? '' : into].filter(Boolean).join(' · ');
      const status = startedForge ? `<div class="welcome-status">${fact('client README, 3.5, The welcome: the forges watch’s Row.{connection, first, repositories}, or the set up’s request', `${done ? '<span class="ok">✓</span> ' : ''}${esc(words)}`)}${done ? '' : eachStep()}</div>` : '';
      // Connect a model's Choose and Details, and its sign-in, within the
      // wizard; a sign-in in this tab comes back to Ready.
      const signedIn = worldKey === GH_WORLD ? 'welcome.html?at=ready&step=6' : 'welcome.html?at=ready&world=first-run-real&step=3';
      const allWays = `<p><a class="text-link" href="${esc(u('welcome.html?at=model'))}">‹ All ways</a></p>`;
      const paste = way === 'chatgpt' && params.get('paste');
      const a = ownAccounts().find(a => a.provider === 'OpenAI' && isPlan(a));
      const body = way === 'key' ? `${allWays}${flowWrap(keyForm())}`
        : paste === 'done' ? `${allWays}${flowWrap(chatgptReady(a || chatgptLike({ source: {} }), false))}`
        : paste ? `${allWays}${flowWrap(signInCard(paste, params.get('elsewhere') === '1'))}`
        : modelChoices(signedIn, 'welcome.html?at=model&way=key');
      content = `${status}<h2>Connect a model</h2><p class="muted welcome-lead">temper’s agents run on it: on a plan you sign in to, or on a key.</p>${body}${foot}`;
    } else {
      const owned = f.mode === 'owned';
      const next = world.config.next;
      const a = next ? account(next.account) : ownAccounts().find(a => a.sources.length);
      const end = !usableModel() && params.get('end');
      const modelLine = a
        ? `<li class="done">Model: ${accountHeading(a)}${future('preview', esc(` · ${next?.model || a.models?.[0] || ''}`))}</li>`
        : `<li>Model: none yet · <a class="text-link" href="${esc(u('welcome.html?at=model'))}">Connect a model</a>${end ? signInEnd(end) : ''}</li>`;
      const forgeLine = !startedForge ? `<li>Forge: none yet · <a class="text-link" href="${esc(u('welcome.html?at=forge'))}">Connect a forge</a></li>`
        : `<li class="${done ? 'done' : 'current'}">${fact('client forge.md, 5.4: the forges watch’s Row.connection, and Connected.login', esc(f.state === 'connected' ? `Forge: ${forgeName(f)}, connected as ${f.login}` : `Forge: ${forgeName(f)} · ${f.state}`))}${!done && f.state !== 'connected' ? eachStep() : ''}</li>`;
      const projectLine = repo ? `<li class="${done ? 'done' : ''}">${fact('client README, 3.5, The welcome: Row.first’s org and repository, and its project; Row.repositories once adopted', esc(`Project ${proj}, with ${repo}`))}${done && owned ? future('ci', ' · its first CI run passed') : ''}</li>` : '';
      // Connected with no first repository: adopt one (client README, 4.4).
      const acts = done ? `<div class="button-row"><a class="button primary" href="${esc(u(`chats.html?project=${encodeURIComponent(proj)}`))}">Open ${esc(proj)}</a>${owned ? `<a class="text-link" href="#" data-later="${esc(`Mockup: Forgejo opens in a new tab at its /user/oauth2/temper, and signs ${f.person} in through temper.`)}">Open Forgejo ↗</a>` : f.kind === 'GitHub' ? `<a class="text-link" href="#" data-later="${esc(`Mockup: github.com/${repo}, in a new tab.`)}">Open on GitHub ↗</a>` : `<a class="text-link" href="#" data-later="Mockup: the repository on Forgejo">Open ${esc(repo)} on Forgejo ↗</a>`}</div>`
        : repo ? `<button class="button primary" disabled>Open ${esc(proj)}</button><p class="muted tiny welcome-after">It opens once ${esc(repo)} is made.</p>`
        : startedForge && f.state === 'connected' ? '<a class="button primary" href="#" data-later="Mockup: the adoption form, into a new project named after the repository chosen (client README, 4.4).">Adopt a repository</a>' : '';
      content = `<h2>${done ? esc(`${repo} is ready`) : 'Almost ready'}</h2><ul class="forge-progress welcome-checklist">${done ? forgeLine + modelLine : modelLine + forgeLine}${projectLine}</ul>${acts}`;
    }
    // Ready is done, not current, once its forge and model both are.
    const step = current === 'ready' && done && usableModel() ? views.length : views.indexOf(current);
    return `${topbar(null, { bare: true })}<main class="welcome-page">${at === 'over' ? '' : flowSteps(step, '', ['Welcome', 'Forge', 'Model', 'Ready'])}${content}</main>`;
  };

  pages['connect-forge'] = () => {
    const which = params.get('forge');
    const f = world.forges.find(f => String(f.n) === which && f.mode !== 'fake');
    const next = Math.max(...world.forges.map(f => f.n)) + 1;
    const owned = world.forges.find(f => f.n === 3);
    const choose = forgeChoices();
    const github = world.forges.find(f => f.kind === 'GitHub');
    const variants = worldKey === 'first-run' ? `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: the flow’s other paths. GitHub’s, with GitHub’s own pages, are drawn in their own world: <a class="text-link" href="${esc(u(`connect-forge.html?world=${GH_WORLD}#other-states`, '1'))}">First run, on GitHub</a>.</p>${[
      ['Forgejo on this machine, set up failed', { ...owned, state: 'set up failed' }],
      ['Forgejo on this machine, without the box', { ...owned, state: 'not set up', box: false }],
      ['A Forgejo elsewhere', null],
    ].map(([heading, f]) => `<h3>${esc(heading)}</h3>${flowSteps(flowCurrent(f))}${flowWrap(f ? flowCard(f) : elsewhereHow(next))}`).join('')}</section>`
      : worldKey === GH_WORLD ? `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: where GitHub’s way can’t go on, temper’s listener answers its own short page, in place of sending the tab back here; and the App of your own’s Details.</p>${[
      ['Back from GitHub with a request, not an installation: the setup route’s page', githubRefusal('request')],
      ['A link that has ended: the manifest’s, the redirect’s or the setup route’s page', githubRefusal('ended')],
      ['An App of your own: its details', githubAppForm(github)],
    ].map(([heading, card]) => `<h3>${esc(heading)}</h3>${flowWrap(card)}`).join('')}</section>` : '';
    const steps = f?.kind === 'GitHub' ? githubSteps(f) : f?.admin ? flowSteps(f.state === 'connected' ? 3 : f.state === 'connecting' ? 2 : 1, 'Connecting') : flowSteps(which ? flowCurrent(f) : 0);
    const atDetails = f?.kind === 'GitHub' ? f.state === 'not connected' : flowCurrent(f) === 1;
    return listPage('forges', [['Forges', 'forges.html'], ['Connect a forge']], `<div class="section-heading" style="margin-top:0"><h2>Connect a forge</h2></div>${steps}${which ? `${atDetails ? `<p><a class="text-link" href="${esc(u('connect-forge.html'))}">‹ All forges</a></p>` : ''}${flowWrap(f ? flowCard(f) : elsewhereHow(next))}` : choose}${variants}`);
  };
  // ---- GitHub's side --------------------------------------------------

  // GitHub's pages on Connect GitHub's way, sketched: not temper's, drawn
  // so that the way can be followed click by click, in the world
  // first-run-github. Each says, in slate, the address the tab is at and
  // how it got there; `from` is the page of temper's the tab returns to.
  pages.github = () => {
    const gh = world.github;
    if (!gh) return `<div class="mock-index"><p>GitHub’s side is drawn in the world <a href="${esc(u(`github.html?world=${GH_WORLD}`, '2'))}">First run, on GitHub</a>.</p></div>`;
    const from = params.get('from') || 'connect-forge';
    const f = world.forges.find(f => f.kind === 'GitHub');
    const install = f.app?.installs[0];
    const asked = params.get('at') || ({ 1: 'create', 2: 'create', 3: 'install' }[stepId] || 'installation');
    const at = asked === 'installation' && !install ? 'install' : asked;
    const where = (address, how) => `<p class="mock-url"><b>${esc(address)}</b>${how ? `<br>${how}` : ''}</p>`;
    const left = (words, target, step) => `<p class="mock-note gh-left">${words} <a class="text-link" href="${esc(u(target, step))}">temper, as it is then</a></p>`;
    if (at === 'refused') {
      document.body.classList.add('terminal-pages');
      const why = params.get('why') === 'ended' ? 'ended' : 'request';
      const how = why === 'request' ? 'GitHub sent the tab here after Request, with no installation: an owner of the organization is to approve it.' : 'The state in the address is used, expired or from before a restart. The manifest’s and the redirect’s addresses answer the same.';
      const route = why === 'request' ? 'setup?setup_action=request&state=Qm9v…' : 'setup?installation_id=…&state=Qm9v…';
      return `<main class="welcome-page">${where(`http://127.0.0.1:8367/forges/${f.n}/github/${route}`, how)}<p class="route-brand">temper</p>${flowWrap(githubRefusal(why, from))}</main>`;
    }
    const accountOf = login => gh.accounts.find(a => a.login === login) || gh.accounts[0];
    const avatar = a => `<span class="gh-avatar${a.kind === 'org' ? ' gh-org' : ''}" aria-hidden="true">${esc(a.login[0].toUpperCase())}</span>`;
    const appHead = (title, sub) => `<div class="gh-app-head"><span class="gh-avatar gh-app" aria-hidden="true">t</span><div><h1 class="gh-title">${title}</h1><p class="gh-muted">${sub}</p></div></div>`;
    const perms = `<p class="gh-sub">with these permissions:</p><ul class="gh-perms"><li><b>Read</b> access to actions, administration, checks, commit statuses and metadata</li><li><b>Read</b> and <b>write</b> access to code, issues, merge queues and pull requests</li></ul>`;
    // GitHub's choice of repositories: all, its default, or those selected.
    const access = (a, chosen) => `<div class="gh-access" data-gh-account="${esc(a.login)}">
        <label class="gh-radio"><input type="radio" name="gh-scope" value="all"${chosen.all ? ' checked' : ''}><span><b>All repositories</b><span class="gh-muted">This applies to all current and future repositories owned by the resource owner.</span></span></label>
        <label class="gh-radio"><input type="radio" name="gh-scope" value="select"${chosen.all ? '' : ' checked'}><span><b>Only select repositories</b><span class="gh-muted">Select at least one repository.</span></span></label>
        <div class="gh-select"${chosen.all ? ' hidden' : ''}>${gh.repos[a.login].map(([n, priv]) => `<label class="gh-repo"><input type="checkbox" value="${esc(n)}"${!chosen.all && chosen.names?.includes(n) ? ' checked' : ''}><span>${esc(`${a.login}/${n}`)}</span>${priv ? '<span class="gh-pill">Private</span>' : ''}</label>`).join('')}</div>
      </div>`;
    const choose = from === 'welcome' ? 'welcome.html?at=forge' : from === 'forges' ? 'forges.html' : 'connect-forge.html';
    let body;
    if (at === 'create') {
      body = `${where('https://github.com/settings/apps/new?state=k3Jx…', `From temper’s manifest page, ${conf('the manifest page’s address, on the clients’ listener', esc(`http://127.0.0.1:8367/forges/${f.n}/github/manifest?state=k3Jx…`))}, which posted temper’s manifest here as it loaded. Connect GitHub sent the tab there.`)}
        <h1 class="gh-title">Create GitHub App</h1>
        <div class="gh-box gh-pad"><label class="gh-label" for="gh-name">GitHub App name</label><input id="gh-name" class="gh-input" value="${esc(gh.app)}"><p class="gh-muted">The name of your GitHub App.</p>
        <a class="gh-btn gh-btn-primary" href="${esc(u(`github.html?at=install&from=${from}`, '3'))}">Create GitHub App for ${esc(gh.login)}</a></div>
        ${left('Back by the browser’s back instead: no App is made, temper shows no connect under way, and Connect GitHub starts afresh.', choose, '1')}`;
    } else if (at === 'install') {
      const rows = gh.accounts.map(a => {
        const on = f.app?.installs.some(i => i.account === a.login);
        const [label, target] = on ? ['Configure', `github.html?at=installation&from=${from}`] : [a.role === 'owner' ? 'Install' : 'Request', `github.html?at=account&account=${a.login}&from=${from}`];
        const note = a.kind === 'user' ? 'your personal account' : a.role === 'owner' ? 'an organization you own' : 'an organization you’re a member of';
        return `<div class="gh-row">${avatar(a)}<span class="gh-grow"><b>${esc(a.login)}</b> <span class="gh-muted">· ${note}</span></span><a class="gh-btn" href="${esc(u(target))}">${label}</a></div>`;
      }).join('');
      body = `${where(`https://github.com/apps/${gh.app}/installations/new?state=Qm9v…`, install ? 'From temper’s “Choose its repositories on GitHub”: a fresh install link, whose state GitHub keeps through Configure.' : `From temper’s redirect, http://127.0.0.1:8367/forges/${f.n}/github/redirect?code=…&state=k3Jx…, which exchanged GitHub’s code for the App’s key, kept it, and sent the tab here.`)}
        ${appHead(`Install ${esc(gh.app)}`, `Where do you want to install ${esc(gh.app)}?`)}
        <div class="gh-box gh-rows">${rows}</div>
        ${install ? '' : left('Back to temper from here instead: the App is made and installed nowhere, and temper offers Install on GitHub.', githubBack(from), '3')}`;
    } else if (at === 'account') {
      const a = accountOf(params.get('account'));
      const owner = a.role === 'owner';
      const go = owner
        ? `<a class="gh-btn gh-btn-primary" data-gh-go="${esc(u(githubBack(from), '4'))}" data-gh-grant href="#">Install</a>`
        : `<a class="gh-btn gh-btn-primary" href="${esc(u(`github.html?at=refused&why=request&from=${from}`, '3'))}">Request</a>`;
      body = `${where(`https://github.com/apps/${gh.app}/installations/new/permissions?target_id=…&state=Qm9v…`, '')}
        ${appHead(`${owner ? 'Install' : 'Request'} ${esc(gh.app)}`, a.kind === 'user' ? `Install on your personal account ${esc(a.login)}` : owner ? `Install on your organization ${esc(a.login)}` : `Request on the organization ${esc(a.login)}. An owner of ${esc(a.login)} must approve the request.`)}
        <div class="gh-box gh-pad"><p class="gh-sub">for these repositories:</p>${access(a, { all: true })}${perms}<div class="gh-acts">${go}<a class="gh-btn" href="${esc(u(`github.html?at=install&from=${from}`))}">Cancel</a></div></div>
        ${owner ? '<p class="mock-note gh-left">Install sends the tab to temper’s setup route, which takes the installation and sends the tab back where Connect GitHub started.</p>' : '<p class="mock-note gh-left">Request sends the tab to temper’s setup route with no installation: temper answers its own page.</p>'}`;
    } else {
      const a = accountOf(install.account);
      const direct = from === 'forges';
      body = `${where(`https://github.com/settings/installations/${install.id}`, direct ? 'From the installation’s link on temper’s Forges page.' : 'From Configure, on the install page.')}
        ${appHead(esc(gh.app), `Installed on ${esc(a.login)} · developed by ${esc(gh.login)}`)}
        <div class="gh-box gh-pad"><h2 class="gh-h2">Repository access</h2>${access(a, install)}${perms}<div class="gh-acts"><a class="gh-btn gh-btn-primary" data-gh-go="${esc(u(githubBack(from), stepId))}" data-gh-grant href="#">Save</a></div></div>
        <p class="mock-note gh-left">temper’s App asks GitHub to send the tab to its setup route on an update too: Save sends it there, and temper reads the installation’s repositories again. ${direct ? 'This visit carries no state of temper’s, so the route sends the tab to Forges.' : 'The state GitHub kept from the install link sends the tab back where it started; were GitHub to drop it, the route would send the tab to Forges.'}</p>`;
    }
    return `<div class="gh"><header class="gh-header"><span class="gh-mark" aria-hidden="true"></span><b>GitHub</b><span class="gh-search">Type / to search</span>${avatar(accountOf(gh.login))}</header><main class="gh-main">${body}</main></div>`;
  };
  // GitHub's choice of repositories, carried in the address of the act
  // that ends it, as `granted`.
  function bindGithub() {
    const box = document.querySelector('.gh-access');
    const go = document.querySelector('[data-gh-go]');
    if (!box || !go) return;
    const update = () => {
      const all = box.querySelector('input[value="all"]').checked;
      box.querySelector('.gh-select').hidden = all;
      const names = [...box.querySelectorAll('.gh-select input:checked')].map(i => i.value);
      const url = new URL(go.dataset.ghGo, location.href);
      url.searchParams.set('granted', `${box.dataset.ghAccount}:${all ? 'all' : names.join(',')}`);
      go.href = url.href;
      go.setAttribute('aria-disabled', String(!all && !names.length));
    };
    box.addEventListener('change', update);
    update();
  }
  // Ready's choice of a repository, carried as `pick`.
  function bindPick() {
    const list = document.querySelector('[data-pick]');
    if (!list) return;
    const go = document.querySelector('[data-pick-go]');
    const name = document.querySelector('[data-pick-project]');
    list.addEventListener('change', () => {
      const pick = list.querySelector('input:checked').value;
      name.value = pick;
      const url = new URL(list.dataset.pick, location.href);
      url.searchParams.set('pick', pick);
      go.href = url.href;
      go.textContent = `Adopt ${list.dataset.owner}/${pick}`;
    });
  }

  pages.forges = () => {
    const owned = world.forges.find(f => f.n === 3);
    const forges = world.forges.filter(f => f.mode !== 'fake' && ['connecting', 'setting up', 'set up failed', 'connected', 'stopped'].includes(f.state));
    const fake = demoRunning() ? world.forges.find(f => f.mode === 'fake' && f.state !== 'retired') : null;
    const variants = worldKey === 'first-run' ? `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: forge 3’s rows in its other states, and a Forgejo of each other mode, connected, to compare their rows.</p>${[
      ['Set up failed', { ...owned, state: 'set up failed', load: null }],
      ['Stopped', { ...owned, state: 'stopped', login: 'temper', id: 2, load: null }],
      ['Managed, for comparison', { n: 4, kind: 'Forgejo', name: 'Forgejo at forge.example', web: 'https://forge.example', mode: 'managed', state: 'connected', login: 'temper', id: 7 }],
      ['Connected, for comparison', { n: 5, kind: 'Forgejo', name: 'Forgejo at git.example', web: 'https://git.example', mode: 'connected', state: 'connected', login: 'ci-bot', id: 31 }],
    ].map(([heading, f]) => `<h3>${esc(heading)}</h3>${forgeRow(f, true)}`).join('')}</section>` : '';
    return listPage('forges', [['Forges']], `<div class="section-heading" style="margin-top:0"><div><h2>Your forges <span class="subtle-count">${fact('temper: the person’s forges, demo excluded', esc(forges.length))}</span></h2><p class="muted tiny">Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.</p></div><a class="button" href="${esc(u('connect-forge.html'))}">+ Add a forge</a></div><div class="stack">${forges.length ? forges.map(f => forgeRow(f)).join('') : `<div class="empty-state">No forge of yours yet. <a class="button primary" href="${esc(u('connect-forge.html'))}">Connect a forge</a></div>`}</div>${fake ? `<div class="section-heading"><h2>The playground</h2></div><details class="card forge-row forge-demo"><summary><span>${fact('temper: the demo’s forge and repository', 'Playground forge · the playground’s, until a forge of yours connects · playground/widget')}</span></summary><div class="card-pad">${forgeBody(fake)}</div></details>` : ''}${worldKey === 'first-run' && demo?.state === 'retired' ? `<p class="gap-note">${ahead('playground’s end', 'the playground ending once a forge of the person’s connects: no mission yet')} The playground’s forge stopped when forge 3 connected; it no longer shows.</p>` : ''}${variants}`);
  };

  // ---- Signing in -----------------------------------------------------

  // The stock sign-in page, the same for every deployment whose people
  // sign in with Google, in the world google-sign-in (draft sign-in.md,
  // sections 4 to 6): the client's signed-out main in the browser, the
  // client's Authorize place for Forgejo, and the pages Google's return
  // route answers at each ending. `?state=` names an ending,
  // `?from=forgejo` an authorization of Forgejo's. Each says, in slate,
  // the address the tab is at and how it got there.
  const SIGN_IN_WORLD = 'google-sign-in';
  // Google's button as its branding guidelines draw it: its mark, its words.
  const googleMark = '<svg class="google-mark" viewBox="0 0 48 48" aria-hidden="true"><path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"/><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"/><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"/><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"/></svg>';
  const googleButton = (label, note) => `<a class="google-button" href="#" data-later="${esc(note)}">${googleMark}<span>${esc(label)}</span></a>`;
  const SIGN_IN_STATES = [
    ['signed-out', 'Signed out, by Sign out: Forgejo keeps its own sign-in'],
    ['not-member', 'Back from Google: an address that is no member’s'],
    ['other-account', 'Back from Google: a member’s address, from another Google account than the one bound to it'],
    ['unverified', 'Back from Google: an address Google has not verified'],
    ['cancelled', 'Back from Google: cancelled there'],
    ['expired', 'Back from Google: the state unknown, used, past its deadline, or from before a restart'],
    ['busy', 'Back from Google: jig’s Mint refused, not ready or busy'],
    ['unknown-client', 'An authorization whose redirect URI is not Forgejo’s registered one'],
  ];
  pages['sign-in'] = () => {
    document.body.classList.add('terminal-pages');
    const site = world.site;
    if (!site) return `<div class="mock-index"><p>The sign-in page is drawn in the world <a href="${esc(u(`sign-in.html?world=${SIGN_IN_WORLD}`, '1'))}">Google sign-in</a>.</p></div>`;
    const forge = forgeOf(site.forge);
    const forgeHost = new URL(forge.web).host;
    const forgeName = conf('the forge’s name', esc(forge.name));
    const state = params.get('state');
    const fromForgejo = params.get('from') === 'forgejo';
    const where = (address, how) => `<p class="mock-url"><b>${esc(address)}</b>${how ? `<br>${how}` : ''}</p>`;
    const googleMarkNote = ahead('Google', 'signing in with Google, through jig’s Mint: draft sign-in.md, section 4');
    const providerNote = ahead('provider', 'temper as Forgejo’s OpenID provider: draft sign-in.md, section 5');
    const brand = '<p class="sign-in-brand"><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 1.5c1.5 5.6 2.9 7 8.5 8.5-5.6 1.5-7 2.9-8.5 8.5C10.5 12.9 9.1 11.5 3.5 10 9.1 8.5 10.5 7.1 12 1.5Z"/><circle cx="18.5" cy="18.5" r="2" fill="currentColor"/></svg><span>temper</span></p>';
    const page = (address, how, body, after = '') => `<main class="sign-in-page">${where(address, how)}<section class="card sign-in-card">${brand}${body}</section>${after}</main>`;
    const authorize = `${site.origin}/oauth/authorize?client_id=forgejo&redirect_uri=https%3A%2F%2F${forgeHost}%2Fuser%2Foauth2%2Ftemper%2Fcallback&response_type=code&scope=openid+email+profile&state=Zk9x…&nonce=…`;
    const returned = query => `${site.origin}/sign-in/google/return?${query}`;
    const start = then => `Mockup: GET /sign-in/google?then=${then} draws the sign-in’s state, binds it to this browser by a cookie, and sends the tab to Google; Google sends it back to /sign-in/google/return.`;
    const chooser = 'Mockup: a fresh sign-in, with Google’s account chooser, back to /sign-in/google/return.';
    const ways = (then, label = 'Continue with Google', note = start(then)) => `<div class="sign-in-ways">${googleButton(label, note)}${googleMarkNote}</div>`;
    const back = `<a class="text-link" href="${esc(u('sign-in.html', '1'))}">Back to sign in</a>`;
    const member = fact('Google: the ID token’s verified address', esc(site.member));
    const stranger = fact('Google: the ID token’s address', esc(site.stranger));
    const front = (under, then) => `<h1>Sign in</h1>${under}${ways(then)}`;
    const variants = bare ? '' : `<section class="mock-variants" id="other-states"><h2>Other states, for reference</h2><p>Not steps of this world: the front page after Sign out (draft sign-in.md, section 6), each page Google’s return route answers instead of signing in (4.3), and Forgejo’s authorization refused (5.3).</p><ul class="sign-in-variants">${SIGN_IN_STATES.map(([s, what]) => `<li><a href="${esc(u(`sign-in.html?state=${s}`))}">${esc(what)}</a></li>`).join('')}</ul></section>`;

    if (fromForgejo && world.signedIn) {
      return page(authorize, `From Forgejo’s “Sign in with temper”, or its route /user/oauth2/temper, which Open Forgejo ↗ opens. temper’s page, read by the client as its Authorize place: signed in, it sends jig’s Authorize, and replaces its address with Forgejo’s callback and the code. ${providerNote}`,
        `<h1>Continuing to Forgejo…</h1><p class="muted">Signed in as ${fact('jig: the party signed in', esc(world.person))} · ${member}. ${forgeName} opens in a moment, signed in as ${conf('the member’s login', esc(site.login))}.</p><p class="sign-in-acts"><a class="text-link" href="#" data-later="${esc(`Mockup: ${forge.web}, signed in as ${site.login}: Forgejo’s page, not drawn.`)}">Forgejo, a blink later ↗</a></p>`);
    }
    if (fromForgejo) {
      return page(authorize, `From Forgejo’s “Sign in with temper”, or its route /user/oauth2/temper. temper’s page, read by the client as its Authorize place: signed out, it draws the sign-in, its then this address. Google’s return lands back here, and the authorization continues. ${providerNote}`,
        front(`<p class="muted">to continue to <b>${forgeName}</b></p>`, '/oauth/authorize…'));
    }
    if (!state && world.signedIn) {
      return `<main class="sign-in-page">${where(`${site.origin}/`, '')}<p class="mock-note">Signed in, the front page is the chats page: <a class="text-link" href="${esc(u('chats.html'))}">Chats</a>. The signed-out page is <a class="text-link" href="${esc(u('sign-in.html?state=signed-out'))}">after Sign out</a>.</p></main>`;
    }
    const tryAgain = ways('/', 'Continue with Google');
    const another = `<div class="sign-in-ways">${googleButton('Use another Google account', chooser)}</div><p class="sign-in-acts">${back}</p>`;
    switch (state) {
      case 'signed-out':
        return page(`${site.origin}/`, 'After Sign out: jig ended the sign-in and cleared its cookie. The client’s signed-out main, with a line of the page’s own for having signed out, and, while a forge signs its people in through temper, for that forge.',
          front(`<p class="sign-in-notice">You signed out. ${forgeName} keeps its own sign-in: sign out there too. ${ahead('sign-out', 'no single sign-out: draft sign-in.md, section 6')}</p>`, '/'));
      case 'not-member':
        return page(returned('code=…&state=Rk2p…'), 'Google’s return: the state matched this browser’s cookie and the ID token checked, but its address is no member’s. Nothing is minted.',
          `<h1>${stranger} can’t sign in here</h1><p class="muted">Google signed you in, but no one here signs in with this address. Ask an owner to add it, or use the Google account they added.</p>${another}`);
      case 'other-account':
        return page(returned('code=…&state=Rk2p…'), `Google’s return: ${esc(site.member)} is a member, bound at its first sign-in to another Google account (its subject), so this one is refused.`,
          `<h1>${member} signs in here with another Google account</h1><p class="muted">This isn’t the Google account that first signed in with this address. Use that one, or ask an owner.</p>${another}`);
      case 'unverified':
        return page(returned('code=…&state=Rk2p…'), 'Google’s return: the ID token says its address is not verified.',
          `<h1>Google hasn’t verified ${stranger}</h1><p class="muted">temper signs in only an address Google has verified. Verify it with Google, or use another account.</p>${another}`);
      case 'cancelled':
        return page(returned('error=access_denied&state=Rk2p…'), 'Google’s return with its error: the person cancelled at Google.',
          `<h1>The sign-in didn’t finish</h1><p class="muted">${fact('Google: the return’s error, in words', 'You cancelled at Google.')} Nothing was kept.</p>${tryAgain}<p class="sign-in-acts">${back}</p>`);
      case 'expired':
        return page(returned('code=…&state=Rk2p…'), 'Google’s return with a state temper does not hold: used, past the sign-in’s deadline, from before a restart, or with no cookie of this browser’s.',
          `<h1>The sign-in didn’t finish</h1><p class="muted">It took too long, or temper restarted while you were at Google. Nothing was kept.</p>${tryAgain}<p class="sign-in-acts">${back}</p>`);
      case 'busy':
        return page(returned('code=…&state=Rk2p…'), 'Google’s return for a member, and jig’s Mint refused: not ready, while the engine stops, or busy, at the people child’s limits.',
          `<h1>temper couldn’t sign you in just now</h1><p class="muted">It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment.</p>${tryAgain}`);
      case 'unknown-client':
        return page(`${site.origin}/oauth/authorize?client_id=forgejo&redirect_uri=https%3A%2F%2Fgit.example%2Fcallback&response_type=code&scope=openid&state=…`, `An authorization whose redirect URI is not the one registered for its client: jig refuses Authorize, and the page goes nowhere. ${providerNote}`,
          `<h1>This sign-in can’t continue</h1><p class="muted">The site that sent you here asked temper to send you back to an address temper doesn’t know. Nothing was sent anywhere.</p><p class="sign-in-acts">${back}</p>`);
      default:
        return page(`${site.origin}/`, 'No cookie of temper’s, or one the engine doesn’t know: the client’s signed-out main, the same for every deployment, with the ways of signing in the configuration gives.',
          front('', '/'), variants);
    }
  };

  function bindModel() {
    document.querySelectorAll('[data-model-go]').forEach(button => button.addEventListener('click', () => {
      location.href = button.dataset.modelGo;
    }));
    document.querySelectorAll('[data-model-paste]').forEach(form => {
      const input = form.querySelector('input');
      const button = form.querySelector('button');
      // The gallery illustrates states; it never submits or keeps an address.
      button.addEventListener('click', () => {
        input.value = '';
        location.href = button.dataset.modelFinish;
      });
    });
  }

  // ---- Mount ----------------------------------------------------------

  const render = pages[page] || pages.index;
  const html = render();
  const restore = '<button type="button" class="mock-restore" data-show-bar title="Show the mockups’ bar (Esc)" aria-label="Show the mockups’ bar">▾</button>';
  document.body.insertAdjacentHTML('afterbegin', (bare ? '' : barless ? restore : mockbar() + stepPop() + stepsPop() + viewPop()) + html);
  bindCasebar();
  bindSources();
  bindTrees();
  bindLaterActs();
  bindGithub();
  bindPick();
  bindModel();

  const notes = document.getElementById('notes');
  if (notes) {
    const open = show => { notes.hidden = !show; document.querySelectorAll('[data-notes]').forEach(b => b.setAttribute('aria-expanded', String(show))); };
    document.querySelectorAll('[data-notes]').forEach(b => b.addEventListener('click', () => open(notes.hidden)));
    document.querySelectorAll('[data-notes-close]').forEach(b => b.addEventListener('click', () => open(false)));
    if (location.hash === '#notes') open(true);
  }
  const thread = document.querySelector('.terminal-thread');
  if (thread && page === 'chat') thread.scrollTop = thread.scrollHeight;
  document.title = `${document.title} · ${W.name.toLowerCase()} · ${step.label.toLowerCase()}`;
})();
