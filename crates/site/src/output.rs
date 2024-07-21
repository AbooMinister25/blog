use std::path::Path;

use color_eyre::Result;

use crate::context::Context;

/// Describes an "output" of the static site generator. This trait will be
/// implemented by any type which will be written to disk as an output
/// of the static site generator.
pub trait Output {
    fn write(&self, ctx: &Context) -> Result<()>;
    fn path(&self) -> &Path;
    fn hash(&self) -> &str;
    fn fresh(&self) -> bool;
}
