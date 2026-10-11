# Traveller cultures and exploration analysis

Version 1.0 — 2026-10-11. Design for Arcs08/09, with identity/observation dependencies in Arcs01–03. Requirements below come from the Operator's discussion; geometric methods are research candidates, not adopted algorithms. No candidate-system source or exact discovery-completeness measure has been demonstrated.

## Confirmed domain semantics

A system's discovery by a particular traveller is the atomic claim. Visiting it, writing about it or establishing a base creates that visitor's experience, not another claim. Two travellers' adjacent or interleaved discoveries form distinct cultures even when both are played by the same human. Each save retains a distinct narrative concerning shared worlds. Do not conflate a system discovery with the discovery of every planet in it.

A culture develops spatial definition as that traveller's discoveries accumulate. The user may name it and choose its collective noun (empire, culture, cluster, or arbitrary text), describe its founding world and history, and attach notes/images. Bases and their counts are useful indicators of attachment; no new manual claimed/cherished flag or mandatory sentimental score is needed.

The user may establish multiple cultures with the same traveller and later learn they adjoin. Preserve their identities and histories when geometric components connect. Record when Copilot observed the connection separately from any evidenced in-game discovery time. A single linking system may be a narrow bridge; later discoveries can broaden the connection. Do not silently merge names, journals or cultures. An optional future explicit unification can preserve predecessor identities, but automatic unification is not part of this design.

**Superseded conversational proposals:** manually assigning discovery ownership as ordinary collection membership, a culture spanning multiple travellers' claims, user-configured neighborhood distance, and ship warp range as a culture-definition parameter. Generic collections and prospective exploration regions remain available, but neither changes who discovered a system. A multi-culture comparison is not a merged culture.

## Identity, attribution and authority

A stable `CultureId` identifies authored name/noun/founding references/history. Its originating traveller links to a logical save lineage, not a filesystem path or display name. Linked synced replicas represent the same lineage; divergent copies need an explicit identity decision. Each narrative entry still retains its save/context and author.

A native owner token may identify an account shared by several saves, not the save that performed a discovery. G10 must establish what the actual fields support. Do not attribute every record in a save to that save's traveller, or assign all of an account's claims to every traveller. Preserve unresolved owner-to-traveller associations. User-supplied attribution can be recorded as such, without relabelling it as independently verified game evidence. Conflicting owner observations remain visible claims about one discoverer, not multiple authoritative discoverers.

Derived analyses retain galaxy/world identity, population generation, algorithm/version, ownership resolution and input coverage. Culture identity, founding/history and authored annotations survive re-analysis. A corrected owner attribution may change computed affiliation; record that change and preserve earlier provenance rather than rewriting history invisibly. Coordinate-identical systems in different galaxies remain distinct and have no geometric edge between them.

## What the mathematics must explain

The desired result is a degree of structure, not an arbitrary distance cutoff. Measures should distinguish:

- **Scale:** number of distinct affiliated systems and spatial extent.
- **Completeness:** how much of a well-defined bounded system population belongs to the traveller, including interior gaps.
- **Cohesion:** strength/shape of spatial connection, including a solid interior versus a thin chain or bridge.
- **Evidence coverage:** how much of that population/ownership/position is actually known. This qualifies all other measurements and is not itself territorial completeness.

No numeric weights, combined score or boundary formula are adopted here. The following are acceptance constraints for selecting a method, not claimed mathematical results:

| Fixture | Expected interpretation |
|---|---|
| One discovery | Isolated founding point; no collective structure yet. A named historical seed can still exist. |
| Two neighboring discoveries, same traveller | Emerging connection, much smaller scale than a developed territory. |
| The same pair with intervening/nearby non-affiliated systems | Weaker cohesion/completeness relative to the unobstructed pair, with the reason exposed. Nearby discoveries by the same traveller must not be mistaken for interference. |
| A compact ball of 100 systems, all affiliated, with its bounded population fully known | Maximum completeness for that population and a cohesive interior. No claim that no other stars exist outside the evaluated boundary. |
| The same boundary/population with interior non-affiliated systems | Lower completeness reflecting the interior gaps. Hold the population fixed for this comparison so shrinking the boundary cannot hide holes. |
| Unknown ownership on interior systems | Report bounds/unknowns; do not classify them as definitely undiscovered or foreign-owned. |
| Delete observations without changing the synthetic universe | Lower knowledge coverage, not improved completeness or invented empty space. |
| Solid ball versus a thin chain of equal count | Both may be connected; geometry should expose the difference without count alone deciding cohesion. |
| Two dense concentrations linked by one same-traveller discovery | Detect the connection and its narrowness; preserve separately named culture histories. |
| Adjacent/interleaved different travellers | Separate affiliations and narratives, visible shared frontier; no automatic culture merge. |
| Duplicate snapshots, renamed systems, reordered inputs | Same unique population/analysis; history and attribution remain stable. |
| Same voxel with different system indices; coplanar/collinear/tied positions | Preserve distinct entities; handle degeneracy explicitly, no fabricated positional jitter or arbitrary nearest winner. |

These examples do not uniquely determine a mathematical model. G12 must choose boundary handling, local density normalization, scale selection and tie/degeneracy behavior with explainable outputs. Stability under translation, rotation (within precision), changes of units and insertion order should be checked. Sampling loss, peripheral outliers, thin tendrils and same-traveller populations separated by a large void must not produce spurious solid empires. Evaluate on independent fixtures beyond the illustrative examples before adoption.

## Geometric research direction

