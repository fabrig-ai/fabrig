import json,sys,collections
errors=[]
for line in open(sys.argv[1]):
 try:d=json.loads(line)
 except ValueError:continue
 if d.get('reason')=='compiler-message' and d['message']['level'] in ['error','warning']:
  m=d['message'];errors.append(m)
if len(sys.argv)>2 and sys.argv[2]=='full':
 for m in errors:print(m['rendered'])
else:
 for m in errors:
  spans=[s for s in m['spans'] if s['is_primary']];s=spans[0] if spans else {}
  print(f"{s.get('file_name','')}:{s.get('line_start','')} {m.get('code') and m['code']['code']} {m['message']}")
print('diagnostics',len(errors))
