import json,pathlib,sys,re,collections
from rewrite import close
edits=collections.defaultdict(set)
for l in open(sys.argv[1]):
 try:d=json.loads(l)
 except:continue
 if d.get('reason')!='compiler-message' or (d['message'].get('code') or {}).get('code')!='unused_variables':continue
 m=d['message'];sp=next((s for s in m['spans'] if s['is_primary']),None)
 if not sp:continue
 p=pathlib.Path(sp['file_name']);s=p.read_text();at=len(s.encode()[:sp['byte_start']].decode())
 if s[at:at+6]!='limits':continue
 a=s.rfind('fn ',0,at);b=s.find('{',at);end=close(s,b)
 sig=s[a:b];body=s[b+1:end]
 if 'Option<w::' in sig and 'List<' not in sig:
  # preserve a projection's existing result and validate it against its family's limits
  if 'Result<Option<w::' in sig:
   new='\n    let value = {'+body+'}?;\n    if let Some(record) = &value { record.measure(limits)?; }\n    Ok(value)\n'
  else:new='\n    let value = {'+body+'}?;\n    value.measure(limits).ok()?;\n    Some(value)\n'
  edits[p].add((b+1,end,new))
 elif 'List<' in sig:edits[p].add((at,at+6,'_limits'))
for p,es in edits.items():
 s=p.read_text()
 for a,b,n in sorted(es,reverse=True):s=s[:a]+n+s[b:]
 p.write_text(s)
print('checked projections',sum(map(len,edits.values())))
