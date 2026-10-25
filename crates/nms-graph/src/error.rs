//! Errors returned by graph queries and model operations.

use thiserror::Error;

/// An error returned by a graph lookup or spatial query.
///
/// Variants describe the graph API's error vocabulary; an individual method
/// returns only the variants documented on that method.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GraphError {
    /// A requested system could not be found.
    #[error("system not found: {0}")]
    SystemNotFound(
        /// Supplied system reference that failed lookup.
        String,
    ),

    /// A requested base could not be found.
    #[error("base not found: {0}")]
    BaseNotFound(
        /// Supplied base name that failed lookup.
        String,
    ),

    /// No model systems, including placeholders, exist in the requested galaxy.
    #[error("no systems in galaxy {0}")]
    NoSystemsInGalaxy(
        /// Galaxy index with no systems.
        u8,
    ),

    /// A named route target could not be resolved.
    #[error("target not found: {0}")]
    TargetNotFound(
        /// Supplied target reference that failed lookup.
        String,
    ),

    /// No player position is available for an implicit reference point.
    #[error("no player position available")]
    NoPlayerPosition,

    /// The player's current-position galaxy differs from the active model galaxy.
    ///
    /// Explicit address and base references use their own galaxy and do not
    /// produce this error.
    #[error(
        "reference galaxy {reference} differs from active galaxy {active}; use an explicit address or base"
    )]
    ReferenceGalaxyMismatch {
        /// Reality index encoded by the player's current-position address.
        reference: u8,
        /// Reality index selected by the model for unqualified queries.
        active: u8,
    },
}
