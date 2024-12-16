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
#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use agb::{
    fixnum::Vector2D,
    include_background_gfx,
    interrupt::VBlank,
    rng::RandomNumberGenerator,
    input::{ButtonController, Tri},
    display::{
        HEIGHT,
        WIDTH,
        object::{Graphics, OamManaged, Object},
        tiled::{
            MapLoan,
            RegularBackgroundSize,
            RegularMap,
            TileFormat,
            TiledMap,
            VRamManager
        },
    },
};

static GRAPHICS: &Graphics = agb::include_aseprite!(
    "gfx/sprites.aseprite"
);

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
        let other_rect = other.get_rect();
        let x_overlap = my_rect.pos.x < other_rect.pos.x + other_rect.size.x
            && my_rect.pos.x + my_rect.size.x > other_rect.pos.x;

        let y_overlap = my_rect.pos.y < other_rect.pos.y + other_rect.size.y
            && my_rect.pos.y + my_rect.size.y > other_rect.pos.y;

        let collides = x_overlap && y_overlap;

        collides
    }
}

struct Obstacle<'obj> {
    top_pipe: Pipe<'obj>,
    bot_pipe: Pipe<'obj>,
}

impl<'obj> Obstacle<'obj> {
    fn create_pipe_pos_and_height(rng: &mut RandomNumberGenerator) -> [[i32; 2];2] {
        let gap = 64;
        let max_pipe_height = (3*HEIGHT)/4 - gap;
        let _min_pipe_height = HEIGHT - max_pipe_height /* + gap */;
        let top_pipe_height = max_pipe_height;
        let top_pipe_top_pos = -(((rng.gen()as u32)%(top_pipe_height as u32)) as i32);
        let top_pipe_bot_pos = top_pipe_top_pos + max_pipe_height;
        let bot_pipe_top_pos = top_pipe_bot_pos + gap;
        let bot_pipe_height = HEIGHT - bot_pipe_top_pos;
        [[top_pipe_top_pos, top_pipe_height],[bot_pipe_top_pos, bot_pipe_height]]
    }

    fn new(object: &'obj OamManaged<'_>, rng: &mut RandomNumberGenerator, x: i32) -> Self {
        let r = Obstacle::create_pipe_pos_and_height(rng);
        let mut top_pipe = Pipe::new(&object);
        let mut bot_pipe = Pipe::new(&object);
        top_pipe.update_pos((x, r[0][0]).into());
        top_pipe.set_height(object, r[0][1]/8);
        top_pipe.show();
        bot_pipe.update_pos((x, r[1][0]).into());
        bot_pipe.set_height(object, r[1][1]/8);
        bot_pipe.show();
        Self {
            top_pipe,
            bot_pipe
        }
    }

    fn set_pos(&mut self, x:i32) {
        self.top_pipe.update_pos((x, self.top_pipe.rect.pos.y).into());
        self.bot_pipe.update_pos((x, self.bot_pipe.rect.pos.y).into());
    }

    fn move_tick(&mut self) {
        if self.top_pipe.moving {
            self.top_pipe.rect.pos = (self.top_pipe.rect.pos.x-1, self.top_pipe.rect.pos.y).into();
            self.top_pipe.update_pos(self.top_pipe.rect.pos);
        }
        if self.bot_pipe.moving {
            self.bot_pipe.rect.pos = (self.bot_pipe.rect.pos.x-1, self.bot_pipe.rect.pos.y).into();
            self.bot_pipe.update_pos(self.bot_pipe.rect.pos);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Rect {
    size: Vector2D<i32>,
    pos: Vector2D<i32>,
}

struct Pipe<'obj> {
    height:i32,
    rect: Rect,
    top_left: Object<'obj>,
    top_right: Object<'obj>,
    leftside: Vec<Object<'obj>>,
    rightside: Vec<Object<'obj>>,
    middle: Vec<Object<'obj>>,
    moving: bool,
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
    fn new(object: &'obj OamManaged<'_>) -> Self {
        let rect = Rect {
            size: (8*3, 0).into(),
            pos:  (0,0).into(),
        };
        let pipe_top_left_tag = GRAPHICS.tags().get("Pipe Top Left");
        let pipe_top_right_tag = GRAPHICS.tags().get("Pipe Top Right");

        let pipe_topleft = object.object_sprite(pipe_top_left_tag.sprite(0));
        let pipe_topright = object.object_sprite(pipe_top_right_tag.sprite(0));

        let pipe = Self {
            height: 0,
            rect: rect,
            top_left: pipe_topleft,
            top_right: pipe_topright,
            leftside: Vec::new(),
            rightside: Vec::new(),
            middle: Vec::new(),
            moving: false,
        };
        pipe
    }
    fn set_height(&mut self, object: &'obj OamManaged<'_>, height:i32) {
        self.height=height;
        let pipe_middle_tag = GRAPHICS.tags().get("Pipe Middle");
        let pipe_left_side_tag = GRAPHICS.tags().get("Pipe Left Side");
        let pipe_right_side_tag = GRAPHICS.tags().get("Pipe Right Side");
        for _ in 0..self.height+1 {
            let pipe_middle = object.object_sprite(pipe_middle_tag.sprite(0));
            self.middle.push(pipe_middle);
        }

        for _ in 0..self.height {
            let pipe_left_side = object.object_sprite(pipe_left_side_tag.sprite(0));
            self.leftside.push(pipe_left_side);

            let pipe_right_side = object.object_sprite(pipe_right_side_tag.sprite(0));
            self.rightside.push(pipe_right_side);
        }

        self.rect.size = (8*3, height*8).into();
    }

    fn show(&mut self) {
        for e in self.leftside.iter_mut() {
            e.show();
        }
        for e in self.rightside.iter_mut() {
            e.show();
        }
        for e in self.middle.iter_mut() {
            e.show();
        }
        self.top_left.show();
        self.top_right.show();
    }

    fn screen_bound(&mut self) {
        let pos = &mut self.rect.pos;
        let size = &self.rect.size;
        if pos.x < -size.x {
            pos.x = WIDTH;
        }
    }

    fn update_pos(&mut self, pos:Vector2D<i32>) {
        self.rect.pos = pos;
        let (x,y) = (self.rect.pos.x, self.rect.pos.y);
        self.top_left.set_position((x,y));
        self.top_right.set_position((x+16,y));
        self.screen_bound();
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
}

struct Bird<'obj> {
    rect: Rect,
    vel: Vector2D<i32>,
    accel: Vector2D<i32>,
    img: Object<'obj>,
    moving: bool,
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
            vel: (0,0).into(),
            accel: (0,0).into(),
            img:bird_img,
            moving:false,
        };
        bird
    }

