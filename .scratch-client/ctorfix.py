import json,pathlib,re,sys,collections
from rewrite import rewrite,close,types
edits=collections.defaultdict(set)
for l in open(sys.argv[1]):
 try:d=json.loads(l)
 except:continue
 if d.get('reason')!='compiler-message':continue
 m=d['message'];msg=m['message'];sp=next((s for s in m['spans'] if s['is_primary']),None)
 if not sp:continue
 p=pathlib.Path(sp['file_name'])
 if not p.exists() or 'generated' in p.parts:continue
 s=p.read_text();a=len(s.encode()[:sp['byte_start']].decode());b=len(s.encode()[:sp['byte_end']].decode())
 if 'no associated function or constant named `new`' in msg and ('temper_client_protocol_wire' in msg or 'temper-client-protocol-wire/src/generated/' in (m.get('rendered') or '') or 'settings::Agent' in msg):
  z=re.search(r'\b([\w:]+)::(\w+)::$',s[:a])
  if not z:continue
  start=z.start();end=close(s,s.find('(',b))+1;prefix=z[1]
  tail=s[end:];t=re.match(r'\s*\.(?:expect|unwrap|expect_err|unwrap_err|is_err|is_ok|ok)\s*\(',tail)
  if t:
   end=close(s,end+t.end()-1)+1
   if s[end:end+1]=='?':end+=1
  elif (t:=re.match(r'\s*\?',tail)):end+=t.end()
  text=s[start:end]
  new=rewrite(text,[prefix])
  if new!=text:edits[p].add((start,end,new))
 elif (m.get('code') or {}).get('code') in ['E0422','E0425'] and s[a:b].endswith('Parts') and s[a:b][:-5] in types:
  edits[p].add((a,b,s[a:b][:-5]))
for p,es in edits.items():
 s=p.read_text();chosen=[]
 for a,b,n in sorted(es,key=lambda e:e[1]-e[0],reverse=True):
  if any(a<bb and b>aa for aa,bb,_ in chosen):continue
  chosen.append((a,b,n))
 for a,b,n in sorted(chosen,reverse=True):s=s[:a]+n+s[b:]
 p.write_text(s)
print('constructors',sum(map(len,edits.values())),'in',len(edits),'files')
