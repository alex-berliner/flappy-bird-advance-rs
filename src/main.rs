// #![no_std]
// #![no_main]
// #![cfg_attr(test, feature(custom_test_frameworks))]
// #![cfg_attr(test, reexport_test_harness_main = "test_main")]
// #![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// use agb::{
//     include_aseprite,
//     display::object::{Graphics, Tag, Object, OamManaged},
// };

// struct Paddle<'obj> {
//     beg: Object<'obj>,
//     mid: Object<'obj>,
//     end: Object<'obj>,
// }

// impl<'obj> Paddle<'obj> {
//     fn new(object: &'obj OamManaged<'_>, facing_right:bool, start_x:i32, start_y:i32) -> Self {
//         let mut paddle_beg = object.object_sprite(PADDLE_END.sprite(0));
//         let mut paddle_mid = object.object_sprite(PADDLE_MID.sprite(0));
//         let mut paddle_end = object.object_sprite(PADDLE_END.sprite(0));

//         paddle_beg.set_hflip(facing_right);
//         paddle_mid.set_hflip(facing_right);
//         paddle_end.set_hflip(facing_right);
//         paddle_beg.show();
//         paddle_mid.show();
//         paddle_end.set_vflip(true).show();

//         let mut paddle = Self {
//             beg: paddle_beg,
//             mid: paddle_mid,
//             end: paddle_end,
//         };

//         paddle.set_position(start_x, start_y);

//         paddle
//     }

//     fn set_position(&mut self, x:i32, y:i32) {
//         self.beg.set_position((x,y));
//         self.mid.set_position((x,y + 16));
//         self.end.set_position((x,y + 32));
//     }
// }

// // load the sprites into this struct. This struct holds both the sprite and
// // palette data.
// static GRAPHICS: &Graphics = include_aseprite!("gfx/sprites.aseprite");

// static PADDLE_END: &Tag = GRAPHICS.tags().get("Paddle End");
// static PADDLE_MID: &Tag = GRAPHICS.tags().get("Paddle Mid");
// static BALL: &Tag = GRAPHICS.tags().get("Ball");

// #[agb::entry]
// fn main(mut gba:agb::Gba) -> ! {
//     let mut input = agb::input::ButtonController::new();
//     let object = gba.display.object.get_managed();
//     let mut ball = object.object_sprite(BALL.sprite(0));
//     let paddle_a = Paddle::new(&object, true, 8, 8);
//     let paddle_b = Paddle::new(&object, false, 240-16-8, 8);

//     ball.set_x(0).set_y(0).show();

//     // object.commit();
//     let mut ball_x = 100;
//     let mut ball_y = 100;
//     let mut vlcy_x = 1;
//     let mut vlcy_y = 1;
//     loop {
//         ball_x = (ball_x+vlcy_x).clamp(0, agb::display::WIDTH  - 16);
//         ball_y = (ball_y+vlcy_y).clamp(0, agb::display::HEIGHT - 16);
//         // logic
//         // ...

//         vlcy_x = input.x_tri() as i32;
//         vlcy_y = input.y_tri() as i32;

//         // if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
//         //     vlcy_x = -vlcy_x;
//         // }

//         // if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
//         //     vlcy_y = -vlcy_y;
//         // }

//         ball.set_x(ball_x as u16).set_y(ball_y as u16);
//         agb::display::busy_wait_for_vblank();
//         object.commit();
//         input.update();
//     }
// }

// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// Imports
use agb::{
    display::{
        object::{Graphics, OamManaged, Object, Sprite, Tag, TagMap},
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, RegularMap, TiledMap, TileFormat, TileSetting, VRamManager},
        Priority, HEIGHT, WIDTH,
    },
    include_aseprite,
    include_background_gfx,
    println,
    input::Button,
    mgba
};

// Add external crates
extern crate alloc;
// mod backgrounds;

// Include background image(s)
include_background_gfx!(background, tiles => deduplicate "map/Single Screen Test.png");

// Define Objects and types
struct Dave<'obj> {
    down_idle: Object<'obj>
}

impl<'obj> Dave<'obj> {
    fn new(object: &'obj OamManaged<'_>, start_x: i32, start_y: i32) -> Self {
        let mut down_idle = object.object_sprite(DAVE_DOWN.sprite(0));

        down_idle.show();

        let mut dave = Self {
            down_idle: down_idle
        };

        dave.down_idle.set_position((start_x, start_y));

        dave
    }
}

// Import the sprites in to this static. This holds the sprite
// and palette data in a way that is manageable by agb.
static DAVE_SPRITES: &Graphics = include_aseprite!("gfx/sprites.aseprite");

// We define some easy ways of referencing the sprites
static DAVE_DOWN: &Tag = DAVE_SPRITES.tags().get("Dave Down");

// The main function must take 1 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly. It will also handle creating the `Gba` struct for you.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    println!("hello");

    // Get the object manager
    let object = gba.display.object.get_managed();

    // Input controller
    let mut input = agb::input::ButtonController::new();

    // Now commit the object controller so this change is reflected on the screen.
    // This isn't how we will do this in the final version of the code, but will do
    // for this example.
    // Dave positions
    let mut dave_x: i32 = 120;
    let mut dave_y: i32 = 80;

    // now we initialise the x and y velocities to 0 rather than 1
    // Dave velocities
    let mut dave_vel_x: i32 = 0;
    let mut dave_vel_y: i32 = 0;

    // Make our buddy Dave
    let mut dave = object.object_sprite(DAVE_DOWN.sprite(0));
    dave.set_x(dave_x as u16).set_y(dave_y as u16).show();


    // Backgrounds
    // Set the display to use tiled backgrounds
    let (gfx, mut vram) = gba.display.video.tiled0();
    let tileset = &background::tiles.tiles;

    vram.set_background_palettes(background::PALETTES);

    let mut bg = gfx.background(Priority::P0, RegularBackgroundSize::Background32x32, tileset.format());

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile(
                &mut vram,
                (x, y),
                tileset,
                background::tiles.tile_settings[(y*30+x) as usize],
            );
            println!("{x}, {y}");
        }
    }

    bg.commit(&mut vram);
    bg.set_visible(true);

    // Main loop
    loop {
        // Dave controls
        dave_x = (dave_x + dave_vel_x).clamp(0, agb::display::WIDTH - 16);
        dave_y = (dave_y + dave_vel_y).clamp(-8, agb::display::HEIGHT - 32);

        // x_tri and y_tri describe with -1, 0 and 1 which way the d-pad
        // buttons are being pressed
        dave_vel_x = input.x_tri() as i32;
        dave_vel_y = input.y_tri() as i32;

        if input.is_pressed(Button::A) {
            // the A button is pressed
            dave_vel_x = dave_vel_x*2;
            dave_vel_y = dave_vel_y*2;
        }

        dave.set_x(dave_x as u16).set_y(dave_y as u16);

        agb::display::busy_wait_for_vblank();
        object.commit();

        // We must call input.update() every frame otherwise it won't update based
        // on the actual button press state.
        input.update();
    }
}
