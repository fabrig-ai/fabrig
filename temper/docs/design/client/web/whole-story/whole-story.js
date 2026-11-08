'use strict';
// The mockups' night, kept in this browser, and How work runs' tabs and stops, as the client would.
(function () {
  const KEY = 'temper-night';
  let on = false;
  try { on = localStorage.getItem(KEY) === '1'; } catch (e) { /* none kept */ }
  const asked = location.hash.slice(1).split(',');
  if (asked.includes('night')) on = true;
  if (asked.includes('day')) on = false;
  const root = document.documentElement;
  const apply = () => {
    root.toggleAttribute('data-night', on);
    for (const b of document.querySelectorAll('.night')) {
      b.setAttribute('aria-pressed', String(on));
      b.title = b.ariaLabel = on ? 'Day mode' : 'Night mode';
    }
  };
  apply();

  const close = panel => {
    for (const s of panel.querySelectorAll('.ways-stop')) s.setAttribute('aria-expanded', 'false');
    for (const c of panel.querySelectorAll('.ways-zoom')) c.hidden = true;
    const wide = panel.querySelector('.ways-wide');
    if (!wide) return;
    wide.setAttribute('viewBox', '0 0 1020 140');
    wide.querySelector('.ways-caret').toggleAttribute('hidden', true);
  };
  const open = stop => {
    const panel = stop.closest('.ways-panel');
    const i = stop.dataset.i;
    const was = stop.getAttribute('aria-expanded') === 'true';
    close(panel);
    if (was) return;
    for (const s of panel.querySelectorAll(`.ways-stop[data-i="${i}"]`)) s.setAttribute('aria-expanded', 'true');
    panel.querySelector(`.ways-zoom[data-i="${i}"]`).hidden = false;
    const wide = panel.querySelector('.ways-wide');
    const x = Number(panel.querySelector(`.ways-wide .ways-stop[data-i="${i}"]`).dataset.x);
    wide.setAttribute('viewBox', '0 0 1020 154');
    const caret = wide.querySelector('.ways-caret');
    caret.setAttribute('points', `${x - 6},154 ${x},148 ${x + 6},154`);
    caret.removeAttribute('hidden');
  };
  const press = target => {
    const tab = target.closest('.ways-tabs > [role=tab]');
    if (tab) {
      for (const t of tab.parentElement.children) t.setAttribute('aria-selected', String(t === tab));
      for (const p of document.querySelectorAll('.ways-panel')) p.hidden = p.dataset.way !== tab.dataset.way;
      return true;
    }
    const stop = target.closest('.ways-stop');
    if (stop) { open(stop); return true; }
    return false;
  };
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element)) return;
    if (event.target.closest('.night')) {
      on = !on;
      try { localStorage.setItem(KEY, on ? '1' : '0'); } catch (e) { /* the page still turns */ }
      apply();
      return;
    }
    if (press(event.target)) event.preventDefault();
  });
  document.addEventListener('keydown', event => {
    if ((event.key === 'Enter' || event.key === ' ') && event.target instanceof Element && event.target.closest('.ways-stop')) {
      event.preventDefault();
      press(event.target);
    }
  });
  document.addEventListener('DOMContentLoaded', () => {
    apply();
    const want = asked.find(a => a.startsWith('stop='));
    if (want) {
      const stop = document.querySelector(`.ways-panel:not([hidden]) .ways-wide .ways-stop[data-i="${Number(want.slice(5))}"]`);
      if (stop) open(stop);
    }
    const way = asked.find(a => a.startsWith('way='));
    if (way) {
      const tab = document.querySelector(`.ways-tabs > [data-way="${way.slice(4)}"]`);
      if (tab) press(tab);
    }
  });
})();
