"""Slice44 exact documentation contract and bounded original-M7.6 composition proof."""
from pathlib import Path
import base64,copy,hashlib,json,re,subprocess,sys
R=Path('/Users/oubiwann/lab/oxur/nms-copilot');S=R/'.worktrees/documentation-reconciliation';M=json.loads(Path('/private/tmp/nms-slice44-contract.json').read_text());BASE=M['baseline']
def git(*a):return subprocess.check_output(['git',*a],cwd=S)
def sha(b):return hashlib.sha256(b).hexdigest()
def baseline(path):return git('show',BASE+':'+path)
def expected():
 out={}
 for path in M['scope']:
  b=baseline(path)
  for z in M['changes']:
   if z['path']==path:
    old=z['old'].encode();assert b.count(old)==1;b=b.replace(old,z['new'].encode(),1)
  out[path]=b
 return out
def exact_source(observed):
 wanted=expected();assert set(observed)==set(wanted)
 for p,b in wanted.items():assert observed[p]==b,('out-of-contract bytes',p)
 return wanted

def known_warning(events):
 all_messages=[e for e in events if e.get('reason')=='compiler-message'];assert len(all_messages)==1,('whole diagnostic population',len(all_messages))
 e=all_messages[0];assert e['manifest_path']==str(S/'crates/nms-core/Cargo.toml') and e['package_id']=='path+file://'+str(S)+'/crates/nms-core#0.3.0'
 assert e['target']['name']=='nms_core' and e['target']['kind']==['lib'];d=e['message']
 assert d['level']=='warning' and d['code']['code']=='missing_docs' and d['message']=='missing documentation for a struct field'
 spans=[z for z in d['spans'] if z.get('is_primary')];assert len(spans)==1;z=spans[0]
 assert z['file_name']=='crates/nms-core/src/system.rs' and (z['line_start'],z['line_end'],z['column_start'],z['column_end'])==(38,38,12,25)
 assert z['expansion']['macro_decl_name']=='#[derive(rkyv::Archive)]' and z['expansion']['def_site_span']['file_name'].endswith('/rkyv_derive-0.8.15/src/lib.rs')
 return e

def reject(fn,value):
 try:fn(value)
 except (AssertionError,KeyError,TypeError,ValueError):return
 raise AssertionError('opposed control was accepted')

def scope(selftest=False):
 wanted=expected();exact_source(wanted)
 mutant=copy.deepcopy(wanted);p='crates/nms-graph/src/model.rs';anchor=b'self.bases.insert(base.name.to_lowercase(), base);';assert mutant[p].count(anchor)==1;mutant[p]=mutant[p].replace(anchor,b'self.bases.insert(base.name.to_uppercase(), base);',1);reject(exact_source,mutant)
 if selftest:
  print(json.dumps({'support_only':True,'twenty_synthetic_doc_paragraphs_plus_preview_accepted':True,'actual_insert_base_case_mutation_rejected':True}));return
 observed={p:(S/p).read_bytes() for p in M['scope']};exact_source(observed)
 assert set(git('diff','--name-only',BASE).decode().splitlines())==set(M['scope']) and git('ls-files','--others','--exclude-standard')==b''
 # Every Rust code fence and every non-doc line is already frozen by exact whole-file expected bytes.
 for p,z in M['source_files'].items():
  if p not in M['scope']:assert sha((S/p).read_bytes())==z['sha256']
 for p,z in M['retained_user_evidence_inputs'].items():assert sha((S/p).read_bytes())==z['sha256']
 print(json.dumps({'source':git('rev-parse','HEAD').decode().strip(),'exact_twelve_paths':M['scope'],'twenty_public_paragraphs':20,'all_other_source_and_examples_frozen':True,'actual_insert_base_case_mutation_rejected':True,'retained29_inputs_exact':True}))

def record(log,identifier):
 original=log.read_bytes();items=[e for e in json.loads(original)['attempts'] if e['id']==identifier];assert len(items)==1;e=items[0]
 assert e['status']=='complete' and e['cwd']==str(S) and e['before']==e['after']
 assert e['before']['head']==git('rev-parse','HEAD').decode().strip() and e['before']['working_diff_sha256']==sha(git('diff','HEAD','--binary')) and e['before']['status']==git('status','--short').decode()
 done=e['raw_completion'];assert done['exit_code']==0 and done['ended_utc'];channels={}
 for name in ['stdout','stderr']:
  c=done[name];b=base64.b64decode(c['base64'],validate=True);assert len(b)==c['bytes'] and sha(b)==c['sha256'];channels[name]=b
 return original,e,channels

