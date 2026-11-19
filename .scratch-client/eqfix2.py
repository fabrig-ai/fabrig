import json,pathlib,sys,re,collections
from rewrite import types,close
fields={f for v in types.values() for f,t in v.items() if t=='&[u8]'}
def borrowed(s):
 edits=[]
 for m in re.finditer(r'(?<!&)&(?!&)([a-z_]\w*(?:::\w+)*)',s):
  a=m.start();pos=m.end()
  while pos<len(s):
   if s[pos] in '([':pos=close(s,pos)+1
   elif s[pos]=='?':pos+=1
   elif s[pos]=='.' and (t:=re.match(r'\.\w+',s[pos:])):pos+=t.end()
   else:break
  expr=s[m.start()+1:pos]
  t=re.search(r'\.(\w+)$',expr)
  if t and t[1] in fields:edits.append((a,pos,expr+'.as_ref()'))
 last=len(s)+1
 for a,b,n in sorted(edits,reverse=True):
  if b>last:continue
  s=s[:a]+n+s[b:];last=a
 return s
changes=collections.defaultdict(set)
for l in open(sys.argv[1]):
 try:d=json.loads(l)
 except:continue
 if d.get('reason')!='compiler-message':continue
 m=d['message'];code=(m.get('code') or {}).get('code')
 if (code=='E0277' and "can't compare" in m['message'] or code=='E0308'):
  z=re.search(r'--> ([^:\n]+):(\d+):',m.get('rendered') or '')
  if not z:continue
  p=pathlib.Path(z[1]);line=int(z[2])-1
  if not p.exists():continue
  s=p.read_text();start=sum(map(len,s.splitlines(keepends=True)[:line]));end=s.find('\n',start)
  candidates=[]
  for macro in ['assert_eq!(', 'assert_ne!(']:
   here=s.find(macro,start,end)
   candidates.append(here if here>=0 else s.rfind(macro,0,start+1))
  a=max(candidates)
  if a<0:continue
  b=close(s,s.find('(',a))+1
  if b<start:continue
  n=borrowed(s[a:b])
  if 'SettingEdit' in m['message']:n=n.replace('&decoded.items.as_slice()[1].edit','decoded.items.as_slice()[1].edit').replace('&decoded.items.as_slice()[2].edit','decoded.items.as_slice()[2].edit')
  if n!=s[a:b]:changes[p].add((a,b,n))
for p,es in changes.items():
 s=p.read_text()
 for a,b,n in sorted(es,reverse=True):s=s[:a]+n+s[b:]
 p.write_text(s)
print('equality blocks',sum(map(len,changes.values())))