    fn handle_movement(&mut self, input: &ButtonController) {
        if !self.moving {
            return;
        }
        let mut new_pos = self.rect.pos;
        let scale_factor = 100;
        let y_state = input.just_pressed_y_tri();
        if y_state == Tri::Negative {
            self.accel.y += (y_state as i32) * 3 * scale_factor;
        }
        self.vel.y += self.accel.y/scale_factor;
        new_pos.y += self.vel.y;
        if self.vel.y > 2 {
            self.vel.y -= 1;
        } else if self.vel.y < 2 {
            self.vel.y += 1;
        }
        if self.accel.y > 0 {
            self.accel.y -= 15;
        } else if self.accel.y < 0 {
            self.accel.y += 15;
        }
        if new_pos.y > HEIGHT-self.rect.size.y {
            new_pos.y = HEIGHT-self.rect.size.y;
        }
        if new_pos.y < 0 {
            new_pos.y = 0;
        }
        self.vel.y = self.vel.y.clamp(0, 100);
        self.accel.y = self.accel.y.clamp(-300, 100);
        log::error!("{:?} {:?} {:?}", new_pos, self.vel, self.accel);
        self.rect.pos = new_pos;
        self.img.set_position(self.rect.pos);
    }
}

struct GameState<'obj> {
    frame_counter: u32,
    bird: Bird<'obj>,
    obstacles: Vec<Obstacle<'obj>>,
}

fn gs_init<'obj>(object: &'obj OamManaged<'obj>) -> GameState<'obj>{
    let mut rng = RandomNumberGenerator::new_with_seed([10,13,14,15]);
    let bird = Bird::new(object, 50,HEIGHT*3/4);
    let obstacles = [
        Obstacle::new(object, &mut rng, 0),
        Obstacle::new(object, &mut rng, 0),
    ];
    let obs_vec: Vec<Obstacle> = Vec::from(obstacles);
    let mut r= GameState {
        frame_counter: 0,
        bird,
        obstacles: obs_vec,
    };
    gs_reset(&mut r);
    r
}

fn gs_reset<'obj>(gs: &'obj mut GameState) {
    gs.bird.img.set_position((50,HEIGHT*3/4));
    gs.bird.vel = (0,0).into();
    gs.bird.accel = (0,0).into();
    gs.obstacles[0].set_pos(WIDTH);
    gs.obstacles[1].set_pos(WIDTH/2);
    gs_set_moving(gs, false);
}

fn gs_set_moving<'obj>(gs: &'obj mut GameState, moving: bool) {
    gs.bird.moving = moving;
    gs.obstacles[0].top_pipe.moving = moving;
    gs.obstacles[0].bot_pipe.moving = moving;
    gs.obstacles[1].top_pipe.moving = moving;
    gs.obstacles[1].bot_pipe.moving = moving;
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    mgba_log::init().expect("unable to initialize mGBA logger");
    let mut input = ButtonController::new();
    let object = gba.display.object.get_managed();
    let vblank = VBlank::get();
    let mut gs = gs_init(&object);
    let (gfx, mut vram) = gba.display.video.tiled0();
    let mut bg = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    object.commit();
    set_background(&mut vram, &mut bg);
    bg.commit(&mut vram);
    bg.set_visible(true);
    loop {
        input.update();
        if input.just_pressed_y_tri() != Tri::Zero {
            gs_set_moving(&mut gs, true);
        }
        for e in gs.obstacles.iter_mut() {
            e.move_tick();
        }
        gs.bird.handle_movement(&input);
        let mut reset = false;
        for o in gs.obstacles.iter() {
            if gs.bird.collides(&o.top_pipe) || gs.bird.collides(&o.bot_pipe) {
                reset = true;
                log::error!("reset");
                break;
            }
        }

        if reset {
            gs_reset(&mut gs);
        }

        gs.frame_counter += 1;
        object.commit();
        vblank.wait_for_vblank();
    }
}