Investigate natural spatial adjacency using 3D Voronoi/Delaunay constructions over a representative population of surrounding systems, then apply ownership labels. Delaunay's empty-sphere property supplies a geometric adjacency construction without a user radius, but it is not itself a culture classifier: it can connect distant points and is sensitive to missing population and boundary effects. Include non-affiliated systems when known; triangulating only a traveller's discoveries conceals intervening structure. [CGAL triangulation reference](https://doc.cgal.org/Manual/latest/doc_html/cgal_manual/Triangulation_3/Chapter_main.html).

Investigate alpha-shape families or another multiscale method for boundaries, gaps and stable connections. Alpha shapes have a scale parameter; examine data-derived transitions/stability rather than expose that parameter as a required culture radius or claim the method is parameter-free. The method must still justify how it chooses/report scales. Convex hulls alone can fill vast unobserved interiors, while connected components alone can join every outpost through a sparse chain. [Edelsbrunner and Mucke, Three-dimensional alpha shapes](https://arxiv.org/abs/math/9410208), [CGAL alpha-shape manual](https://doc.cgal.org/latest/Alpha_shapes_3/index.html).

CGAL is a mathematical/implementation reference, not an approved C++/FFI dependency. Before implementation, compare Rust 3D candidates for robust predicates, degeneracies, coordinate precision, license/MSRV/build cost, reproducibility and bounded resource usage; do not choose a 2D-only crate for a 3D problem. Reuse `petgraph` for connectivity/path algorithms and `rstar` for appropriate proximity work; neither supplies culture semantics by itself. DuckDB can assemble populations and compute aggregates/joins; validate 3D suitability before using any spatial extension function. No geometry dependency is added in this pass.

## Candidate systems and discovery-state research

Inventory actual save fields, reference implementations and any supported local data sources for nearby unvisited systems. Investigate procedural reconstruction only as a hypothesis. Record source/version, extraction method, universe/galaxy scope, licensing/access limits, population coverage and reproducible game witnesses. No cloud API, memory reader, OCR pipeline or platform adapter is implicitly authorized by this design; an additional acquisition mechanism requires its own scoped decision.

Keep these facts independent: system existence, position/precision, observed discoverer, save/traveller visit evidence, and Copilot observation. A missing local record cannot establish globally undiscovered status. An apparent game “undiscovered” status has a source/time and does not become a timeless guarantee. A system known from another save may be unvisited by the active traveller but is already someone else's discovery. Copilot cannot infer unrecorded stars merely because two discoveries surround a gap.

Coordinate identity is not necessarily exact geometry. Establish whether existing voxel coordinates provide sufficient resolution and how actual inter-system LY positions can be obtained. Several system indices may share one voxel. Do not order tied/coarse candidates as exact nearest, infer a star's existence from a numeric address, or call voxel separation an independently measured in-game distance. Geometry returns insufficient-data/approximate results where required.

G11 should run early, alongside Arc02/03 research. Outcomes must state which capabilities are supported: known-population analysis, enumerated candidates, spatial precision, ownership state and demonstrated completeness. If full enumeration cannot be supported, retain it as an unresolved capability for Operator disposition with a re-entry condition. A limited direction/volume suggestion is useful, but does not silently satisfy a promise to list all undiscovered systems.

## Queries and exploration objectives

Proposed query vocabulary, not final command grammar:

```text
culture show "Quiet Reach"
culture systems "Quiet Reach"
culture analyze "Quiet Reach"
culture frontier "Quiet Reach"
culture gaps "Quiet Reach"
culture next "Quiet Reach" --from current
culture compare "Quiet Reach" "Distant Garden"
```

Support inventory, interior gaps, corridors between concentrations, outward expansion and neighboring-culture comparisons. List affiliated discovered systems separately from other systems within an analyzed territory. “All systems” always names the population/scope/coverage basis. Nearest-outward must distinguish distance from the calculated frontier, distance from the current traveller, and reachable route distance under a specified warp range. Warp range constrains travel, not culture definition. Cross-save reads do not switch the active save or journal author.

Every recommendation identifies its target or search volume, galaxy, evidence/discovery status, coordinate precision, objective, distance basis and explanation. Return ties and insufficient-data states honestly. Reproduce rankings against brute-force expected distances on fixtures and independently witnessed game candidates where the source supports them. Reuse Project01's accepted galaxy-scoped routing behavior; do not create a parallel route implementation in SQL.

## NMS regions and terminology

Store observed game-region names/identifiers separately from personal culture names. One culture may overlap multiple game regions; one region may contain several travellers' cultures. Names are not globally unique region keys. Region derivation requires validated inputs/version/provenance and comparison against in-game witnesses across galaxies/boundaries.

The independent [nms_namegen project](https://github.com/stuart/nms_namegen) is a lead for deriving region names from coordinates and galaxy; its claims are not yet independently reproduced here and do not enumerate undiscovered systems or establish their owners. Preserve observed, derived and user-labelled names separately. G14 must research game-region subdivision semantics and compare them with astronomical terminology, documenting differences rather than asserting equivalence. The user can choose “cluster” as a collective noun regardless of astrophysical classification.

## Delivery and unresolved decisions

Arc08 delivers identity/history/attribution, names/nouns, notes/media links, counts and region evidence. Arc09 delivers accepted geometry and exploration capabilities after G11/G12 decisions. Basic journals and culture narratives must remain useful while geometry research is unresolved. All analyses need reproducible inputs, explicit scope and confidence limits; no exact global population or “100% explored” claim is justified by a discovery-only sample.

Before production assignment: settle traveller attribution (G10), candidate populations/precision (G11), geometry and measures (G12), ranking/route/display contracts (G13), and region semantics (G14). The Operator's illustrated outcomes are requirements; the proposed technical approaches remain replaceable based on evidence.
