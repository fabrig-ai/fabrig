import re
from pathlib import Path
# Remaining newly introduced usage records use the base's lean-record API.
def lean_new(s,qual):
 pat=re.compile(r'\b'+re.escape(qual)+r'::(\w+)::new\(')
 while True:
  matches=list(pat.finditer(s)); changed=False
  for m in reversed(matches):
   i=m.end(); depth=1; comma=None; j=i
   while depth and j<len(s):
    c=s[j]
    if c in '({[': depth+=1
    if c in ')}]': depth-=1
    if c==',' and depth==1 and comma is None: comma=j
    j+=1
   if comma is None: continue
   arg=s[comma+1:j-1].strip().rstrip(',').strip()
   prefix=f'{qual}::{m[1]}Parts'
   if not arg.startswith(prefix): continue
   arg=arg.replace(prefix,f'{qual}::{m[1]}',1)
   end=j
   tail=re.match(r'\s*\.(?:unwrap\(\)|expect\("[^"]*"\)|ok\(\)\?)',s[j:])
   if tail: end=j+tail.end()
   s=s[:m.start()]+arg+s[end:]; changed=True
  if not changed: return s
for f in ['jig/crates/jig-protocol-worker-link/src/translate.rs','jig/crates/jig-protocol-engine-workers/src/translate.rs','jig/crates/jig-protocol-worker-wire/src/tests/translation.rs']:
 p=Path(f); s=lean_new(p.read_text(),'wire')
 if f.endswith('/tests/translation.rs'):
  s=s.replace('window.label().into()','window.label.clone()')
  for n in ('length','used','resets'): s=s.replace('*window.'+n+'()', 'window.'+n)
  s=s.replace('window.state()', 'window.state')
  # Usage is new in 19, whereas model refusals remain bridged at 18.
  start=s.index('fn usage_records()'); end=s.index('#[test]\nfn older_notices',start)
  part=s[start:end].replace('eighteen','nineteen').replace(' 18',' 19').replace('[16, 17]','[16, 17, 18]')
  s=s[:start]+part+s[end:]
  s=s.replace('fn a_worker_of_eighteen_cannot_send_whole_to_an_engine_below_eighteen()', 'fn a_worker_of_nineteen_cannot_send_whole_to_an_engine_below_nineteen()')
  start=s.index('fn a_worker_of_nineteen'); end=s.index('\nfn model_answer',start)
  s=s[:start]+s[start:end].replace('[16, 17]','[16, 17, 18]').replace('version: 18','version: 19')+s[end:]
 if 'engine-workers' in f:
  s=s.replace('.ok()?.into_parts()', '.ok()?').replace('windows.into_parts().windows','windows.windows').replace('let row = row.into_parts();','').replace('let failure = failure.into_parts();','')
  a=s.index('            let record =\n                wire::ReadUsage')
  b=s.index('\n        host::LinkDown::ReadList',a)
  s=s[:a]+'''            Ok(Down::ReadUsage(wire::ReadUsage { read, source: Box::from(source.as_bytes()) }))
        }'''+s[b:]
 else:
  s=s.replace('?.into_parts()', '?')
  if 'worker-link' in f:
   s=s.replace('source_failed(reason, detail, limits)?','source_failed(reason, detail)')
   s=s.replace('let mut body = Writer::new(skein_lib::length::usize_of(record.measure()));\n            record.encode(&mut body).ok()?;\n            frame(wire::Kind::Usage.number(), body.finish())','frame(wire::Kind::Usage.number(), record.encode(limits).ok()?)')
   s=s.replace('let mut body = Writer::new(skein_lib::length::usize_of(record.measure()));\n    record.encode(&mut body).ok()?;\n    frame(wire::Kind::UsageRead.number(), body.finish())','frame(wire::Kind::UsageRead.number(), record.encode(limits).ok()?)')
 p.write_text(s)
p=Path('smith/tests/codecs/tests/suite/channel.rs'); s=p.read_text().replace('smith_channel::UsageWindowParts','smith_channel::UsageWindow'); s=s.replace('let parts = smith_channel::UsageWindow','let window = smith_channel::UsageWindow'); s=s.replace('            let window = smith_channel::UsageWindow::new(&CEILINGS, parts).expect("window");\n',''); s=lean_new(s,'smith_channel'); s=s.replace('read.measure()', 'read.measure(&CEILINGS).expect("bounded usage")'); p.write_text(s)
p=Path('smith/crates/smith-protocol-channel/src/component/notices.rs'); s=p.read_text(); a=s.index('        let window = match smith_channel::UsageWindow::new('); b=s.index('        let frame = match smith_channel::frame(smith_channel::Body::Usage',a); s=s[:a]+'''        let window = smith_channel::UsageWindow {
            label: window.label,
            length: window.length,
            used: window.used,
            resets: window.resets,
            state,
        };
        let record = smith_channel::Usage { account, window };
        match record.measure(&self.bodies) {
            Ok(_) => (),
            Err(problem) => return Err(Error::ChannelBody(problem)),
        }
'''+s[b:]; p.write_text(s)
