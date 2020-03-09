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
//! use rotsprite::rotsprite;
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
//!         // Create a sprite of 4 pixels
//!         let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
//!
//!         // Load the sprite and get a reference
//!         load(sprite, 1)?
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

pub extern crate blit;

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
///     // Create a sprite of 4 pixels
///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
///
///     // Load the sprite and get a reference
///     load(sprite, 1)?
/// };
///
/// // Create a new sprite entity in the ECS system
/// world.create_entity()
///     .with(Sprite::new(sprite_ref))
///     .build();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Sprite {
    /// The reference to the heap allocated array of sprites.
    pub(crate) reference: SpriteRef,
    /// Where on the screen the sprite needs to be rendered.
    pos: (i32, i32),
    /// The current rotation of the sprite, it will match the nearest rotating divisor of the
    /// loaded version.
    rot: i16,
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
    ///     // Create a sprite of 4 pixels
    ///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
    ///
    ///     // Load the sprite and get a reference
    ///     load(sprite, 1)?
    /// };
    ///
    /// // Create a specs sprite from the image
    /// let sprite = Sprite::new(sprite_ref);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(sprite_reference: SpriteRef) -> Self {
        Self {
            reference: sprite_reference,
            pos: (0, 0),
            rot: 0,
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

    /// Set the rotation in degrees of the sprite.
    /// The rotation will attempt to match the nearest degrees of rotation divisor.
    pub fn set_rot(&mut self, rotation: i16) {
        let mut rotation = rotation;
        while rotation < 0 {
            rotation += 360;
        }
        while rotation > 360 {
            rotation -= 360;
        }

        self.rot = rotation;
    }

    /// Get the pixel rotation as degrees.
    pub fn rot(&self) -> i16 {
        self.rot
    }

    /// Get the data needed for rendering this sprite.
    pub(crate) fn render_info(&self) -> (usize, i32, i32) {
        self.reference.render_info(self.rot)
    }
}

/// Reference to a heap-allocated sprite.
/// Contains the index of the vector, only this crate is allowed to access this.
#[derive(Debug, Clone)]
pub struct SpriteRef {
    /// In how many degrees the rotation is divided.
    rot_divisor: f64,
    /// Array of different rotations sprite references with their position offsets.
    sprites: Vec<(usize, i32, i32)>,
}

impl SpriteRef {
    // Return the reference index and the offsets of the position.
    pub(crate) fn render_info(&self, rotation: i16) -> (usize, i32, i32) {
        let rotation_index = rotation as f64 / self.rot_divisor;

        // Return the proper sprite depending on the rotation
        *self
            .sprites
            .get(rotation_index as usize)
            // Get the sprite at the index or the first if that's not valid
            .unwrap_or(&self.sprites[0])
    }
}

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
#[derive(Debug, Default)]
pub struct PixelBuffer {
    pub(crate) pixels: Vec<u32>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl PixelBuffer {
    /// Create a new buffer filled with black pixels.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![0; width * height],
            width,
            height,
        }
    }

    /// Get the array of pixels.
    pub fn pixels(&self) -> &Vec<u32> {
        &self.pixels
    }

    /// Get the width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Set all the pixels to the passed color.
    pub fn clear(&mut self, color: u32) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
    }
}

/// Specs system for rendering sprites to a buffer.
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
            let (index, x_offset, y_offset) = sprite_component.render_info();

            // Get the sprite from the array
            let sprite = &SPRITES.read().unwrap()[index];

            let pos = (
                sprite_component.pos.0 + x_offset,
                sprite_component.pos.1 + y_offset,
            );

            // Draw the sprite on the buffer
            sprite.blit(&mut buffer.pixels, width, pos);
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
/// // Create a sprite of 4 pixels
/// let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
///
/// // Load the sprite in rotations of 0, 90, 180 & 270 degrees and get a reference
/// let sprite_ref = load(sprite, 8)?;
/// # Ok(())
/// # }
/// ```
pub fn load(sprite: BlitBuffer, rotations: u16) -> Result<SpriteRef> {
    let rotations = if rotations == 0 { 1 } else { rotations };

    let rot_divisor = 360.0 / (rotations as f64);
    let raw_buffer = sprite.to_raw_buffer();

    // Create a rotation sprite for all rotations
    let sprites = (0..rotations)
        .map(|r| {
            let (rotated_width, rotated_height, rotated) = rotsprite::rotsprite(
                &raw_buffer,
                &sprite.mask_color().u32(),
                sprite.size().0 as usize,
                r as f64 * rot_divisor,
            )?;

            let rotated_sprite =
                BlitBuffer::from_buffer(&rotated, rotated_width as i32, sprite.mask_color());

            let mut sprites_vec = SPRITES.write().unwrap();
            sprites_vec.push(rotated_sprite);

            let index = sprites_vec.len() - 1;

            let x_offset = (sprite.width() - rotated_width as i32) / 2;
            let y_offset = (sprite.height() - rotated_height as i32) / 2;

            Ok((index, x_offset, y_offset))
        })
        .collect::<Result<Vec<_>>>()?
        // Return the first error
        .into_iter()
        .collect::<_>();

    Ok(SpriteRef {
        rot_divisor,
        sprites,
    })
}
