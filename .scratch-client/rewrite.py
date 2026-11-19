import re,pathlib,json,sys
meta=json.loads(pathlib.Path('.scratch-client/meta.json').read_text())
types={k:v for m in meta.values() for k,v in m['types'].items()}; types['LimitOverrides']=types.pop('TemperLimits',{})
paths={k:v for m in meta.values() for k,v in m['paths'].items()}
def close(s,i):
 stack=[s[i]]; j=i+1
 while j<len(s):
  if s.startswith('//',j):
   n=s.find('\n',j);j=len(s) if n<0 else n;continue
  if s[j]=='"':
   j+=1
   while j<len(s):
    if s[j]=='\\':j+=2;continue
    if s[j]=='"':break
    j+=1
  elif s[j] in '([{':stack.append(s[j])
  elif s[j] in ')]}':
   stack.pop()
   if not stack:return j
  j+=1
 raise ValueError(s[i:i+200])
def split(s):
 i=0
 while i<len(s):
  if s[i] in '([{':i=close(s,i)
  elif s[i]==',':return s[:i].strip(),s[i+1:].strip().rstrip(',').strip()
  i+=1
 raise ValueError(s[:200])
def rewrite(s,prefixes):
 # rename only qualified names, not logical form Shape
 s=re.sub(r'\b('+ '|'.join(map(re.escape,prefixes))+r')::TemperLimits(Parts)?\b',lambda m:m[1]+'::LimitOverrides'+(m[2] or ''),s)
 s=s.replace('.temper_limits_overrides','.limit_overrides_overrides').replace('temper_limits_overrides:', 'limit_overrides_overrides:')
 for prefix in prefixes:
  for old,new in paths.items():
   if new:s=s.replace(prefix+'::Path::'+old,prefix+'::paths::'+new)
  names='|'.join(types)
  pat=re.compile(r'\b'+re.escape(prefix)+r'::('+names+r')::new\s*\(')
  # outer first with recursive args
  pos=0
  while (m:=pat.search(s,pos)):
   end=close(s,m.end()-1)
   limits,value=split(s[m.end():end]); value=rewrite(value,[prefix]);
   value=re.sub(r'\b'+re.escape(prefix)+r'::(\w+)Parts\b',lambda m:prefix+'::'+m[1],value)
   q=end+1
   suffix=re.match(r'\s*\.(?:expect|unwrap)\s*\(',s[q:])
   if suffix:
    q=close(s,q+suffix.end()-1)+1; new=value
   elif (z:=re.match(r'\s*\.ok\(\)\s*\?',s[q:])):q+=z.end();new=value
   elif (z:=re.match(r'\s*\?',s[q:])):q+=z.end();new=value
   elif (z:=re.match(r'\s*\.ok\(\)(?!\s*\.)',s[q:])):q+=z.end();new='{ let value = '+value+'; value.measure('+limits+').ok()?; Some(value) }'
   elif re.match(r'\s*\.(?:expect_err|unwrap_err|is_err|is_ok)',s[q:]):new='('+value+').encode('+limits+')'
   else:
    # raw constructor result: make checking explicit until its use is reviewed
    new='{ let value = '+value+'; match value.encode('+limits+') { Ok(_) => Ok(value), Err(problem) => Err(problem) } }'
   s=s[:m.start()]+new+s[q:];pos=m.start()+len(new)
  s=re.sub(r'\b'+re.escape(prefix)+r'::(\w+)Parts\b',lambda m:prefix+'::'+m[1] if m[1] in types else m[0],s)
 return s
if __name__=='__main__':
 prefixes=sys.argv[1].split(',')
 for name in sys.argv[2:]:
  p=pathlib.Path(name)
  files=list(p.rglob('*.rs')) if p.is_dir() else [p]
  for f in files:
   if 'generated' in f.parts:continue
   old=f.read_text();new=rewrite(old,prefixes)
   if new!=old:f.write_text(new)
