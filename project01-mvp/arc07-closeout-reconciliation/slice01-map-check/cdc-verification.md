# Slice01 — independent CDC verification

2026-10-25. CDC `/root/cdc_arc07`, requested/applied GPT6.1 Sol High, independently reviewed CC `/root/cdc_arc07/cc_arc07` (requested/applied GPT6 Luna High), planning return e2cd88e, unchanged clean source fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc. Verdict: MAP-01–03 accepted; zero formal refinements. This is bounded native map acceptance, not original D0042 or Project01 acceptance.

Inspected the full issued prompt/open set, map result, close report, all three rows, decoded successful PTY bytes and metadata, actual source map entry/guard/input/renderer/state and REPL dispatch. Successful CC transcript independently decodes to14103 bytes with SHA256 a4cc472cebab147f42aa73c8f4ca9b9c39191694476804288b1fc2d1c13dbb47. Actual bytes contain one alternate enter/leave, eight cursor-show sequences, Galaxy/Region/Local/help and ordinary REPL help/Goodbye. The false map_seen bookkeeping flag is explicitly qualified; bytes establish the actual map observation. Initial unanswered CPR failure and pre-build observation remain preserved, not promoted to source-qualified proof.

## Fresh independent native observations

With the CC-built current-source binary, temporary HOME/config/save copy, watcher/cache disabled and owned loopback listener, CDC directly used Python stdlib PTY/ioctl and answered CPR queries. The first run observed real Galaxy entry; Right/Down output; Region then Local; help/dismiss; resize100×32→90×28 with full redraw; zoom back; q with alternate leave/cursor show; ordinary `status` returning Euclid (0;Normal),20systems/13planets and the next prompt. Reedline-active lflag was65 (0x41). That run's final sampling after child exit raised termios ENOTTY because the controlling PTY had hung up. The tool returned that actual failure; it is not a successful after-exit sample, and no raw transcript file was written for that failed capture.

The single proportionate correction authorized by root's scope guard kept an owned shell alive after the binary exited and used `stty -g` at that boundary. It again observed the map, one alternate enter/leave and cursor show, `CHILD_EXIT=0`, and BOUNDARY_READY. Full termios before/after compared equal, including control characters and speeds. Both stty readings were:

```text
gfmt1:cflag=4b00:iflag=2b02:lflag=5cb:oflag=3:discard=f:dsusp=19:eof=4:eol=ff:eol2=ff:erase=7f:intr=3:kill=15:lnext=16:min=1:quit=1c:reprint=12:start=11:status=14:stop=13:susp=1a:time=0:werase=17:ispeed=9600:ospeed=9600
```

Actual temporary shell-boundary transcript: `/private/tmp/arc07-cdc-shell-_rsfdc9x/actual-pty.bin`,5185bytes; this record retains the direct result rather than adding another capture framework or durable transcript family. The independently observed leave sequences, continued REPL use and shell-boundary exact restoration support the ordinary lifecycle. CC additionally captured the brief cooked-mode sample at map leave before Reedline resumed; that sampling timing remains CC-attested, checked against source/bytes, not falsely attributed to CDC. No public run_map probe or further terminal attempt was needed.

## Row walk and retained evidence

- MAP-01 DONE, reproduced: independent ordinary interaction above agrees with CC bytes and source semantics. PTY output changes are observed; no reconstructed exact-cell screen or broad terminal claim.
- MAP-02 DONE, reproduced/reconciled: actual alternate leave/cursor show, continued REPL, shell-boundary exact termios equality/exit0, plus accurately scoped CC map-boundary sample. Reedline raw mode is separately identified. Existing Slice48 guard/input error proof applies at the identical source/dependency revision; native happy-path checks do not replace its injected failures.
- MAP-03 DONE, reproduced: source status is empty atfb2ecebb; e2cd88e changes exactly four assigned planning paths; main's known helper blob remains2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e; old49 remains unissued/untracked (SHA256 d9600d9cddf3f9583fc1768cd67644b1e32ec6926b8f1f298b2982289de30986). No source/test/coverage/main/stack mutation or parent waiver occurred. Two required commit trailers parse once.

## Bubble-up and limits

All three rows and the assigned native usable result are delivered. Artifacts are exactly map-result.md and the single transcript container; canonical closing documents are separate. Original error-path proof is retained from `project01-mvp/arc06-fit-and-finish/slice48-map-terminal-guard/cdc-verification.md` atfb2ecebb. This slice does not establish live updates while the map is open, terminal OS-failure/abort guarantees, an exact rendering oracle, original D0042 completion,95%coverage or any release gate. Main's long-held model read guard is a source-grounded inventory finding; it does not require changing this arc's first-three-slice scope. Arc plan gets routine result status only; no structural amendment is needed before02.
