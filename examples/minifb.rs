use anyhow::Result;
use blit::{blit_buffer, Color};
use minifb::*;
use specs::prelude::*;
use specs_blit::{load, PixelBuffer, RenderSystem, Sprite};

use std::thread::sleep;
use std::time::Duration;

const WIDTH: usize = 250;
const HEIGHT: usize = 250;

const MASK_COLOR: u32 = 0xFF_00_FF;

// A resource for rotating the sprite
#[derive(Debug, Default)]
pub struct Rotation(pub f64);

// The system for rotating the sprite
pub struct RotationSystem;
impl<'a> System<'a> for RotationSystem {
    type SystemData = (Read<'a, Rotation>, WriteStorage<'a, Sprite>);

    fn run(&mut self, (rot, mut sprite): Self::SystemData) {
        // Rotate the sprite
        for (sprite,) in (&mut sprite,).join() {
            sprite.set_rot(rot.0 as i16);
        }
    }
}

fn main() -> Result<()> {
    // Setup specs
    let mut world = World::new();

    // Load the blit components into the world
    world.register::<Sprite>();

    // Add the pixel buffer as a resource so it can be accessed from the RenderSystem later
    world.insert(PixelBuffer::new(WIDTH, HEIGHT));

    // Add the rotation of the sprite
    world.insert(Rotation(0.0));

    // Load the sprite
    let sprite_ref = {
        // Load the image using the image crate
        let img = image::open("examples/smiley.png")?;
        // Create a sprite from it
        let sprite = blit_buffer(&img, Color::from_u32(MASK_COLOR));

        // Move the sprite to the render system with 16 rotations
        load(sprite, 16)?
    };

    // Create an entity with the sprite
    world.create_entity().with(Sprite::new(sprite_ref)).build();

    // Setup the dispatcher with the blit system
    let mut dispatcher = DispatcherBuilder::new()
        .with(RotationSystem, "rotation", &[])
        .with_thread_local(RenderSystem)
        .build();

    // Setup a minifb window
    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Specs Blit Example - ESC to exit", WIDTH, HEIGHT, options)?;

    let mut rotation = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            // Clear the buffer
            let mut buffer = world.write_resource::<PixelBuffer>();
            buffer.clear(0);

            // Update the rotation
            let mut rot_resource = world.write_resource::<Rotation>();
            rot_resource.0 = rotation;
            rotation += 1.0;
        }

        // Update specs
        dispatcher.dispatch(&world);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Get the pixel buffer resource to render it
        let buffer = world.read_resource::<PixelBuffer>();

        // Render the pixel buffer
        window
            .update_with_buffer(&buffer.pixels(), buffer.width(), buffer.height())
            .unwrap();

        // Don't use 100% CPU
        sleep(Duration::from_millis(12));
    }

    Ok(())
}
