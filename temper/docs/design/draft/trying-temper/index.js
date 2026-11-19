(() => {
  const all = ['41', '42', '43', '44', '45', '46', '47', '48'];
  const KEY = 'trying-temper-off';
  let off = [];
  try { off = JSON.parse(localStorage.getItem(KEY) || '[]'); } catch { off = []; }
  const frames = () => document.querySelectorAll('iframe');
  const apply = () => {
    all.forEach(c => document.documentElement.classList.toggle('off-' + c, off.includes(c)));
    document.querySelectorAll('.sw').forEach(b => b.setAttribute('aria-pressed', String(!off.includes(b.dataset.c))));
    frames().forEach(f => { try { f.contentWindow.postMessage({ trying: off }, '*'); } catch { /* not loaded */ } });
    try { localStorage.setItem(KEY, JSON.stringify(off)); } catch { /* none */ }
  };
  frames().forEach(f => f.addEventListener('load', () => f.contentWindow.postMessage({ trying: off }, '*')));
  document.querySelector('.switches').addEventListener('click', e => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.dataset.all) off = b.dataset.all === 'on' ? [] : all.slice();
    else off = off.includes(b.dataset.c) ? off.filter(c => c !== b.dataset.c) : off.concat(b.dataset.c);
    apply();
  });
  apply();
})();
