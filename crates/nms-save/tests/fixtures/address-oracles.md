# Address oracle fixture provenance

`address-oracles.json` is an exact byte copy of the accepted Slice20 projection
at `project01-mvp/arc06-fit-and-finish/slice20-save-address-decoder/artifacts/address-oracles.json`
(SHA-256 `2a48d77a745002227e6bdd5957189a35d415d69f4023d4c93e616236a9d4aeaa`).
The external integration test decodes every listed input with the public API
and compares all six named address fields and the 12-digit portal value.

The one discovery row, `W-CHILI`, is the previously accepted game-confirmed
galaxy-0 witness from Slice02. It establishes that this discovery field's
56-bit packed layout decodes its recorded value; it is not a general claim
about all game versions or nonzero galaxies.

The 44 base rows are one-save `HomePlanetBase` records associated by exact
`Name` plus `Position` with independent expanded-address endpoints from
Slice03. The projection retains all rows, their original string or integer
JSON kinds, repeated values and repeated expected addresses. These rows support
the bounded persistent-base field mapping in that save. They do not establish
universal base/discovery equivalence, native-ID identity, arbitrary game
formats, or a game-confirmed nonzero packed galaxy. Synthetic nonzero-galaxy
tests elsewhere in the suite test the declared bit contract only.

Expected coordinates and portal strings are copied from the independent
expanded witness projection, not computed from the decoder under test or from
`GalacticAddress::from_packed`. The existing core constructor intentionally
implements a separate 48-bit portal-frame contract.
