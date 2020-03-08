#![cfg(feature = "rotation")]

use crate::{load, PixelBuffer, SpriteRef, SPRITES};
use anyhow::Result;
use blit::BlitBuffer;
use specs::prelude::*;

/// Specs component representing a sprite that can be drawn in a rotated fashion.
///
/// ```rust
/// use blit::{BlitBuffer, Color};
/// use specs::prelude::*;
/// use specs_blit::rotation::{load_with_rotations, RotatingSprite};
///
/// const MASK_COLOR: u32 = 0xFF00FF;
///
/// # fn main() -> anyhow::Result<()> {
/// // Setup the specs world
/// let mut world = World::new();
///
/// // Load the blit components into the world
/// world.register::<RotatingSprite>();
///
/// let sprite_ref = {
///     // Create a sprite of 4 pixels
///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
///
///     // Load the sprite in rotations of 0, 90, 180 & 270 degrees and get a reference
///     load_with_rotations(sprite, 4)?
/// };
///
/// // Create a new sprite entity in the ECS system
/// world.create_entity()
///     .with(RotatingSprite::new(sprite_ref))
///     .build();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RotatingSprite {
    /// The reference to the heap allocated array of sprites.
    pub(crate) reference: RotatingSpriteRef,
    /// Where on the screen the sprite needs to be rendered.
    pos: (i32, i32),
    /// The current rotation of the sprite, it will match the nearest rotating divisor of the
    /// loaded version.
    rot: u16,
}

impl RotatingSprite {
    /// Instantiate a new sprite from a loaded sprite index.
    ///
    /// ```rust
    /// use blit::{BlitBuffer, Color};
    /// use specs_blit::rotation::{load_with_rotations, RotatingSprite};
    ///
    /// const MASK_COLOR: u32 = 0xFF00FF;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let sprite_ref = {
    ///     // Create a sprite of 4 pixels
    ///     let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
    ///
    ///     // Load the sprite in rotations of 0, 90, 180 & 270 degrees and get a reference
    ///     load_with_rotations(sprite, 4)?
    /// };
    ///
    /// // Create a specs sprite from the image
    /// let sprite = RotatingSprite::new(sprite_ref);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(sprite_ref: &RotatingSpriteRef) -> Self {
        Self {
            reference: sprite_ref.clone(),
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
    pub fn set_rot(&mut self, rotation: u16) {
        self.rot = rotation % 360;
    }

    /// Get the pixel rotation as degrees.
    pub fn rot(&self) -> u16 {
        self.rot
    }

    /// Get the reference matching the rotation of this sprite.
    pub(crate) fn reference(&self) -> &SpriteRef {
        self.reference.reference(self.rot)
    }
}

impl Component for RotatingSprite {
    type Storage = VecStorage<Self>;
}

/// Reference to a heap-allocated sprite.
/// Contains the index of the vector, only this crate is allowed to access this.
#[derive(Debug, Clone)]
pub struct RotatingSpriteRef {
    /// In how many degrees the rotation is divided.
    rot_divisor: f64,
    /// Array of different rotations sprite references.
    sprites: Vec<SpriteRef>,
}

impl RotatingSpriteRef {
    pub(crate) fn reference(&self, rotation: u16) -> &SpriteRef {
        let rotation_index = rotation as f64 / self.rot_divisor;

        // Get the sprite at the index or the first if that's not valid
        self.sprites
            .get(rotation_index as usize)
            .unwrap_or(&self.sprites[0])
    }
}

/// Specs system for rendering rotated sprites to a buffer.
///
/// *Note*: This can only be used in conjunction with a `.with_thread_local()`
/// function in specs and not with a normal `.with()` call.
///
/// ```rust
/// use specs::prelude::*;
/// use specs_blit::RotationRenderSystem;
///
/// let mut dispatcher = DispatcherBuilder::new()
///     // Expose the sprite render system to specs
///     .with_thread_local(RotationRenderSystem)
///     .build();
/// ```
pub struct RotationRenderSystem;
impl<'a> System<'a> for RotationRenderSystem {
    type SystemData = (Write<'a, PixelBuffer>, ReadStorage<'a, RotatingSprite>);

    fn run(&mut self, (mut buffer, sprites): Self::SystemData) {
        let width = buffer.width;

        for sprite_component in sprites.join() {
            // Get the sprite from the array
            let sprite = &SPRITES.read().unwrap()[sprite_component.reference().0];

            // Draw the sprite on the buffer
            sprite.blit(&mut buffer.pixels, width, sprite_component.pos);
        }
    }
}

/// Load a sprite buffer, rotate it in different rotations and place it onto the heap.
///
/// Returns a reference that can be used in sprite components.
///
/// ```rust
/// use blit::{BlitBuffer, Color};
/// use specs_blit::rotation::load_with_rotations;
///
/// const MASK_COLOR: u32 = 0xFF00FF;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create a sprite of 4 pixels
/// let sprite = BlitBuffer::from_buffer(&[0, MASK_COLOR, 0, 0], 2, MASK_COLOR);
///
/// // Load the sprite and get a reference
/// // Creates rotations for 0, 45, 90, 135, 180, 225, 270 & 315 degrees
/// let sprite_ref = load_with_rotations(sprite, 8)?;
/// # Ok(())
/// # }
/// ```
pub fn load_with_rotations(sprite: BlitBuffer, rotations: u16) -> Result<RotatingSpriteRef> {
    let rot_divisor = 360.0 / (rotations as f64);
    let raw_buffer = sprite.to_raw_buffer();

    // Create a rotation sprite for all rotations
    let sprites = (0..rotations)
        .map(|r| {
            let (rotated_width, _rotated_height, rotated) = rotsprite::rotsprite(
                &raw_buffer,
                &sprite.mask_color().u32(),
                sprite.size().0 as usize,
                r as f64 * rot_divisor,
            )?;

            let sprite =
                BlitBuffer::from_buffer(&rotated, rotated_width as i32, sprite.mask_color());

            load(sprite)
        })
        .collect::<Result<Vec<_>>>()?
        // Return the first error
        .into_iter()
        .collect::<_>();

    Ok(RotatingSpriteRef {
        rot_divisor,
        sprites,
    })
}
