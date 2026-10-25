"""Slice43 one-file ordered documentation-only preservation and actual status-response control."""
import argparse,json,hashlib,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/repl-dispatch-api-documentation')
BASELINE='2ce768fc390810eac4bc3d59c7cefd85a2ddce14'
PATH='crates/nms-copilot/src/dispatch.rs'
ANCHOR=b'        Action::Status => Ok(session.format_status()),\n'
def git(*a):return subprocess.check_output(['git',*a],cwd=SOURCE)
def non_doc(b):return b''.join(x for x in b.splitlines(keepends=True) if x.strip() and not x.lstrip().startswith((b'///',b'//!')))
def opposed(b):
 assert b.count(ANCHOR)==1
 assert non_doc(b)!=non_doc(b.replace(ANCHOR,b'        Action::Status => Ok(String::new()),\n',1))
def run(support):
 old=git('show',BASELINE+':'+PATH)
 if support:
  assert non_doc(old)==non_doc(b'//! Synthetic support-only documentation.\n'+old);opposed(old)
  print(json.dumps({'support_only':True,'synthetic_doc_addition_preserves_ordered_non_doc':True,'actual_status_empty_response_mutation_rejected':True}));return
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines());assert changed=={PATH},changed
 assert not git('ls-files','--others','--exclude-standard','-z')
 now=(SOURCE/PATH).read_bytes();assert non_doc(old)==non_doc(now);opposed(now)
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'exact_changed_paths':sorted(changed),'ordered_non_doc_equal':True,'actual_status_empty_response_mutation_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');a=p.parse_args();run(a.self_test)
