//! # Data Resource
//!
//! `data-resource` is a crate for managing resource identifiers.
use core::{fmt::Display, str::FromStr};
use data_error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, hash::Hash, path::Path};

/// This trait defines a generic type representing a resource identifier.
///
/// Resources are identified by a hash value, which is computed from the
/// resource's data. The hash value is used to uniquely identify the resource.
///
/// Implementors of this trait must provide a way to compute the hash value from
/// the resource's data.
pub trait ResourceId:
    Debug
    + Display
    + FromStr
    + Clone
    + PartialEq
    + Eq
    + Ord
    + PartialOrd
    + Hash
    + Serialize
    + DeserializeOwned
{
    /// Computes the resource identifier from the given file path
    fn from_path<P: AsRef<Path>>(file_path: P) -> Result<Self>;

    /// Computes the resource identifier from the given bytes
    fn from_bytes(data: &[u8]) -> Result<Self>;
}
