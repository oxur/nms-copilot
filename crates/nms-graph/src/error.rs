use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GraphError {
    #[error("system not found: {0}")]
    SystemNotFound(String),

    #[error("base not found: {0}")]
    BaseNotFound(String),

    #[error("no player position available")]
    NoPlayerPosition,

    #[error(
        "reference galaxy {reference} differs from active galaxy {active}; use an explicit address or base"
    )]
    ReferenceGalaxyMismatch { reference: u8, active: u8 },
}
