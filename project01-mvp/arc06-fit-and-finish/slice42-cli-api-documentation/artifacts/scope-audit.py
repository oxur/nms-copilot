"""Slice42 complete CLI doc surface; unchanged derive inputs and ordered executable bytes."""
import argparse,hashlib,json,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/cli-api-documentation')
BASELINE='83e67d62d46fa0a3aee004f0f2a51c8d53cf33e8'
PATHS=['crates/nms-cli/src/'+name+'.rs' for name in ['main','completions','convert','export','find','import','info','list','route','saves','show','stats']]
ANCHOR=b'            "csv" => Ok(Self::Csv),\n'
HELP=b'    /// Use a specific save slot (1-15) instead of most recent.\n'
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(body):return b''.join(line for line in body.splitlines(keepends=True) if line.strip() and not line.lstrip().startswith((b'///',b'//!')))
def normalized(body,rel):
 if rel.endswith('/show.rs'):
  for variant in (b'System',b'Base'):
   old=b'    '+variant+b' { name: String },\n'
   new=b'    '+variant+b' {\n        name: String,\n    },\n'
   body=body.replace(old,new)
 return non_doc(body)
def main_preserved(old,current):
 if not current.endswith(old):return False
 prefix=current[:-len(old)]
 return all(not line.strip() or line.startswith(b'//!') for line in prefix.splitlines())
def opposed(export,main):
 assert export.count(ANCHOR)==1
 assert non_doc(export)!=non_doc(export.replace(ANCHOR,b'            "csv" => Ok(Self::Json),\n',1))
 assert main.count(HELP)==1
 assert not main_preserved(main,main.replace(HELP,HELP.replace(b'1-15',b'1-14'),1))
def run(support):
 original={rel:git('show',BASELINE+':'+rel) for rel in PATHS}
 if support:
  for rel,old in original.items():
   current=b'//! Synthetic support-only documentation.\n'+old
   if rel.endswith('/show.rs'):
    for variant in (b'System',b'Base'):
     oldline=b'    '+variant+b' { name: String },\n'
     newline=b'    '+variant+b' {\n        /// Exact target name.\n        name: String,\n    },\n'
     assert current.count(oldline)==1
     current=current.replace(oldline,newline,1)
   assert normalized(old,rel)==normalized(current,rel),rel
   if rel.endswith('/main.rs'):assert main_preserved(old,current)
  opposed(original[PATHS[3]],original[PATHS[0]])
  print(json.dumps({'support_only':True,'synthetic_doc_additions':12,'two_exact_show_layouts':True,'actual_csv_dispatch_mutation_rejected':True,'actual_existing_derive_help_mutation_rejected':True}));return
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines());assert changed==set(PATHS),changed
 assert not git('ls-files','--others','--exclude-standard','-z')
 for rel,old in original.items():
  current=(SOURCE/rel).read_bytes()
  assert normalized(old,rel)==normalized(current,rel),rel
  if rel.endswith('/main.rs'):assert main_preserved(old,current),'all old main bytes including derive help must remain exact'
 opposed((SOURCE/PATHS[3]).read_bytes(),original[PATHS[0]])
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'ordered_non_doc_comparisons':12,'existing_main_including_derive_inputs_exact':True,'two_exact_show_layouts_only':True,'opposed_csv_and_help_mutations_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--self-test',action='store_true');args=parser.parse_args();run(args.self_test)
