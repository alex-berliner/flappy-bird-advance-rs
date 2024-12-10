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
    input::{Button, ButtonController, Tri},
    display::{
        object::{Graphics, OamManaged, Object, Sprite, TagMap},
        tiled::{MapLoan, RegularBackgroundSize, RegularMap, TileFormat, TileSet, TileSetting, TiledMap, VRamManager}
    },
    fixnum::Vector2D,
    include_background_gfx,
    interrupt::{add_interrupt_handler, Interrupt, VBlank},
    rng,
    Gba
};

extern crate alloc;
use alloc::vec::Vec;

enum CollType {
    Bird,
    Pipe,
    Ground,
}

static GRAPHICS: &Graphics = agb::include_aseprite!(
    "gfx/sprites.aseprite"
);
static SPRITES: &[Sprite] = GRAPHICS.sprites();
static TAG_MAP: &TagMap = GRAPHICS.tags();

include_background_gfx!(backgrounds, "000000",
    background => deduplicate "gfx/background.aseprite",
    sprites => deduplicate "gfx/sprites.aseprite",
);

fn set_background(mut vram: &mut VRamManager, bg: &mut MapLoan<'_, RegularMap>) {
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
}

pub trait Collidable {
    fn get_name(&self) -> &str;
    fn get_rect(&self) -> &Rect;
    fn collides(&self, other: &impl Collidable) -> bool {
        let my_rect = self.get_rect();
        // log::error!("1: {:?}", my_rect);
        // log::error!("2: {:?}", other.get_rect());
        // log::error!("");
        let other_rect = other.get_rect();
        // Check for overlap in the x-axis
        let x_overlap = my_rect.pos.x < other_rect.pos.x + other_rect.size.x
            && my_rect.pos.x + my_rect.size.x > other_rect.pos.x;

        // Check for overlap in the y-axis
        let y_overlap = my_rect.pos.y < other_rect.pos.y + other_rect.size.y
            && my_rect.pos.y + my_rect.size.y > other_rect.pos.y;

        // Collision occurs if there's overlap on both axes
        let collides = x_overlap && y_overlap;

        if collides {
            log::error!("{} collides {}", self.get_name(), other.get_name());
        }

        collides
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Rect {
    size: Vector2D<i32>,
    pos: Vector2D<i32>,
}

struct Pipe<'obj> {
    rect: Rect,
    top_left: Object<'obj>,
    top_right: Object<'obj>,
    leftside: Vec<Object<'obj>>,
    rightside: Vec<Object<'obj>>,
    middle: Vec<Object<'obj>>,
}

impl Collidable for Pipe<'_> {
    fn get_rect(&self) -> &Rect {
        &self.rect
    }
    fn get_name(&self) -> &str {
        "Pipe"
    }
}

impl<'obj> Pipe<'obj> {
    fn new(object: &'obj OamManaged<'_>, x: i32, y: i32, height:i32) -> Self{
        let rect = Rect {
            size: (8*3, 8*height).into(),
            pos: (x,y).into(),
        };
        let pipe_top_left_tag = GRAPHICS.tags().get("Pipe Top Left");
        let pipe_top_right_tag = GRAPHICS.tags().get("Pipe Top Right");
        let pipe_middle_tag = GRAPHICS.tags().get("Pipe Middle");
        let pipe_left_side_tag = GRAPHICS.tags().get("Pipe Left Side");
        let pipe_right_side_tag = GRAPHICS.tags().get("Pipe Right Side");

        let mut pipe_topleft = object.object_sprite(pipe_top_left_tag.sprite(0));
        let mut pipe_topright = object.object_sprite(pipe_top_right_tag.sprite(0));
        // pipe_topright.set_hflip(true);
        let mut midvec:Vec<Object<'_>> = Vec::new();
        let mut leftsidevec:Vec<Object<'_>> = Vec::new();
        let mut rightsidevec:Vec<Object<'_>> = Vec::new();
        // let height = 4;
        for _ in 0..height+1 {
            let mut pipe_middle = object.object_sprite(pipe_middle_tag.sprite(0));
            pipe_middle.show();
            midvec.push(pipe_middle);
        }
        for _ in 0..height {
            let mut pipe_left_side = object.object_sprite(pipe_left_side_tag.sprite(0));
            pipe_left_side.show();
            leftsidevec.push(pipe_left_side);

            let mut pipe_right_side = object.object_sprite(pipe_right_side_tag.sprite(0));
            pipe_right_side.show();
            rightsidevec.push(pipe_right_side);
        }
        pipe_topleft.show();
        pipe_topright.show();
        let mut pipe = Self {
            rect: rect,
            top_left: pipe_topleft,
            top_right: pipe_topright,
            leftside: leftsidevec,
            rightside: rightsidevec,
            middle: midvec,
        };
        pipe.update_pos(pipe.rect.pos);
        pipe
    }

