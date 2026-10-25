"""CDC gate capture. No SUT launches on import; persist raw completion before metadata."""
import base64, datetime, hashlib, json, os, pathlib, subprocess, sys, uuid
ROOT = pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot')
SOURCE = ROOT / '.worktrees/public-api-documentation'
RECORD = pathlib.Path('/private/tmp/nms-slice30-cdc-attempts.json')
ENV_KEYS = ('TMPDIR','RUSTDOCFLAGS','CARGO_TARGET_DIR','CARGO_NET_OFFLINE','RUSTUP_TOOLCHAIN','TERM','PATH')

def utc(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def atomic_save(data):
    RECORD.parent.mkdir(parents=True, exist_ok=True)
    temp = RECORD.with_suffix('.json.pending')
    with temp.open('w') as f:
        json.dump(data, f, indent=2); f.write('\n'); f.flush(); os.fsync(f.fileno())
    os.replace(temp, RECORD)
def metadata():
    def git(*args): return subprocess.check_output(['git', *args], cwd=SOURCE)
    head=git('rev-parse','HEAD').decode().strip()
    diff_sha=hashlib.sha256(git('diff','HEAD','--binary')).hexdigest()
    untracked={}
    for raw in git('ls-files','--others','--exclude-standard','-z').split(b'\0'):
        if raw:
            relative=os.fsdecode(raw); body=(SOURCE/relative).read_bytes()
            untracked[relative]={'bytes':len(body),'sha256':hashlib.sha256(body).hexdigest()}
    identity={'head':head,'tracked_diff_sha256':diff_sha,'untracked':untracked}
    return {'head':head,'status':git('status','--short').decode(),
            'working_diff_sha256':diff_sha,'untracked_files':untracked,
            'working_identity_sha256':hashlib.sha256(json.dumps(identity,sort_keys=True).encode()).hexdigest()}
def channel(raw):
    return {'base64':base64.b64encode(raw).decode(), 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest()}
def capture(label, argv, overrides=None, kind='SUT gate'):
    overrides = overrides or {}
    data=json.loads(RECORD.read_text()) if RECORD.exists() else {'capture_owner':'CDC /root/cdc_arc06; independent execution','attempts':[]}
    entry={'id':uuid.uuid4().hex,'label':label,'kind':kind,'argv':argv,'cwd':str(SOURCE),
           'started_utc':utc(),'before':metadata(),'inherited_env':{k:os.environ.get(k) for k in ENV_KEYS},
           'overrides':overrides,'status':'running'}
    data['attempts'].append(entry); atomic_save(data)
    try:
        result=subprocess.run(argv,cwd=SOURCE,env=os.environ|overrides,capture_output=True)
        # Persist child completion FIRST. No git/summary/report parsing before this write.
        entry['raw_completion']={'exit_code':result.returncode,'ended_utc':utc(),
                                 'stdout':channel(result.stdout),'stderr':channel(result.stderr)}
        entry['status']='child-complete'; atomic_save(data)
    except BaseException as exc:
        entry['launch_or_capture_exception']={'type':type(exc).__name__,'message':str(exc),'utc':utc()}
        atomic_save(data); raise
    try:
        entry['after']=metadata(); entry['status']='complete'; atomic_save(data)
    except BaseException as exc:
        entry['post_metadata_exception']={'type':type(exc).__name__,'message':str(exc),'utc':utc()}
        atomic_save(data); raise
    print(json.dumps({'id':entry['id'],'label':label,'exit_code':result.returncode,
                      'stdout_bytes':len(result.stdout),'stderr_bytes':len(result.stderr),
                      'stdout_tail':result.stdout.decode(errors='replace')[-1800:],
                      'stderr_tail':result.stderr.decode(errors='replace')[-900:]}),flush=True)
    return result.returncode
if __name__=='__main__':
    label=sys.argv[1]; spec=json.loads(sys.argv[2])
    raise SystemExit(capture(label,spec['argv'],spec.get('overrides'),spec.get('kind','SUT gate')))
