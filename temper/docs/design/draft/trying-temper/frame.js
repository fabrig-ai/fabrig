// Each change of the draft's section 4 is on unless named off: by the
// address's #off=41,48 when a page is opened alone, or by the journey
// page around it.
(() => {
  const all = ['41', '42', '43', '44', '45', '46', '47', '48'];
  const apply = off => all.forEach(c => document.documentElement.classList.toggle('off-' + c, off.includes(c)));
  const m = /off=([\d,]*)/.exec(location.hash);
  apply(m ? m[1].split(',') : []);
  addEventListener('message', e => { if (e.data && Array.isArray(e.data.trying)) apply(e.data.trying); });
})();
