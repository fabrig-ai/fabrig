from pathlib import Path
import re
base=Path('skein/crates/skein-forge/src')
# Simple checks whose operands already carry the figures. Intermediate checks give a lower bound.
for p in list((base/'github/plans/reads').glob('*.rs'))+[base/x for x in ['forgejo/plans/contents.rs','forgejo/plans/turn.rs','forgejo/plans/actions.rs','forgejo/plans/reads/done.rs','forgejo/plans/reads.rs']]:
 s=p.read_text()
 pat=r'if ([\w.]+) > u64::from\(bounds.answer_bytes\) \{\s*(return )?Err\(api::Error::TooLarge\((?:api::)?Excess::unnamed\(\)\)\)'
 def byte(m):
  return m.group(0).replace(re.search(r'(?:api::)?Excess::unnamed\(\)',m.group(0))[0],f'api::Excess {{ bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: {m[1]}, exact: false }}')
 s=re.sub(pat,byte,s)
 pat=r'if self.count > self.limit \{\s*return Err\(api::Error::TooLarge\((?:api::)?Excess::unnamed\(\)\)\)\);'
 s=re.sub(pat,lambda m:m[0].replace(re.search(r'(?:api::)?Excess::unnamed\(\)',m[0])[0],'api::Excess { bound: api::Bound::Rows, limit: u64::from(bounds.rows), size: u64::from(self.count), exact: false }'),s)
 p.write_text(s)
def rep(path,a,b):
 p=base/path;s=p.read_text();assert a in s,(path,a);p.write_text(s.replace(a,b))
rep('github/id.rs','api::Excess::unnamed()', 'api::Excess::exact(api::Bound::Forge, u64::from(u32::MAX), id)')
rep('forgejo/plans/writes.rs','let Ok(id) = u32::try_from(repository.id) else {\n                    return Err(api::Error::TooLarge(api::Excess::unnamed()));','let Ok(id) = u32::try_from(repository.id) else {\n                    return Err(api::Error::TooLarge(api::Excess::exact(api::Bound::Forge, u64::from(u32::MAX), repository.id)));')
rep('forgejo/plans/reads/done.rs','d::Document::Repository(repository) => u32::try_from(repository.id),','d::Document::Repository(repository) => repository.id,')
rep('forgejo/plans/reads/done.rs','let Ok(id) = id else {\n        return Err(api::Error::TooLarge(Excess::unnamed()));','let Ok(id) = u32::try_from(id) else {\n        return Err(api::Error::TooLarge(Excess::exact(api::Bound::Forge, u64::from(u32::MAX), id)));')
rep('forgejo/plans/reads/size.rs','api::Excess::unnamed()));\n    }\n    Ok(())','api::Excess::exact(api::Bound::Rows, u64::from(limit), skein_lib::len_u64(count))));\n    }\n    Ok(())')
rep('forgejo/plans/reads/size.rs','if !crate::measure::answer_rows(answer, limit) {\n        return Err(api::Error::TooLarge(api::Excess::unnamed()));','let rows = skein_lib::len_u64(crate::measure::answer_list_rows(answer));\n    if rows > u64::from(limit) {\n        return Err(api::Error::TooLarge(api::Excess::exact(api::Bound::Rows, u64::from(limit), rows)));')
rep('forgejo/plans/contents.rs','if more {\n                    return Err(api::Error::TooLarge(api::Excess::unnamed()));','if more {\n                    return Err(api::Error::TooLarge(api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: self.size.saturating_add(skein_lib::len_u64(bytes.len())).saturating_add(1), exact: false }));')
rep('github/plans/reads/files.rs','if more {\n                return Err(api::Error::TooLarge(Excess::unnamed()));','if more {\n                return Err(api::Error::TooLarge(api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: self.size.saturating_add(skein_lib::len_u64(bytes.len())).saturating_add(1), exact: false }));')
rep('github/plans/reads/files.rs','if skein_lib::len_u64(bytes.len()) > u64::from(self.raw_cap) {\n                return Err(api::Error::TooLarge(Excess::unnamed()));','if skein_lib::len_u64(bytes.len()) > u64::from(self.raw_cap) {\n                return Err(api::Error::TooLarge(api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: self.size.saturating_add(skein_lib::len_u64(bytes.len())), exact: false }));')
# charge keeps the same decision, but carries both figures instead of a bool.
p=base/'forgejo/plans/reads.rs';s=p.read_text();s=s.replace('rows.len() < rows.capacity(),','rows.len(), rows.capacity(),').replace('rows.len() < rows.capacity(), bounds','rows.len(), rows.capacity(), bounds')
s=s.replace('if !charge(', 'if let Err(error) = charge(').replace('return Err(api::Error::TooLarge(api::Excess::unnamed()));\n                }\n                rows.push','return Err(error);\n                }\n                rows.push').replace('return Err(api::Error::TooLarge(api::Excess::unnamed()));\n                    }\n                    rows.push','return Err(error);\n                    }\n                    rows.push')
s=s.replace('if charge(', 'if charge(')
# the non-refusing charge paths become is_ok
s=re.sub(r'if charge\((.*?)\) \{',r'if charge(\1).is_ok() {',s,flags=re.S)
a=s.index('fn charge(');b=s.index('fn row_size(',a)
s=s[:a]+'''fn charge(size: &mut u64, added: u64, count: u32, capacity: u32, bounds: Bounds) -> Result<(), api::Error> {
    if count >= capacity {
        let excess = if count >= bounds.rows {
            api::Excess { bound: api::Bound::Rows, limit: u64::from(bounds.rows), size: u64::from(count).saturating_add(1), exact: false }
        } else {
            api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: size.saturating_add(added), exact: false }
        };
        return Err(api::Error::TooLarge(excess));
    }
    let total = size.checked_add(added).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
    if total > u64::from(bounds.answer_bytes) {
        return Err(api::Error::TooLarge(api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: total, exact: false }));
    }
    *size = total;
    Ok(())
}
''' + s[b:]
s=s.replace('if bounds.rows == 0 {\n        return Err(api::Error::TooLarge(api::Excess::unnamed()));','if bounds.rows == 0 {\n        return Err(api::Error::TooLarge(api::Excess::exact(api::Bound::Rows, 0, 1)));')
s=s.replace('if size::answer(&answer, bounds.rows)? > u64::from(bounds.answer_bytes) {\n        return Err(api::Error::TooLarge(api::Excess::unnamed()));','let size = size::answer(&answer, bounds.rows)?;\n    if size > u64::from(bounds.answer_bytes) {\n        return Err(api::Error::TooLarge(api::Excess::exact(api::Bound::AnswerBytes, u64::from(bounds.answer_bytes), size)));')
p.write_text(s)
