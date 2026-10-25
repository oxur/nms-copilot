"""Slice39 ordered documentation-only and exact existing Clap-help-input preservation."""
import argparse,difflib,hashlib,json,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/repl-presentation-api-documentation')
BASELINE='3fc7c3ff7bdd65b6b13d9db087dbeb01a447007a'
PATHS=['crates/nms-copilot/src/commands.rs','crates/nms-copilot/src/completer.rs','crates/nms-copilot/src/prompt.rs']
BEGIN=b'/// Top-level REPL command parser.\n'
END=b'/// Parse a REPL input line into a command.\n'
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(body):return b''.join(x for x in body.splitlines(keepends=True) if x.strip() and not x.lstrip().startswith((b'///',b'//!')))
def region(body):
 assert body.count(BEGIN)==body.count(END)==1
 return body.split(BEGIN,1)[1].split(END,1)[0].splitlines(keepends=True)
def allowed(old,index):
 line=old[index] if index<len(old) else b''
 if line in [b'    pub action: Option<Action>,\n',b'        target: ShowTarget,\n',b'        target: SetTarget,\n',b'        target: ListTarget,\n']:return True
 return line==b'#[derive(Subcommand, Debug)]\n' and index+1<len(old) and old[index+1] in [b'pub enum Action {\n',b'pub enum SetTarget {\n',b'pub enum ListTarget {\n',b'pub enum ShowTarget {\n']
def help_inputs_preserved(oldbody,newbody,require_all):
 old,new=region(oldbody),region(newbody);sites=[]
 for tag,a,b,c,d in difflib.SequenceMatcher(None,old,new,autojunk=False).get_opcodes():
  if tag=='equal':continue
  if tag!='insert' or not allowed(old,a):return False
  inserted=new[c:d]
  if not inserted or not any(x.lstrip().startswith(b'///') for x in inserted):return False
  if not all(not x.strip() or x.lstrip().startswith(b'///') for x in inserted):return False
  sites.append(a)
 return len(sites)==8 and len(set(sites))==8 if require_all else True
def self_test():
 old=git('show',BASELINE+':'+PATHS[0]);lines=region(old);out=[]
 for index,line in enumerate(lines):
  if allowed(lines,index):out.append(b'/// Synthetic support-only documentation.\n')
  out.append(line)
 candidate=old.split(BEGIN,1)[0]+BEGIN+b''.join(out)+END+old.split(END,1)[1]
 assert non_doc(old)==non_doc(candidate) and help_inputs_preserved(old,candidate,True)
 marker=b'/// Search planets by biome, distance, name.\n';assert candidate.count(marker)==1
 assert not help_inputs_preserved(old,candidate.replace(marker,b'/// Changed old help input.\n',1),True)
 body=git('show',BASELINE+':'+PATHS[1]);anchor=b'            .take(20)\n';assert body.count(anchor)==1
 assert non_doc(body)!=non_doc(body.replace(anchor,b'            .take(19)\n',1))
 print(json.dumps({'support_only':True,'synthetic_eight_documentation_insertions':True,'existing_help_mutation_rejected':True,'actual_unique_completion_cap_anchor':True,'executable_cap_mutation_rejected':True}))
def audit():
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines());assert changed==set(PATHS),changed
 assert not git('ls-files','--others','--exclude-standard','-z')
 for rel in PATHS:assert non_doc((SOURCE/rel).read_bytes())==non_doc(git('show',BASELINE+':'+rel)),rel
 old=git('show',BASELINE+':'+PATHS[0]);assert help_inputs_preserved(old,(SOURCE/PATHS[0]).read_bytes(),True),'existing Clap help input changed or unapproved insertion site'
 body=(SOURCE/PATHS[1]).read_bytes();anchor=b'            .take(20)\n';assert body.count(anchor)==1
 assert non_doc(body)!=non_doc(body.replace(anchor,b'            .take(19)\n',1))
 marker=b'/// Search planets by biome, distance, name.\n';current=(SOURCE/PATHS[0]).read_bytes();assert current.count(marker)==1
 assert not help_inputs_preserved(old,current.replace(marker,b'/// Changed old help input.\n',1),True)
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'ordered_non_doc_comparisons':3,'existing_help_inputs_preserved':True,'exact_new_derived_doc_sites':8,'opposed_completion_cap_mutation_rejected':True,'opposed_existing_help_mutation_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');a=p.parse_args()
 if a.self_test:self_test()
 else:audit()
