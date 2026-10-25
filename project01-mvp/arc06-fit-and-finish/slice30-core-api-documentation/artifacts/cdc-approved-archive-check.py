"""Verify only the Operator-approved exact generated resolver warning."""
import base64
import copy
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import sys

SOURCE = '/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/public-api-documentation'
HEAD = '40e6bc8f2b26606393931a232e604ce0e6f38e86'
TARGET = Path('/private/tmp/nms-slice30-cdc-approved-archive-09fd3092babf4c3893d0434e7bee8961')
ARGV = ['cargo', 'doc', '-p', 'nms-core', '--all-features', '--no-deps', '--offline', '--message-format=json']

def exact_diagnostic(events):
    messages = [e for e in events if e.get('reason') == 'compiler-message']
    assert len(messages) == 1, ('compiler-message population', len(messages))
    event = messages[0]
    assert event['manifest_path'] == SOURCE + '/crates/nms-core/Cargo.toml'
    assert event['package_id'] == 'path+file://' + SOURCE + '/crates/nms-core#0.3.0'
    assert event['target']['name'] == 'nms_core' and event['target']['kind'] == ['lib']
    message = event['message']
    assert message['level'] == 'warning'
    assert message['code']['code'] == 'missing_docs'
    assert message['message'] == 'missing documentation for a struct field'
    spans = [s for s in message['spans'] if s.get('is_primary')]
    assert len(spans) == 1
    span = spans[0]
    assert span['file_name'] == 'crates/nms-core/src/system.rs'
    assert (span['line_start'], span['line_end'], span['column_start'], span['column_end']) == (38, 38, 12, 25)
    assert span['expansion']['macro_decl_name'] == '#[derive(rkyv::Archive)]'
    assert span['expansion']['def_site_span']['file_name'].endswith('/rkyv_derive-0.8.15/src/lib.rs')
    return event

def must_reject(events):
    try:
        exact_diagnostic(events)
    except (AssertionError, KeyError, TypeError):
        return
    raise AssertionError('out-of-exception diagnostic was accepted')

class Anchors(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = set()
    def handle_starttag(self, tag, attrs):
        for name, value in attrs:
            if name == 'id':
                self.ids.add(value)

def main():
    log, identifier, target = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    assert target == TARGET
    original = log.read_bytes()
    records = json.loads(original)['attempts']
    selected = [r for r in records if r['id'] == identifier]
    assert len(selected) == 1
    r = selected[0]
    assert r['status'] == 'complete' and r['argv'] == ARGV and r['cwd'] == SOURCE
    assert r['before'] == r['after'] and r['before']['head'] == HEAD and r['before']['status'] == ''
    assert r['overrides'] == {'RUSTDOCFLAGS': '-D warnings --force-warn missing_docs', 'CARGO_TARGET_DIR': str(target)}
    done = r['raw_completion']
    assert done['exit_code'] == 0 and done['ended_utc']
    decoded = {}
    for channel in ('stdout', 'stderr'):
        c = done[channel]
        body = base64.b64decode(c['base64'], validate=True)
        assert len(body) == c['bytes'] and hashlib.sha256(body).hexdigest() == c['sha256']
        decoded[channel] = body
    events = [json.loads(line) for line in decoded['stdout'].decode('utf-8').splitlines() if line.strip()]
    finished = [e for e in events if e.get('reason') == 'build-finished']
    assert len(finished) == 1 and finished[0]['success'] is True
    warning = exact_diagnostic(events)
    extra = copy.deepcopy(events)
    extra.append(copy.deepcopy(warning))
    must_reject(extra)
    other = copy.deepcopy(events)
    wrong = next(e for e in other if e.get('reason') == 'compiler-message')
    next(s for s in wrong['message']['spans'] if s.get('is_primary'))['file_name'] = 'crates/nms-core/src/player.rs'
    must_reject(other)
    pages = {}
    for relative in ('index.html', 'system/enum.ArchivedDiscoverySource.html', 'system/enum.DiscoverySourceResolver.html'):
        path = target / 'doc/nms_core' / relative
        body = path.read_bytes()
        assert body
        parser = Anchors()
        parser.feed(body.decode('utf-8'))
        if relative.endswith('enum.DiscoverySourceResolver.html'):
            assert {'variant.Community', 'variant.Community.field.source'} <= parser.ids
        pages[relative] = {'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest()}
    assert log.read_bytes() == original
    print(json.dumps({'source': HEAD, 'actual_build_id': identifier, 'exact_warning_count': 1,
                      'extra_warning_rejected': True, 'different_origin_rejected': True,
                      'html': pages, 'source_annotation_repair_claimed': False}, indent=2))

if __name__ == '__main__':
    main()