    fn update_pos(&mut self, pos:Vector2D<i32>) {
        self.rect.pos = pos;
        let (x,y) = (self.rect.pos.x, self.rect.pos.y);
        self.top_left.set_position((x,y));
        self.top_right.set_position((x+16,y));
        for (i, e) in self.middle.iter_mut().enumerate() {
            e.set_position(((x+8) as i32, y+(i as i32)*8));
        }
        for (i, e) in self.leftside.iter_mut().enumerate() {
            e.set_position(((x) as i32, y+((i+1) as i32)*8));
        }
        for (i, e) in self.rightside.iter_mut().enumerate() {
            e.set_position(((x+16) as i32, y+((i+1) as i32)*8));
        }
    }

    fn get_pos(&self) -> Vector2D<i32> {
        (self.top_left.x(), self.top_left.y()).into()
    }

    // fn is_colliding(&self, other_rect: &Rect) -> bool { // Check for overlap in the x-axis let x_overlap = self.rect.pos.x < other_rect.pos.x + other_rect.size.x && self.rect.pos.x + self.rect.size.x > other_rect.pos.x; // Check for overlap in the y-axis let y_overlap = self.rect.pos.y < other_rect.pos.y + other_rect.size.y && self.rect.pos.y + self.rect.size.y > other_rect.pos.y; // Collision occurs if there's overlap on both axes x_overlap && y_overlap }
}

struct Bird<'obj> {
    rect: Rect,
    img: Object<'obj>,
}

impl Collidable for Bird<'_> {
    fn get_rect(&self) -> &Rect {
        &self.rect
    }

    fn get_name(&self) -> &str {
        "bird"
    }
}

impl<'obj> Bird<'obj> {
    fn new(object: &'obj OamManaged<'_>, x: i32, y: i32) -> Self{
        let bird_idle_tag = GRAPHICS.tags().get("Bird Idle");
        let mut bird_img:Object<'obj> = object.object_sprite(bird_idle_tag.sprite(0));
        bird_img.show();
        let bird = Self {
            rect: Rect {
                size: (8,8).into(),
                pos: (x,y).into(),
            },
            img:bird_img
        };
        bird
    }

    fn handle_movement(&mut self, input: &ButtonController, frame_ctr: u32) {
        if frame_ctr % 2 == 0{
            let pixel_move_x = match input.x_tri() {
                Tri::Negative => -1,
                Tri::Positive => 1,
                Tri::Zero => 0,
            };
            let pixel_move_y = match input.y_tri() {
                Tri::Negative => -1,
                Tri::Positive => 1,
                Tri::Zero => 0,
            };
            self.rect.pos = (&self.rect.pos.x + pixel_move_x, self.rect.pos.y + pixel_move_y).into();
            self.img.set_position(self.rect.pos);
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    mgba_log::init().expect("unable to initialize mGBA logger");
    let mut input = ButtonController::new();
    let object = gba.display.object.get_managed();
    let bird_tag = GRAPHICS.tags().get("Bird Idle");
    let mut bird = Bird::new(&object, 0,0);//= object.object_sprite(bird_tag.sprite(0));
    // bird.set_x(50).set_y(50).show();
    let vblank = VBlank::get();
    let (mut gfx, mut vram) = gba.display.video.tiled0();
    let (gfx, mut vram) = gba.display.video.tiled0();
    let mut bg = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut pipes = [
        Pipe::new(&object, 40, 104, 4),
        Pipe::new(&object, 100, 104, 4),
    ];
    // let mut pipe2 = ;
    // let mut pipe = ;
    object.commit();
    set_background(&mut vram, &mut bg);
    bg.commit(&mut vram);
    bg.set_visible(true);
    let mut frame_ctr:u32 = 0;
    loop {
        input.update(); // Update button states
        bird.handle_movement(&input, frame_ctr);
        if frame_ctr%20 == 0{
            let ppos = pipes[0].get_pos();
            pipes[0].update_pos(((ppos.x-1) as i32, ppos.y as i32).into());
        }
        object.commit();
        for p in pipes.iter() {
            bird.collides(p);
        }

        frame_ctr += 1;
        vblank.wait_for_vblank();
    }
}