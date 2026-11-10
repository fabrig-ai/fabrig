from pathlib import Path
import re
roots=['skein/crates/skein-forge','skein/crates/skein-forge-client','skein/crates/skein-forge-connection','skein/tests/forge','temper']
exclude=['temper-settings-form','/generated/','tests/settings.rs','src/settings.rs']
for root in roots:
 for p in Path(root).rglob('*.rs'):
  if any(x in str(p) for x in exclude): continue
  s=p.read_text()
  if 'TooLarge' not in s: continue
  # Only forge Error references; other error types keep their shape.
  if root=='temper' and not any(x in str(p) for x in ['forge','engine-smith/src/answers.rs','engine-smith/src/tests.rs','protocol-store/src/records/api.rs','protocol-store/src/tests/fixtures.rs','system/src/composed.rs']): continue
  if 'src/records/api.rs' in str(p): continue
  own=str(p).startswith('skein/crates/skein-forge/')
  def sub(m):
   prefix=m.group(0)
   if prefix.startswith('forge::Error') and ('translate.rs' in str(p)): return prefix # fake forge error
   tail=s[m.end():]
   line=s[s.rfind('\n',0,m.start())+1:m.start()]
   # Patterns end in an arm arrow/guard/or, or appear in an or-pattern on its own line.
   pattern=bool(re.match(r'\)*\s*(?:=>|\bif\b|\|)',tail)) or bool(re.match(r'\s*\|',line))
   if pattern: return prefix+'(_)'
   return prefix+'('+('crate' if own else 'skein_forge')+'::api::Excess::unnamed())'
  s=re.sub(r'(?<![\w:])(?:(?:skein_forge::api|api|client|d|w)::)?Error::TooLarge\b(?!\s*\()',sub,s)
  p.write_text(s)
