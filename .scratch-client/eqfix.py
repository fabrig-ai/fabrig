import json,pathlib,sys,re,collections
from rewrite import types
fields={f for v in types.values() for f,t in v.items() if t=='&[u8]'}
changes=collections.defaultdict(set)
for l in open(sys.argv[1]):
 try:d=json.loads(l)
 except:continue
 if d.get('reason')!='compiler-message':continue
 m=d['message'];code=(m.get('code') or {}).get('code')
 if code=='E0277' and "can't compare" in m['message'] or code=='E0308' and '&std::boxed::Box<[u8]>' in (m.get('rendered') or ''):
  z=re.search(r'--> ([^:\n]+):(\d+):',m.get('rendered') or '')
  if not z:continue
  p=pathlib.Path(z[1]);line=int(z[2])-1
  if p.exists():changes[p].add(line)
for p,rows in changes.items():
 lines=p.read_text().splitlines(keepends=True)
 for i in rows:
  lines[i]=re.sub(r'&([a-z_]\w*(?:\.\w+(?:\([^\n]*?\))?)*\.(\w+))\b(?![\w.]|\()',lambda m:m[1]+'.as_ref()' if m[2] in fields else m[0],lines[i])
 p.write_text(''.join(lines))
print('equality lines',sum(map(len,changes.values())))
