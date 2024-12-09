/*
=== design ===
the game flappy bird
a bird has to come incoming pipes by flapping its wings
the bird gains height with the jump button and falls with gravity
pipes come from the right to the left, with an opening of static size in the middle
the placement of the opening is at a different height each time a pipe comes through
50 points are added each time the bird flies through a pipe, with a score counter at the top right
=== architecture ===
- assets
    bird: at least 2 sprites, for flapping and not flapping
    pipe: 6 tiles? left, middle, right for pipe and top/bottom
    ground: grass (top), soil (bottom)
    background: a full tile sheet
    music: flappy bird song, flapping noise
- code
    bird movement
    collision detection
    displaying sprites
    - an on-the-fly updated background may be able to be used for the pipes but
        a simpler answer would be to just use sprites
    displaying score
    playing music

=== implementation ===
sprite display:
    - idk just display the sprites man
give objects a type - ground, pole, char to distinguish during collision
*/


/*
use single background for floor and pipes
have a struct for object data and collision type
*/
#![no_std]
#![no_main]
use agb::{
    Gba,
    interrupt::{add_interrupt_handler, Interrupt, VBlank},
    display::tiled::{TileFormat, RegularBackgroundSize, RegularMap, TileSet, TileSetting, TiledMap, VRamManager},
    include_background_gfx, rng,
};

enum CollType {
    Bird,
    Pipe,
    Ground,
}

include_background_gfx!(backgrounds, "121105",
    background => deduplicate "gfx/background.aseprite",
    // title => deduplicate "gfx/title-screen.aseprite",
    // help => deduplicate "gfx/help-text.aseprite",
    // descriptions1 => deduplicate "gfx/descriptions1.png",
    // descriptions2 => deduplicate "gfx/descriptions2.png",
);

// Safety: doesn't allocate
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    mgba_log::init().expect("unable to initialize mGBA logger");
    let vblank = VBlank::get();
    let (mut gfx, mut vram) = gba.display.video.tiled0();
    let (gfx, mut vram) = gba.display.video.tiled0();
    let mut bg = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    vram.set_background_palettes(backgrounds::PALETTES);
    let bg_tiledata = &backgrounds::background;
    for i in 0..20*30 {
        let x:u16 = i%30;
        let y:u16 = i/30;
        bg.set_tile(
            &mut vram,
            (x as u16, y as u16),
            &bg_tiledata.tiles,
            bg_tiledata.tile_settings[i as usize],
        );
    }
    bg.commit(&mut vram);
    bg.set_visible(true);

    loop {
        vblank.wait_for_vblank();
    }
}