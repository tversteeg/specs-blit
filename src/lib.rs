//! # Specs ECS Rendering System
//!
//! This library exposes a 2D rendering system to be used in [specs](https://github.com/amethyst/specs).
//! It is based around the [blit](https://github.com/tversteeg/blit) library.
//! All the images will be rendered to a buffer which can be used in various
//! graphics libraries, e.g. minifb.
//! ```rust
//! use specs::prelude::*;
//! use specs_blit::{register_components, RenderSystem};
//!
//! fn main() {
//!     // Setup the specs world
//!     let mut world = World::new();
//!
//!     // Load the blit components into the world
//!     register_components(&mut world).unwrap();
//!
//!     // Setup the dispatcher with the blit system
//!     let mut dispatcher = DispatcherBuilder::new()
//!         .with_thread_local(RenderSystem)
//!         .build();
//! }
//! ```

use anyhow::Result;
use blit::*;
use specs::prelude::*;

/// Specs system for rendering to a buffer.
///
/// *Note*: This can only be used in conjunction with a `.with_thread_local()`
/// function in specs and not with a normal `.with()` call.
///
/// ```rust
/// # use specs_blit::RenderSystem;
/// use specs::prelude::*;
///
/// let mut dispatcher = DispatcherBuilder::new()
///     // Expose the blit render system to specs
///     .with_thread_local(RenderSystem)
///     .build();
/// ```
pub struct RenderSystem;
impl<'a> System<'a> for RenderSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

/// Register all rendering components in the specs world.
///
/// ```rust
/// # use specs_blit::register_components;
/// use specs::prelude::*;
///
/// // Setup the specs world
/// let mut world = World::new();
///
/// // Load the blit components into the world
/// register_components(&mut world).unwrap();
/// ```
pub fn register_components(_world: &mut World) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
