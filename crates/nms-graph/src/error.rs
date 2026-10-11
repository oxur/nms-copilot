use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GraphError {
    #[error("system not found: {0}")]
    SystemNotFound(String),

    #[error("base not found: {0}")]
    BaseNotFound(String),

    /// No discovered systems exist in the requested galaxy.
    #[error("no systems are known in galaxy {0}")]
    NoSystemsInGalaxy(u8),

    /// A named route target could not be resolved.
    #[error("route target not found: {0}")]
    TargetNotFound(String),

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
