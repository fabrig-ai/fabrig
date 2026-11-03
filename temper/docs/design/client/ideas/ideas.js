// Ideas: the banner, the side pane and the mockups' strip, shared by
// every page. A page's own drawing is plain HTML; this draws only what
// every page repeats. Nothing is sent anywhere.
(() => {
  const PAGES = [
    ['index.html', 'Contents'],
    ['home.html', 'Home'],
    ['waiting.html', 'Waiting on you'],
    ['draft.html', 'Draft session'],
    ['start-goal.html', 'Start a goal'],
    ['goal.html', 'Goal'],
    ['away.html', 'Back from away'],
    ['small.html', 'A small project'],
    ['settings.html', 'How work runs'],
  ];
  const body = document.body;
  const d = body.dataset;
  const bare = new URLSearchParams(location.search).has('bare');
  if (bare) body.classList.add('bare');
  const keep = h => bare ? `${h}?bare` : h;
  const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

  const small = d.project === 'hello';
  const NAV = small
    ? [['home', 'home.html', 'Home'], ['chats', 'small.html', 'Chats']]
    : [['home', 'home.html', 'Home'], ['waiting', 'waiting.html', 'Waiting', d.waiting], ['goals', 'goal.html', 'Goals'], ['chats', 'draft.html', 'Chats'], ['drafts', 'draft.html', 'Drafts'], ['changes', '', 'Changes']];

  const banner = document.querySelector('[data-idea-banner]');
  if (banner) {
    const page = location.pathname.split('/').pop() || 'index.html';
    const i = PAGES.findIndex(p => p[0] === page);
    const prev = PAGES[i - 1], next = PAGES[i + 1];
    const strip = `<div class="idea-strip"><strong>Idea</strong><span>${d.note || ''}</span><nav aria-label="Mockups">${prev ? `<a href="${keep(prev[0])}">‹ ${esc(prev[1])}</a>` : ''}${i > 1 ? `<a href="${keep('index.html')}">Contents</a>` : ''}${next ? `<a href="${keep(next[0])}">${esc(next[1])} ›</a>` : ''}</nav></div>`;
    const links = NAV.map(([key, h, label, n]) => `<a href="${h ? keep(h) : '#'}"${key === d.active ? ' aria-current="page"' : ''}><span>${label}</span>${n ? `<span class="idea-count">${esc(n)}</span>` : ''}</a>`).join('');
    const away = d.away
      ? `<button type="button" class="idea-away" aria-pressed="true">${esc(d.away)}</button>`
      : '<button type="button" class="idea-away" aria-pressed="false">Away: off</button>';
    const projects = small ? ['hello', 'fabrig'] : ['fabrig', 'hello', 'Demo'];
    banner.outerHTML = `${strip}<header class="temper-banner"><a aria-label="temper home" class="temper-brand" href="${keep('home.html')}"><span>temper</span></a>
      <details class="temper-menu"><summary>Menu</summary>
        <details class="temper-projects"><summary>${esc(projects[0])}</summary>${projects.map((p, k) => `<button type="button"${k === 0 ? ' aria-pressed="true"' : ''}>${esc(p)}</button>`).join('')}</details>
        <nav aria-label="Main navigation" class="temper-navigation">${links}</nav>
        <div class="temper-corner">${small ? '' : away}<section data-region="temper-link"><div role="status" aria-label="Live" class="temper-live"><span>Live</span></div></section><a href="${keep('settings.html')}" role="status"><span>Owner</span></a></div>
      </details></header>`;
  }

  const row = (status, cls, title, t, when, href = '#') => `<li class="temper-row"><div class="temper-meta"><div role="status" class="${cls}"><span>${status}</span></div><div><span>${small ? 'hello' : 'fabrig'}</span><span> · </span><a href="${href}"><span>${t}</span></a></div></div><p><span>${title}</span></p><p class="temper-details"><span></span></p><p class="temper-time"><span>${when}</span></p></li>`;
  const pane = document.querySelector('[data-idea-pane]');
  if (pane) {
    const drafts = small ? '' : `<div class="temper-heading"><h1><span>Drafts</span></h1></div>
      <ul aria-label="Drafts" class="temper-rows">
        ${row('writing', 'temper-status-waiting', 'away-mode.md', 'T4', '1 min ago', keep('draft.html'))}
        ${row('goal running', 'temper-status-quiet', 'first-run-projects.md', 'G2', 'Mon', keep('goal.html'))}
        ${row('idea', 'temper-status-quiet', 'dashboards.md', 'D', 'Oct 28')}
      </ul>`;
    const chats = small
      ? `${row('waiting on your words', 'temper-status-waiting', 'Add a README', 'T3', '4 min ago')}
         ${row('landed', 'temper-status-quiet', 'Hello in Rust', 'T2', '1 h ago')}
         ${row('closed', 'temper-status-quiet', 'What is in this repository?', 'T1', '1 h ago')}`
      : `${row('waiting on your words', 'temper-status-waiting', 'Away mode, draft', 'T4', '1 min ago', keep('draft.html'))}
         ${row('landed', 'temper-status-quiet', 'Pane chat line', 'T3', '2 h ago')}
         ${row('handed to a goal', 'temper-status-quiet', 'Welcome stall', 'T2', '5 h ago')}`;
    pane.innerHTML = `<div class="temper-chatbar"><div><p><span>New chat</span></p></div></div>
      <div class="temper-context"><p><span class="temper-repository">${small ? 'work/hello' : 'fabrig-ai/fabrig'}</span>${small ? '' : ' · <a href="' + keep('draft.html') + '" style="color:var(--temper-accent-dark)">New draft</a>'}</p></div>
      ${drafts}
      <div class="temper-heading"><h1><span>Chats</span></h1><div class="temper-filters"><button type="button" aria-pressed="true">All</button><button type="button">Live</button><button type="button">Closed</button></div></div>
      <ul aria-label="Your chats" class="temper-rows">${chats}</ul>`;
  }
})();
