use context::Context;

mod config;
mod context;

/// Represents a site, and holds all the pages that are currently being worked on.
pub struct Site {
    ctx: Context,
}
