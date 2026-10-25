"""Exact ordered source preservation, with only 8 named payload layouts normalized."""
import json,pathlib,re,subprocess,hashlib
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/cache-compat-api-documentation')
BASELINE='65ac646db635aa97030c8a21fc46aa8d93f0ecb3'
PATHS=['crates/nms-cache/src/data.rs', 'crates/nms-cache/src/error.rs', 'crates/nms-cache/src/freshness.rs', 'crates/nms-cache/src/lib.rs', 'crates/nms-cache/src/serialize.rs', 'crates/nms-compat/src/lib.rs', 'crates/nms-compat/src/nomnom.rs']
LAYOUTS={'crates/nms-cache/src/error.rs': [b'TruncatedHeader { actual: usize },', b'SchemaMismatch { found: u32, expected: u32 },', b'SemanticMismatch { found: u32, expected: u32 },', b'Serialize(String),', b'Deserialize(String),', b'Io(io::Error),', b'Incompatible(CacheCompatibilityError),'], 'crates/nms-compat/src/nomnom.rs': [b'Json(#[from] serde_json::Error),']}
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(data):return b''.join(x for x in data.splitlines(keepends=True) if x.strip() and not x.lstrip().startswith((b'///',b'//!')))
def normalized(data,rel):
 data=non_doc(data)
 for declaration in LAYOUTS.get(rel,[]):
  tokens=re.findall(rb'[A-Za-z_][A-Za-z_0-9]*|::|[^\s]',declaration)
  closing=max(i for i,t in enumerate(tokens) if t in (b'}',b')'))
  pieces=[re.escape(t) for t in tokens];pieces[closing]=rb'(?:,\s*)?'+pieces[closing]
  pattern=rb'\s*'.join(pieces)
  data,count=re.subn(pattern,lambda _:declaration,data)
  assert count==1,(rel,declaration,count)
 return data
def compare(data,baseline,rel):return normalized(data,rel)==normalized(baseline,rel)
if __name__=='__main__':
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines())
 assert changed and changed.issubset(PATHS),(changed,PATHS)
 assert not git('ls-files','--others','--exclude-standard','-z'), 'unexpected untracked source'
 for rel in PATHS:assert compare((SOURCE/rel).read_bytes(),git('show',BASELINE+':'+rel),rel),rel
 rel='crates/nms-cache/src/serialize.rs';body=(SOURCE/rel).read_bytes();old=git('show',BASELINE+':'+rel)
 anchor=b'        cached_at,';assert body.count(anchor)==1
 assert not compare(body.replace(anchor,b'        cached_at: 0,',1),old,rel)
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'all_7_ordered_non_doc_comparisons':True,'permitted_named_layout_sites':sum(map(len,LAYOUTS.values())),'opposed_in_memory_executable_mutation_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
