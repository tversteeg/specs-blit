# specs-blit
2D sprite rendering extension for the [Specs ECS](https://github.com/amethyst/specs) system.

![CI](https://github.com/tversteeg/specs-blit/workflows/CI/badge.svg)
<a href="https://crates.io/crates/specs-blit"><img src="https://img.shields.io/crates/v/specs-blit.svg" alt="Version"/></a>
<a href="https://docs.rs/specs-blit"><img src="https://img.shields.io/badge/api-rustdoc-blue.svg" alt="Rust Documentation"/></a>
<img src="https://img.shields.io/crates/l/specs-blit.svg" alt="License"/>

All sprites are loaded onto a big array on the heap.

## Example

```rust
// Setup the specs world
let mut world = specs::World::new();

// Load the blit components into the world
world.register::<specs_blit::Sprite>();

// Add the pixel buffer as a resource so it can be accessed from the RenderSystem later
const WIDTH: usize = 800;
const HEIGHT: usize = 600;
world.insert(specs_blit::PixelBuffer::new(WIDTH, HEIGHT));

let sprite_ref = {
    // Load the image using the image crate
    let img = image::open("examples/smiley.png")?;
    // Create a sprite from it
	const MASK_COLOR: u32 = 0xFF00FF;
    let sprite = blit::blit_buffer(&img, blit::Color::from_u32(MASK_COLOR));

    // Move the sprite to the render system with 4 rotations
    specs_blit::load(sprite, 4)?
};

// Create a new sprite entity in the ECS system
world.create_entity()
	.with(specs_blit::Sprite::new(sprite_ref))
	.build();

// Setup the dispatcher with the blit system
let mut dispatcher = specs::DispatcherBuilder::new()
	.with_thread_local(specs_blit::RenderSystem)
	.build();

// Enter the render loop that should be called every frame
while render_frame() {
	// Update specs
	dispatcher.dispatch(&mut world);

	// Add/remove entities added in dispatch through `LazyUpdate`
	world.maintain();

	// Get the pixel buffer resource to render it
	let buffer = world.read_resource::<specs_blit::PixelBuffer>();
	// Render the pixel buffer
	window.update_with_buffer(&buffer.pixels(), buffer.width(), buffer.height())?;
}
```
