//! # Specs ECS Rendering System
//!
//! This library exposes a 2D rendering system to be used in [specs](https://github.com/amethyst/specs).
//! It is based around the [blit](https://github.com/tversteeg/blit) library.
//! All the images will be rendered to a buffer which can be used in various
//! graphics libraries, e.g. minifb.
//!
//! All sprites are loaded onto a big array on the heap.
//! ```rust
//! use anyhow::Result;
//! use blit::{BlitBuffer, Color};
//! use specs::prelude::*;
//! use specs_blit::{load, PixelBuffer, RenderSystem, Sprite};
//!
//! const WIDTH: usize = 800;
//! const HEIGHT: usize = 800;
//! const MASK_COLOR: u32 = 0xFF00FF;
//!
//! fn main() -> Result<()> {
//!     // Setup the specs world
//!     let mut world = World::new();
//!
//!     // Load the blit components into the world
//!     world.register::<Sprite>();
//!
//!     // Add the pixel buffer as a resource so it can be accessed from the RenderSystem later
//!     world.insert(PixelBuffer::new(WIDTH, HEIGHT));
//!
//!     let sprite_ref = {
//!         // Create a sprite of 2 pixels
//!         let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR], 1, Color::from_u32(MASK_COLOR));
//!
//!         // Load the sprite and get a reference
//!         load(sprite)?
//!     };
//!
//!     // Create a new sprite entity in the ECS system
//!     world.create_entity()
//!         .with(Sprite::new(sprite_ref))
//!         .build();
//!
//!     // Setup the dispatcher with the blit system
//!     let mut dispatcher = DispatcherBuilder::new()
//!         .with_thread_local(RenderSystem)
//!         .build();
//!
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use blit::BlitBuffer;
use lazy_static::lazy_static;
use specs::prelude::*;

use std::sync::RwLock;

// The heap allocated array of sprites
// It's wrapped in a RwLock so all threads can access it
lazy_static! {
    static ref SPRITES: RwLock<Vec<BlitBuffer>> = RwLock::new(vec![]);
}

/// Specs component representing a sprite that can be drawn.
///
/// ```rust
/// use blit::{BlitBuffer, Color};
/// use specs::prelude::*;
/// use specs_blit::{load, Sprite};
///
/// const MASK_COLOR: u32 = 0xFF00FF;
///
/// # fn main() -> anyhow::Result<()> {
/// // Setup the specs world
/// let mut world = World::new();
///
/// // Load the blit components into the world
/// world.register::<Sprite>();
///
/// let sprite_ref = {
///     // Create a sprite of 2 pixels
///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR], 1, Color::from_u32(MASK_COLOR));
///
///     // Load the sprite and get a reference
///     load(sprite)?
/// };
///
/// // Create a new sprite entity in the ECS system
/// world.create_entity()
///     .with(Sprite::new(sprite_ref))
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct Sprite {
    /// The reference to the heap allocated array of sprites.
    pub(crate) index: usize,
    /// Where on the screen the sprite needs to be rendered.
    pos: (i32, i32),
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

impl Sprite {
    /// Instantiate a new sprite from a loaded sprite index.
    ///
    /// ```rust
    /// use blit::{BlitBuffer, Color};
    /// use specs_blit::{load, Sprite};
    ///
    /// const MASK_COLOR: u32 = 0xFF00FF;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let sprite_ref = {
    ///     // Create a sprite of 2 pixels
    ///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR], 1, Color::from_u32(MASK_COLOR));
    ///
    ///     // Load the sprite and get a reference
    ///     load(sprite)?
    /// };
    ///
    /// // Create a specs sprite from the image
    /// let sprite = Sprite::new(sprite_ref);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(index: SpriteRef) -> Self {
        Self {
            index: index.0,
            pos: (0, 0),
        }
    }

    /// Set the pixel position of where the sprite needs to be rendered.
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.pos.0 = x;
        self.pos.1 = y;
    }

    /// Get the pixel position as an (x, y) tuple of where the sprite will be rendered.
    pub fn pos(&self) -> (i32, i32) {
        self.pos
    }
}

/// Reference to a heap-allocated sprite.
/// Contains the index of the vector, only this crate is allowed to access this.
pub struct SpriteRef(pub(crate) usize);

/// Array of pixels resource that can be written to from the [`RenderSystem`] system.
///
/// ```rust
/// use specs::prelude::*;
/// use specs_blit::PixelBuffer;
///
/// const WIDTH: usize = 800;
/// const HEIGHT: usize = 800;
///
/// // Setup the specs world
/// let mut world = World::new();
///
/// // Add the pixel buffer as a resource so it can be accessed from the RenderSystem later
/// world.insert(PixelBuffer::new(WIDTH, HEIGHT));
/// ```
#[derive(Default)]
pub struct PixelBuffer {
    pub(crate) pixels: Vec<u32>,
    pub(crate) width: usize,
}

impl PixelBuffer {
    /// Create a new buffer filled with black pixels.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![0; width * height],
            width,
        }
    }

    /// Get the array of pixels.
    pub fn pixels(&self) -> &Vec<u32> {
        &self.pixels
    }
}

/// Specs system for rendering to a buffer.
///
/// *Note*: This can only be used in conjunction with a `.with_thread_local()`
/// function in specs and not with a normal `.with()` call.
///
/// ```rust
/// use specs::prelude::*;
/// use specs_blit::RenderSystem;
///
/// let mut dispatcher = DispatcherBuilder::new()
///     // Expose the sprite render system to specs
///     .with_thread_local(RenderSystem)
///     .build();
/// ```
pub struct RenderSystem;
impl<'a> System<'a> for RenderSystem {
    type SystemData = (Write<'a, PixelBuffer>, ReadStorage<'a, Sprite>);

    fn run(&mut self, (mut buffer, sprites): Self::SystemData) {
        let width = buffer.width;

        for sprite_component in sprites.join() {
            // Get the sprite from the array
            let sprite = &SPRITES.read().unwrap()[sprite_component.index];

            // Draw the sprite on the buffer
            sprite.blit(&mut buffer.pixels, width, sprite_component.pos);
        }
    }
}

/// Load a sprite buffer and place it onto the heap.
///
/// Returns an index that can be used in sprite components.
///
/// ```rust
/// use blit::{BlitBuffer, Color};
/// use specs_blit::load;
///
/// const MASK_COLOR: u32 = 0xFF00FF;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create a sprite of 2 pixels
/// let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR], 1, Color::from_u32(MASK_COLOR));
///
/// // Load the sprite and get a reference
/// let sprite_ref = load(sprite)?;
/// # Ok(())
/// # }
/// ```
pub fn load(sprite: BlitBuffer) -> Result<SpriteRef> {
    let mut sprites_vec = SPRITES.write().unwrap();
    sprites_vec.push(sprite);

    Ok(SpriteRef(sprites_vec.len() - 1))
}
