//! Compatibility adapters for selected No Man's Sky save representations.
//!
//! This crate currently exposes the [`nomnom`] JSON heuristic and typed
//! passthrough parser. It does not implement the goatfungus escape fixer or
//! claim that synthetic JSON demonstrates an authentic NomNom export.

pub mod nomnom;