def archive(log,identifier):
 original,e,channels=record(log,identifier)
 assert e['argv']==['cargo','doc','--workspace','--lib','--all-features','--no-deps','--offline','--message-format=json']
 assert e['overrides']['RUSTDOCFLAGS']=='-D warnings --force-warn missing_docs' and set(e['overrides'])=={'RUSTDOCFLAGS','CARGO_TARGET_DIR'}
 target=Path(e['overrides']['CARGO_TARGET_DIR']);assert target.parent==S/'target' and target.name in ['m76-cc-library-docs','m76-cdc-library-docs']
 events=[json.loads(line) for line in channels['stdout'].decode().splitlines() if line.strip()];finish=[z for z in events if z.get('reason')=='build-finished'];assert len(finish)==1 and finish[0]['success'] is True
 warning=known_warning(events);extra=copy.deepcopy(events);extra.append(copy.deepcopy(warning));reject(known_warning,extra)
 wrong=copy.deepcopy(events);next(z for z in next(v for v in wrong if v.get('reason')=='compiler-message')['message']['spans'] if z.get('is_primary'))['file_name']='crates/nms-core/src/player.rs';reject(known_warning,wrong)
 pages={}
 for rel in M['library_landing_pages']:
  p=target/'doc'/rel;b=p.read_bytes();assert b and b'<!DOCTYPE html>' in b and b'<title>' in b;pages[rel]={'bytes':len(b),'sha256':sha(b)}
 for rel in ['system/enum.ArchivedDiscoverySource.html','system/enum.DiscoverySourceResolver.html']:
  b=(target/'doc/nms_core'/rel).read_bytes();assert b'<!DOCTYPE html>' in b
  if rel.endswith('enum.DiscoverySourceResolver.html'):assert b'id="variant.Community"' in b and b'id="variant.Community.field.source"' in b
  pages['nms_core/'+rel]={'bytes':len(b),'sha256':sha(b)}
 assert log.read_bytes()==original
 print(json.dumps({'actual_build_id':identifier,'exact_generated_field_warning_count':1,'extra_warning_rejected':True,'wrong_origin_rejected':True,'library_root_and_module_page_count':70,'new_archive_pages_present':True,'html':pages},indent=2))

def composition(log,full_id,nodefault_id,cli_id):
 original,full,fc=record(log,full_id);assert full['argv']==['cargo','test','--workspace','--all-features','--offline'];assert b'test result: FAILED' not in fc['stdout']
 original2,nd,nc=record(log,nodefault_id);assert nd['argv']==['cargo','test','--workspace','--doc','--no-default-features','--offline']
 original3,cli,cc=record(log,cli_id);assert cli['argv']==['cargo','doc','-p','nms-cli','--bin','nms','--document-private-items','--all-features','--no-deps','--offline','--target-dir',str(S/'target/cli-docs')]
 assert cli['overrides']=={'RUSTDOCFLAGS':'-D warnings -D missing_docs'}
 modules=['completions','convert','export','find','import','info','list','route','saves','show','stats'];index=(S/'target/cli-docs/doc/nms/index.html').read_text()
 for mod in modules:assert 'href="'+mod+'/index.html"' in index and (S/'target/cli-docs/doc/nms'/mod/'index.html').is_file()
 # Current preview commands are actually represented by the library/CLI build argv. Other accepted29 user-documentation witnesses remain unchanged.
 assert (S/'CONTRIBUTING.md').read_bytes()==expected()['CONTRIBUTING.md']
 for p,z in M['retained_user_evidence_inputs'].items():
  b=(S/p).read_bytes();assert len(b)==z['bytes'] and sha(b)==z['sha256'];assert b==git('show','4a935408460349e9aa64fecd570d9960094191b1:'+p)
 # No host-save access, OS/game screenshot, color-theme, distribution, coverage or runtime-repair claim.
 summaries={}
 for key,channels in [('all_features_workspace',fc),('no_default_doctests',nc)]:
  text=channels['stdout'].decode();summaries[key]=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;[^\n]*',text)
  assert summaries[key] and all(f=='0' and i=='0' for _,f,i in summaries[key])
 assert log.read_bytes()==original==original2==original3
 print(json.dumps({'actual_workspace_test_id':full_id,'actual_no_default_doc_id':nodefault_id,'actual_isolated_cli_doc_id':cli_id,'workspace_harness_summaries':summaries,'cli_internal_modules':modules,'retained29_user_inputs_byte_exact':True,'original_M76_criterion_mapping':'README usage/captured-output images + CONTRIBUTING + all public library docs/key examples; exact core generated-field exception retained; higher original owners open'},indent=2))

if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='selftest':scope(True)
 elif mode=='scope':scope()
 elif mode=='archive':archive(Path(sys.argv[2]),sys.argv[3])
 elif mode=='composition':composition(Path(sys.argv[2]),*sys.argv[3:6])
 else:raise SystemExit('unknown mode')
