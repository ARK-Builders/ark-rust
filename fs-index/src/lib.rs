pub mod index;
mod serde;
mod utils;

pub use utils::load_or_build_index;

#[cfg(test)]
mod tests;

pub use index::ResourceIndex;
