import re,pathlib,subprocess,sys
from rewrite import close,split
lib=pathlib.Path('temper/crates/temper-client-protocol-wire/src/lib.rs').read_text()
helpers={m[2]:m[1] for m in re.finditer(r'pub fn (\w+)\(record: &v1::(\w+)\) -> Box<\[u8\]>',lib)}
def functions(s):
 out={}
 for m in re.finditer(r'\bfn (\w+)\s*\(',s):
  try:
   sigend=close(s,m.end()-1);start=s.find('{',sigend)
   if start<0:continue
   end=close(s,start)
  except ValueError:continue
  out[m[1]]=(start,end)
 return out
for arg in sys.argv[1:]:
 p=pathlib.Path(arg);fs=p.rglob('*.rs') if p.is_dir() else [p]
 for p in fs:
  if 'generated' in p.parts:continue
  try:old=subprocess.check_output(['git','show','HEAD:'+str(p)],text=True,stderr=subprocess.DEVNULL)
  except subprocess.CalledProcessError:continue
  current=p.read_text();edits=[];cf=functions(current)
  for fn,(start,end) in functions(old).items():
   if fn not in cf:continue
   a,b=cf[fn];body=current[a:b+1];ob=old[start:end+1]
   for m in re.finditer(r'let (\w+)\s*=\s*(?:\w+::)+(\w+)::new\s*\(',ob):
    var,typ=m[1],m[2]
    if typ not in helpers:continue
    stop=close(ob,m.end()-1);lim,val=split(ob[m.end():stop]);tail=ob[stop+1:]
    if re.match(r'\s*\.ok\(\)\s*\?',tail):suffix='.ok()?'
    elif (z:=re.match(r'\s*\.expect\(("[^"]*")\)',tail)):suffix='.expect('+z[1]+')'
    else:continue
    name=helpers[typ]
    # aliases wire only where explicit target imports; roots helper alias inferred from old uses
    helper=re.search(r'\b(\w+)::'+name+r'\(&'+var+r'\)',ob)
    if not helper:continue
    call=helper[0]
    if call in body:
     body=body.replace(call,var+'.encode('+lim+')'+suffix)
   # inline root helper holding constructor
   for m in list(re.finditer(r'\b(\w+)::(\w+)\(\s*&\s*(\w+)::(\w+)::new\s*\(',ob)):
    alias,name,prefix,typ=m[1],m[2],m[3],m[4]
    if helpers.get(typ)!=name:continue
    stop=close(ob,m.end()-1);lim,val=split(ob[m.end():stop]);tail=ob[stop+1:]
    if re.match(r'\s*\.ok\(\)\s*\?',tail):suffix='.ok()?'
    elif (z:=re.match(r'\s*\.expect\(("[^"]*")\)',tail)):suffix='.expect('+z[1]+')'
    else:continue
    # find corresponding helper in new body; all such calls of this type get same limits per fn
    pat=re.compile(r'\b'+alias+'::'+name+r'\(\s*&')
    pos=0
    while (n:=pat.search(body,pos)):
     stop=close(body,body.find('(',n.start()));expr=body[n.end():stop].strip().removesuffix(',').strip()
     new='('+expr+').encode('+lim+')'+suffix
     body=body[:n.start()]+new+body[stop+1:];pos=n.start()+len(new)
   if body!=current[a:b+1]:edits.append((a,b+1,body))
  for a,b,v in sorted(edits,reverse=True):current=current[:a]+v+current[b:]
  if edits:p.write_text(current);print(p,len(edits))
