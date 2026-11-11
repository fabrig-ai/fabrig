'use strict';

function journeyKey(event) {
  if (event.metaKey) return null;
  if (event.ctrlKey) return `control ${event.key.toLowerCase()}`;
  if (event.altKey) return `alt ${event.key}`;
  if (event.key === 'Tab' && event.shiftKey) return 'backtab';
  const names = { Enter: 'enter', Tab: 'tab', Backspace: 'backspace', Delete: 'delete',
    ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right',
    Home: 'home', End: 'end', PageUp: 'pageup', PageDown: 'pagedown', Escape: 'escape' };
  return names[event.key] || event.key;
}

document.addEventListener('keydown', event => {
  const page = document.querySelector('pre.tui');
  const key = journeyKey(event);
  if (!page || key === null) return;
  if (page.dataset.journeyKey === key) {
    event.preventDefault();
    location.href = page.dataset.journeyNext;
    return;
  }
  const rel = key === 'n' ? 'next' : key === 'p' ? 'prev' : null;
  if (!rel) return;
  const link = document.head.querySelector(`link[rel="${rel}"]`);
  if (link) {
    event.preventDefault();
    location.href = link.href;
  }
});
