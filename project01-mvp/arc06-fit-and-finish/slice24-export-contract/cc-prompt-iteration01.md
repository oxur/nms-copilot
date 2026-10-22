# Slice24 iteration01 — consolidated held test/docs correction

2026-10-22 UTC. First of five ordinary refinements, no reset/exception. Same standing Autonomous Project01 Arc06 ONLY, CDCgpt-6.1-solHigh/soleexistingCCgpt-6-lunaHigh. Immutable initialprompt2401314 remains binding/history; same exactTHREEpaths/productionfreeze/allpriorassertions/fulloriginalM7.1scope/gates. No new case population/dependency/runtime/API/schema/native/inquiry/model/role/integration/release/otherarc/Project02authority.

## Pin, intake and ownership

Full initialthreepath/767line helper/assertion/error/README walk complete, no runtime released/productiondefect established. Source `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/export-contract`, branchcodex/export-contract, HEAD2f2814c780f5022dea3bc1643d99828ba679e8d0; tracked diff e94040cee74c877bcc79926dd8636db6d9e4fb589e7a04284b6a319d7d88c2ee, newtest26372B/SHA48d67383c9b49862cc952a0fa72ef3953ad851f76af874634ba060c612f86436; composite14272ed7d6a6d3537d67c04e7acddce6b24dd5833745640b5cc09201d4cac75d. Actual lastformat89f00c66/lint9c3c425d bothcomplete0/fullstatuses/stablepin. Preserve all eight actualcapture entries, originalpair/afterpairchange/redunusedhelper9941713c/selfrepair chronology. No acceptance of earlierpair for laterbytes.

Required-full THISprompt, activeinitialprompt/plan/ledger/allowlist/currentexecution and all affected currenthelpers/calls/assertionsequences. Same sourceAGENTS/framework/verification/testing/fullRust+AP/CLI guidance retained. Revalidate actualpin/index/noactiveprocess/mainb616 protectednativeblob2cd450ce/allheldexternalpins. Absolute threepathmanifest beforeeverypatch; explicitcwd, no mixedrootwrites. CDC owns planning; CC source/index/gates after full attributable readback. Implement ONLY F1–F6 below, preserving valid existing assertions/constants/goldens/cases; local helpernames discretionary. If another material issue emerges, report, do not opportunistically widen. Captureformat→lint then IMMEDIATEHOLDsource/index/gates without post-pair edits; CDC settleddelta review before originalruntime release.

## F1 — independently specified Anchor distances must reach actual expected rows

The INITIALprompt table already correctly states B1(0),A1/A2/A3(2000),C1(2400) for Anchor. Current test passes current-origin constants instead; this is a wiring miss, not author-literal retraction. Current A/B/C literal positions and Euclidean400ly-pervoxel give these values; same-systemB0 is0. Keep original A1/A2/A3/B1/C1 unchanged for all current-origin cases. Introduce explicit copied Row variants (const or local) using struct update syntax, for example `Row { distance: 0.0, ..B1 }`, `Row { distance: 2000.0, ..A1 }`, matchingA2/A3, `Row { distance: 2400.0, ..C1 }`. FromAnchor fullpair receives [BAnchor,A1Anchor,A2Anchor,A3Anchor,CAnchor]; fromAnchor+within0+nearest1 receives [BAnchor]. Allother11fields/ordering/duplicate multiplicity and BOTHJSON+CSVchecks retained. No production distancehelper computesexpected.

## F2 — actual wrapper-specific writer error inspection

Cargo.lock pins serde_json1.0.149 andcsv1.4.0. Read exact localprimarysource `/Users/oubiwann/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.149/src/error.rs:150–161,388–397`: `io_error_kind()` returns actualcontainedioerror.kind, while Error::source forIo delegates to `err.source()` and skips io::Errorwrapper. Thus current recursive downcast loop cannot find the simple-message io::Error from JSON serialization, although production properly propagates failure. No testweakening/source-chain claim or productionrepair.

In export.rs TESTMODULEONLY retain failwriter writes>0 and unwrap_err. Inspect the actual boxedroot error using:
- `downcast_ref::<serde_json::Error>().and_then(|e| e.io_error_kind())`, or
- direct `downcast_ref::<io::Error>().map(|e| e.kind())`, or
- if actualcsvwrapper, `downcast_ref::<csv::Error>()` then match `.kind()` as `csv::ErrorKind::Io(e)` andtake`.kind()` (pinnedcsvsrc/error.rs25–48).
Assert the observedkind is EXACTSome(io::ErrorKind::BrokenPipe) and error display retains failwriter sentinel `closed test pipe`; bothwriter tests must actuallyattemptwrite and fail. No genericis_err-only/errorstring-only substitution. A small helper for these actualroot cases is allowed; no source()-walk assumption necessary. Productionwriters and theircalls/imports untouched, all11oldunit tests preserved.

## F3 — default/uppercase actual full rows and source immutability

