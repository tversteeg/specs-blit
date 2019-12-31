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

fn main() -> Result<()> {
    // Setup specs
    let mut world = World::new();

    // Load the blit components into the world
    world.register::<Sprite>();

    // Add the pixel buffer as a resource so it can be accessed from the RenderSystem later
    world.insert(PixelBuffer::new(WIDTH, HEIGHT));

    // Load the sprite
    let sprite_ref = {
        // Load the image using the image crate
        let img = image::open("examples/smiley.png")?;
        // Create a sprite from it
        let sprite = blit_buffer(&img, Color::from_u32(MASK_COLOR));

        // Move the sprite to the render system
        load(sprite)?
    };

    // Create an entity with the sprite
    world.create_entity().with(Sprite::new(sprite_ref)).build();

    // Setup the dispatcher with the blit system
    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(RenderSystem)
        .build();

    // Setup a minifb window
    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Specs Blit Example - ESC to exit", WIDTH, HEIGHT, options)?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update specs
        dispatcher.dispatch(&world);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Get the pixel buffer resource to render it
        let buffer = world.read_resource::<PixelBuffer>();
        // Render the pixel buffer
        window
            .update_with_buffer(&buffer.pixels(), WIDTH, HEIGHT)
            .unwrap();

        // Don't use 100% CPU
        sleep(Duration::from_millis(12));
    }

    Ok(())
}
