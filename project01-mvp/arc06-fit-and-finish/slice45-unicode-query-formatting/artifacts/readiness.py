from pathlib import Path
import base64,datetime,hashlib,json,os,subprocess,tempfile
S=Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/documentation-reconciliation')
def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(b):return hashlib.sha256(b).hexdigest()
def channel(b):return {'base64':base64.b64encode(b).decode(),'bytes':len(b),'sha256':sha(b)}
home=Path(tempfile.mkdtemp(prefix='nms-unicode-readiness-'));binary=S/'target/debug/nms';seed=(S/'data/test/save56_minimal.json').read_bytes()
cases=[('find','123456789012345678ééééé','1234567890123456ééééé'),('route','12345678901234567ééééé','Planet')]
out={'created_utc':utc(),'source':str(S),'binary':str(binary),'binary_sha256':sha(binary.read_bytes()),'seed_sha256':sha(seed),'home':str(home),'cases':[]}
record=home/'receipts.json'
def save():
 with record.open('w') as f:json.dump(out,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
save()
for label,system,planet in cases:
 data=json.loads(seed);records=data['DiscoveryManagerData']['DiscoveryData-v1']['Store']['Record'];assert len(records)==2
 assert [r['DD']['DT'] for r in records]==['SolarSystem','Planet']
 records[0]['DM']['CN']=system;records[1]['DM']['CN']=planet
 path=home/(label+'.json');path.write_text(json.dumps(data,ensure_ascii=False)+'\n')
 argv=[str(binary),label,'--save',str(path)]
 if label=='route':argv+=['--target',system,'--algo','nn']
 e={'label':label,'argv':argv,'cwd':str(S),'HOME':str(home),'fixture_sha256':sha(path.read_bytes()),'started_utc':utc()};out['cases'].append(e);save()
 r=subprocess.run(argv,cwd=S,env=os.environ|{'HOME':str(home),'XDG_CONFIG_HOME':str(home/'config'),'XDG_CACHE_HOME':str(home/'cache')},capture_output=True)
 e['completion']={'exit_code':r.returncode,'ended_utc':utc(),'stdout':channel(r.stdout),'stderr':channel(r.stderr)};save()
assert all(e['completion']['exit_code']==101 and b'is not a char boundary' in base64.b64decode(e['completion']['stderr']['base64']) for e in out['cases']),record
print(json.dumps({'record':str(record),'cases':[{'label':e['label'],'exit':e['completion']['exit_code'],'stderr':base64.b64decode(e['completion']['stderr']['base64']).decode()} for e in out['cases']]},ensure_ascii=False))