Current uppercase tests compare twochildren'sstatus/stdout without checking independentrowvalues or eitherstderr; default only compares twochildren and nevercalls its ownimmutabilitycheck. Keepexistingpositive byte-equivalence/defaultnewline claims, but BOTH uppercaseJSON andCSV actualoutputs must pass existing `assert_json`/`assert_csv` (which require actualsuccess/empty stderr) against literal[A1,A2,A3,B1,C1]. Do not deriveexpectedrowsfromanotherchild. Lowercasecomparison if retainedmustalsoassertsuccess/stderrempty. For omittedformat, call `child.assert_unchanged()` immediatelyafterchildcompletion andbefore movingOutputfields; inspect actualfullJSONarray against thoseindependentrows, retain `[\n`start/newline and equivalence to explicitjson. Ownership viable: Output canbecheckedbyrefbeforeconsume or clone itsbytevec; do not use movedvalue. All existing empty independentfixture/filterempty/nearest0 cases remain unchanged. No newcasesbeyondissued variants.

## F4 — finish exactly the already-issued meaningful negatives

Retain actualchild positivebaseline and all existing12field/type/missingextra/reorder/drop/leak/CSVcontrols. Add the absent explicitcases by CLONING actualobservedbaseline, through SAMEexistingpredicates:
- JSON extradupe: push observed[0].clone withoutremovinganything; json_validates false. Existingdroppeddupe remains.
- JSON wrongplanetprojection: only portal_glyphs ofrow0 to literal `100100000000` (P1 ratherthanP0), false.
- JSON actualwrongentityalias: separately change system_name to `Nine Decoy` and galaxy to `Eissentam`, false for each, retaininggenericfieldnegatives.
- CSV headerreorder: cloneactualbytes, swap first/thirdheadernames while all bodybytes remainexact; `csv_payload_validates` false. ExistingheaderXcase stays.
- CSV malformedescaping: cloneactualbytes, remove the closingquote at first `Moon",Lush` occurrence (offset4 within thatneedle), preserve otherbytes; assert the actualpayloadpredicate rejects and bytes differfromliteralpositivegolden. Existinggoldenequality/decodedwholevectors/multiplicity/realchildstatus remain; no csvwriter usedtobuildgolden/mutant. This rejects damagedquotednewline serialization, not justcount/shape.

Existingreordered row0↔3 ismeaningfullydifferent; A1↔A3 wouldnotbe. No exhaustive mutation framework/extra fixtures, no weakened literal fields/keys/order/multiplicity.

## F5 — accurate existing export documentation

READMEexportsection ONLY currently says “Errors go to stderr without a partial data payload.” Directserialization can fail afterwritingbytes, so that universalIOatomicity isunsupported. Qualify EXACTclaim as “Input and query errors go to stderr before a data payload is written.” Keepactualread-onlysource/zeroCSV/sharedfilter/namedPLANETS/basegalaxy/systemP0/JSONdefault descriptions and unchangedimportline/remainder. As already requested in initialREADMEscope, identify default currentgalaxy selection and list the existingtwelve flat fieldnames in order using the exactHEADER names; no extraformat/field/policy promise. Fullaffectedparagraphwalk; no broader errorpolicy/APIchange. Initialpromptdidnotauthorize universalatomicstdoutforallIOerrors; no suchproductionguarantee isadopted.

## F6 — repository test naming

SourceAGENTS test-naming convention is `test_<fn>_<scenario>_<expectation>`. Rename ONLY the four new `#[test] fn export_...` functions to corresponding `test_export_...` names; preserve all bodies/assertions/cases. Existingoldtestnames/helpers untouched. This is same-file project convention, no new test population.

## Records, capture, gates and stop

Appendreadback/currentassignment/F1–F6 mapping and sourcechronology to currentignoredexecution; preserveinitialhistory/JSONentries. Helper--help interfaceinspection wasnoncounting beforeSUT, unknownnestedexit/channel/toolobject notinferred. Exactotherincidentalcall usedrglineinventory&&two gitdiff--checks, wrappertext(r.output); displayedrglist retained, missingnestedexit/separatestderr/fullobjectunknown. These wereinitialread-onlyinventory, no compiler/test/capturedgatecredit and notrequiredfinaldiffobservations; preserveoriginalpurpose/limits. DoNOToverwriteoriginalclaim ofemptydisplay: clarifyfulloutputwasrglistwhilegitdiffportionsprintednothing. Finalrequiredworking/baselinediffchecks muststillrun via kernel after runtime. No newwaiver/attemptsuccessfabricated.

Beforefirstnewactualgate usefullproven `/private/tmp/nms-slice24-cc-capture.py`/existingactualrecord, no directhelperprobes. Rawcompletion savedFIRST, complete before/afterHEAD/status/diff/untrackedidentity/env/originalTMPDIR/UTC/channels andfulloutertoolobjectstoredbeforepostprocess; eachfreshUUIDimmutable. Captured makeformat→makelint, compiler selfrepairs onlyexactscopewithrecords andrenewedpair; immediatelyHOLDallsource/index/gates, noproductruntimeuntilCDCexplicitrelease. Do not make anotherpost-pairedit. Existingoriginalrequiredorderedfocusedexport→wholeCLI→query→fullworkspace/build/doctests/strictCLIdocs/diffs stayidentical; no newnative/broadproofpopulation. Original11tests/productionprefixfrozen. Namedpathcommits/trailers/return/independentacceptance/retro/bubble-up andallremainingoriginalhigherArc06gates staybinding. Firstof5ordinarypasses, noimplicitnextpass/Operatorexception.
