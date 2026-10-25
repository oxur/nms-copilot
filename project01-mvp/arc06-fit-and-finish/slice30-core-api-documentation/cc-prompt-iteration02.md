# Slice30 correction02 — complete the issued docs worksheet

2026-10-25 UTC. FIRST actual refinement1/5, four remain. Initial+01 first implementation is preserved. Same CC6-lunaHigh/CDC6.1-solHigh, dedicated public-api-documentation baseline4a935408, nine-file outer allowlist; executable/API/tests/attributes/tables/Cargo/other paths frozen. Only eight source files may change for this correction: address/biome/delta/discovery/galaxy/glyph/player/system.rs. lib.rs is reviewed and frozen. No new product scenario or API; this closes demonstrated original worksheet omissions and an AUTHOR scope contradiction.

## Preserved state and author correction F1

CDC read the complete nine-file initial delta (554 insertions/50 deletions), all source production portions and each affected helper. Initial format30df48746b104e9f9a21df27f0b9c75e actual0/full107B stdout/0stderr/complete statuses/equal workingidentity556f6eada8ee83f809f5a5c6bd7a2ad321c084137c09452ef4b9205c1ff9fa0e, diff63f1a976cc6aed51bd5d83fac04de0fb1d1f37aef7eb48bf0ef06623cd267691, occurred just before hold arrived. No lint/runtime ran; source/index/gates held. Retain original attempt and messages, never recast it as post-correction proof.

Original exact non-doc LINE equality was incompatible with documenting inline tuple payload fields. Explicitly permit ONLY whitespace/layout and a trailing field comma inside these SIX exact declarations:

- biome::BiomeParseError(pub String)
- discovery::DiscoveryParseError(pub String)
- galaxy::GalaxyTypeParseError(pub String)
- glyph::GlyphParseError(pub String)
- player::BaseTypeParseError(pub String)
- system::DiscoverySource::Community { source: String }

Names, visibility, field count/types/order, derives/attributes and every other non-doc token remain exact. No other declaration/body normalization. This is an author contract repair, not a production defect or human policy exception. Five-only earlier wording was incomplete: Community.source is the sixth layout site. Preserve original constraints and all valid docs additively.

Scope-audit recipe: remove ONLY complete //!/ /// lines and blank lines. In each appropriate file canonicalize ONLY its known declaration using an exact-name/type anchored match allowing whitespace and optional trailing field comma; replace it with its original one-line spelling. For the five errors the exact remaining tokens must be `pub struct NAME ( pub String [,] ) ;`; for Community exact remaining tokens `Community { source : String [,] } ,`. Compare all remaining ordered bytes with baseline after that same normalization. Do not globally remove whitespace, punctuation, comments or string-literal contents. Require exactly the expected declaration match in its file. All other paths byte-exact; existing synthetic executable module-declaration mutation still must be rejected. This is line-content evidence plus full review, not a Rust AST proof.

## F2 — restore the already-issued independent literal examples

Current address example covers packed/signal formatting only; required reverse/galaxy-loss/distance cases are missing. Extend one real address example with `?`/hidden typed Ok, retaining A=new(3,0,0,1,2,9), literal0x2001_0000_0003 and signal0802:007F:07FF:0001. Assert reverse from that literal signal with P2/GG9 equals A; A.to_portal_address().to_hex_string()=="200100000003" and portal roundtrip reality_index0. Samevoxel SSI2/P2/GG9 distance264.6828728 (abs difference <1e-9), neighborX4 otherwise same400.0, sameXYZ/SSI1/P3/GG9 distance0.0. Literal expectations must not be generated from the method being demonstrated. No save56 fixture/new files.

In the existing system module example retain A and add exact SystemId system_address0x0001_0000_0003/Display"9:000100000003"; opposed samepacked/GG0 SystemId differs. This demonstrates the already-required exactidentity vs legacycomparison distinction; no graph/live scenario.

In existing glyph example retain index13/nameRocket and assert hex'D' plus Glyph::try_new(16)==None. Existing prefix Rocket0/rest0 example stays intact. In existing delta example retain empty count0 and then set PlayerMoved from A to X4 and assert !is_empty/count1; imports can use existing actual core types. Preserve existing DiscoveryNames set/clear/absent example unchanged. No new examples for every trivial field, no unrelated cases.

## F3 — complete actual Errors/Panics and section ordering

Apply loaded Rust documentation DC04–06, not compiler-warning count alone. Keep valid comments and attach useful failure sections to actual custom public parser methods/impls: Biome/BiomeSubType/Discovery/GalaxyType/BaseType FromStr need # Errors spelling accepted names/aliases, no trim, unmatched input returning the respective original-string error. Existing Glyph/GalacticAddress FromStr Errors remain; Portal FromStr must explicitly document propagated wrong-length/invalid-glyph and existing UTF8 diagnostic panic through parse_mixed. Inherent GalacticAddress::from_portal_string also needs the propagated # Panics. Do not invent errors for infallible constructors/lookups. Standard formatter/serde inherited trait boilerplate does not require a new docs project; preserve their behavior and existing identity docs.

Document actual panics at Glyph::hex_char/name/emoji/abbrev on an out-of-range deserialized index, and canonical # Panics on From<u8> >=16. Glyph::index merely returns the stored index; qualify its 0..15 wording rather than claiming unchecked deserialization enforces it. PortalAddress::to_emoji_string/to_abbrev_string panic for invalid stored values; add their # Panics. to_hex_string formats stored values without validating them, so qualify twelve-digit output for valid nibble values instead of falsely asserting it validates or panics. Type-level unchecked-deserialization warnings stay.

Order new sections Examples→Errors→Panics (current parse_next_glyph and parse_mixed have Examples last); move existing useful examples, do not drop assertions. Keep summaries to one short line before a blank/detail paragraph (e.g. WrongLength has a single summary then sentinel13 detail). No change to actual grammar, + qualification, Unicode behavior or bodies.

## Readback, pair, hold and unchanged proof

Read full02; record bounded F1–F3 readback/current source pin and author-vs-implementation attribution before edits, then proceed directly under current authority. Absolute dedicated paths AND current eight-path correction manifest; main exact protected blob/index/pins checked pre/post. lib and all non-doc source frozen, no planning writes.

Capture renewed make format→make lint through the supplied inspected raw-first kernel, every actual attempt retained. Then HOLD source/index/gates for targeted complete F1–F3 review. No runtime yet; original default/archive core tests+strictdocs/scopeaudit/diffs remain unchanged and will be released only after this settled review. No broad workspace/native proof or discretionary new scenarios. Return chronology preserves initial format and actual before/after identities. First actual refinement only, four remain; no waiver/reset or further automatic corrective authority.
