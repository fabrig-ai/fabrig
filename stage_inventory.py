from pathlib import Path
import re
P=Path('temper/crates/temper-client-view/src')
N='accounts cards choice confirm connect_forge connect_model flow forge_page frame how limits look model_check paths public rail repository_link settings shared sheet sign_in_wait team team_follows team_sessions times views welcome words'.split()
def functions(t):
 out={}
 for m in re.finditer(r'^(?:(?:pub(?:\([^\n]*?\))?) )?fn (\w+)',t,re.M):
  end=re.search(r'^}',t[m.start():],re.M)
  if not end:continue
  start=t.rfind('\n\n',0,m.start())+2
  out[m[1]]=(start,m.start()+end.end(),t[start:m.start()+end.end()])
 return out
if __name__=='__main__':
 import sys
 for n in sys.argv[1:] or N:
  t=(P/'tests'/f'{n}.rs').read_text();fs=functions(t)
  print('\n###',n)
  print(t[:next(iter(fs.values()))[0]] if fs else t)
  for name,(start,end,b) in fs.items():
   calls=[x for x in fs if re.search(r'\b'+x+r'\s*\(',b[b.find('{')+1:])]
   print(t[:start].count('\n')+1,'TEST' if '#[test]' in b else 'help',name,':',','.join(sorted(set(re.findall(r'crate::[\w:]+|(?:fixtures|f|p|cf|forge|super)::[\w:]+',b)))), 'calls='+','.join(calls))
