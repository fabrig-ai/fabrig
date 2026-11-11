// The loader (ui/web/bundle.md, 3); words belong to the markup.
const $=s=>document.querySelector(s), b=$("#boot"), p=$("#boot progress"), l=$("#boot-line"), n=$("#boot-note"), r=$("#boot button"), w={...b.dataset};
const url=s=>new URL(s,import.meta.url).href, mb=n=>(n/1e6).toFixed(1), say=(s,v)=>s.replace(/\{(\w+)\}/g,(_,k)=>v[k]);
const rows=[GLUE,WASM,...SHEETS], TOTAL=rows.reduce((s,r)=>s+r[1],0), begun=performance.now();
let got=0, first=0, last=begun, left="", over=0, travel=TOTAL, answered=0;
const figure=s=>say(s,{got:mb(got/TOTAL*travel),total:mb(travel)});
function draw(){
  if(over)return;
  p.value=got/TOTAL;
  const waiting=performance.now()-last>15e3;
  if(!answered){l.textContent=waiting?w.waiting:"";return;}
  if(got<TOTAL)l.textContent=waiting?w.waiting:figure(w.figure)+left;
  else{b.dataset.state="start";l.textContent=w.starting;}
}
function add(n){last=performance.now();first||=last;got+=n;draw();}
function stop(){over=1;clearInterval(tick);}
function fail(why){
  if(over)return;
  stop();b.dataset.state="failed";l.textContent=w.failed;
  n.textContent=why||(!got?w.silent:got<TOTAL?figure(w.dropped):w.noStart);r.hidden=false;
}
const tick=setInterval(()=>{
  const now=performance.now();
  if(now-begun>8e3)n.textContent=now-last>15e3?"":w.keeps;
  if(first&&now-first>3e3){
    const s=(TOTAL-got)/got*(now-first)/1e3;
    left=s<5?"":say(w.left,{left:say(s<60?w.seconds:w.minutes,{n:s<60?Math.round(s/5)*5:Math.round(s/60)})});
  }
  draw();
},500);
r.addEventListener("click",()=>location.reload());
const sheets=Promise.all(SHEETS.map(([name,n])=>new Promise((done,no)=>{
  const link=document.createElement("link");link.rel="stylesheet";link.href=url(name);
  link.addEventListener("load",()=>{add(n);done();});link.addEventListener("error",no);document.head.append(link);
})));
sheets.catch(()=>fail());
new MutationObserver((_,watch)=>{
  watch.disconnect();sheets.then(()=>{
    stop();b.classList.add("out");b.inert=true;setTimeout(()=>b.remove(),200);
  },()=>{});
}).observe($("#app"),{childList:true});
async function start(){
  if(typeof WebAssembly!=="object")return fail(w.noWasm);
  const [glue,answer]=await Promise.all([import(url(GLUE[0])).then(m=>{add(GLUE[1]);return m;}),fetch(url(WASM[0]))]);
  travel=rows.reduce((s,r)=>s+r[answer.headers.get("Content-Encoding")==="gzip"?2:1],0);answered=1;
  if(!answer.ok)return fail(say(w.answered,{status:answer.status}));
  draw();const count=answer.clone().body.getReader();
  (async()=>{for(let r;!(r=await count.read()).done;)add(r.value.length);})().catch(()=>{});
  await sheets;await glue.default({module_or_path:answer});
}
start().catch(()=>fail());
