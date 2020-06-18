//! Cool stuff and states

mod drawing;
mod objects;
pub mod wording;

use objects::{Enemy, Player};
use wording::{Enemies, WordProducer};

use drawing::ColoredTriangles;

use std::io::Write;

use ggez::event::EventHandler;
use ggez::nalgebra as na;
use ggez::{
    graphics,
    input::keyboard::{KeyCode, KeyMods},
    input::mouse::MouseButton,
    Context, GameResult,
};

use rand::prelude::*;
use rand_distr::{Bernoulli, Distribution, Exp};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum States {
    StartScreen,
    Game,
    Pause,
    NextLevel,
    Victory,
    Quit,
    Paint,
}

pub trait State: EventHandler {
    fn next_state(&mut self) -> Option<States>;
}

// A map from enum to EventHandler
pub type StateMap = std::collections::HashMap<States, Box<dyn State>>;

/// Manages the state of the game, also rendering background
pub struct Manager {
    stars: Vec<(f32, f32)>,
    planets: graphics::Mesh,
    space: graphics::Color,

    states: StateMap,
    current_state: States,
}

impl Manager {
    pub fn new(ctx: &mut Context, initial_state: States, states: StateMap) -> Manager {
        let (width, height) = graphics::size(ctx);
        let exp = Exp::new(2.0).unwrap();
        let mut rng = thread_rng();
        let stars = (0..60)
            .map(|_| {
                (
                    rng.gen_range(0, width as usize) as f32,
                    width * 0.9 * exp.sample(&mut rng),
                )
            })
            .collect();
        let mut planets: ColoredTriangles = include_str!("planets.txt").into();
        // Assume it's normalized
        planets.scale(width, height);
        // println!("Planets {:?}", planets.triangles[0]);
        let mut builder = graphics::MeshBuilder::new();
        for i in 0..planets.colors.len() {
            if !planets.triangles[i].is_empty() {
                builder
                    .triangles(&planets.triangles[i], planets.colors[i])
                    .expect("Unable to add triangles to builder");
            }
        }
        let planets = builder.build(ctx).expect("Unable to build planets mesh");

        // TODO prepare here option texts
        Manager {
            stars,
            planets,
            current_state: initial_state,
            states,
            space: (22, 2, 33).into(),
        }
    }
}

impl EventHandler for Manager {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        let res = st.update(ctx);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
        res
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Draw background
        graphics::clear(ctx, self.space);

        let mut mb = graphics::MeshBuilder::new();
        // Draw stars below everything, these are animated so we render
        // them here
        let mut rng = thread_rng();
        let shine = Bernoulli::new(0.01).unwrap();
        let l = 3.0;
        for s in &self.stars {
            let w = if shine.sample(&mut rng) { 2.0 } else { 0.0 };
            let (x, y) = *s;
            mb.line(
                &[[x - l - w, y], [x + l + w, y]],
                1.0,
                (255, 255, 255).into(),
            )
            .expect("Unable to build star line")
            .line(
                &[[x, y - l - w], [x, y + l + w]],
                1.0,
                (255, 255, 255).into(),
            )
            .expect("Unable to build star line");
        }
        let stars = mb.build(ctx)?;
        // Draw stars planets
        graphics::draw(ctx, &stars, graphics::DrawParam::default())?;
        graphics::draw(ctx, &self.planets, graphics::DrawParam::default())?;

        // Let state to draw the rest
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        st.draw(ctx)
    }
    fn text_input_event(&mut self, ctx: &mut Context, ch: char) {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        st.text_input_event(ctx, ch);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
    }
    /// Called when Esc is pressed
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        st.key_down_event(ctx, keycode, keymods, repeat);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
    }
    /// Called when mouse button is pressed
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        st.mouse_motion_event(ctx, x, y, dx, dy);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
    }
    /// Called when mouse button is pressed
    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        st.mouse_button_down_event(ctx, button, x, y);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
    }
    /// Called when window is closed
    fn quit_event(&mut self, ctx: &mut Context) -> bool {
        let st = self
            .states
            .get_mut(&self.current_state)
            .expect("Cannot get state");
        let qe = st.quit_event(ctx);
        if let Some(ns) = st.next_state() {
            self.current_state = ns
        }
        qe
    }
}

pub struct Quit;
impl EventHandler for Quit {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        ggez::event::quit(ctx);
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}

impl State for Quit {
    fn next_state(&mut self) -> Option<States> {
        None
    }
}

pub enum MenuEntry {
    /// A message that displays a message and transitions to another state when key is pressed
    Transition(f32, String, KeyCode, States),
    /// Just a message text, no transition
    Message(f32, String),
    /// Some vertical space between entries
    VSpace(f32),
}

pub struct UI {
    goto_state: Option<States>,
    // This is a vector of things to show on the screen
    options: Vec<MenuEntry>,
    font: graphics::Font,
}

impl UI {
    pub fn new(ctx: &mut Context, options: Vec<MenuEntry>) -> UI {
        let font = ggez::graphics::Font::new(ctx, "VCRFont.ttf")
            .map_err(|e| {
                println!("Unable to find VCRFont.ttf");
                e
            })
            .unwrap_or_default();
        UI {
            goto_state: None,
            options,
            font,
        }
    }
}

impl EventHandler for UI {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, _) = graphics::size(ctx);
        // Draw options
        let mut top = 0.0;
        for opt in self.options.iter() {
            match opt {
                MenuEntry::Transition(sc, msg, _, _) | MenuEntry::Message(sc, msg) => {
                    let frag = graphics::TextFragment::new(msg.clone());
                    let frag = frag.scale(graphics::Scale::uniform(*sc)).font(self.font);
                    let text = graphics::Text::new(frag);
                    let (w, y) = text.dimensions(ctx);

                    let left = (width - w as f32) * 0.5;

                    graphics::draw(
                        ctx,
                        &text,
                        graphics::DrawParam::default().dest(na::Point2::new(left, top)),
                    )?;
                    top += y as f32;
                }
                MenuEntry::VSpace(y) => {
                    top += *y as f32;
                }
            };
        }

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        // Check if the pressed char is part of the menu
        for opt in self.options.iter() {
            if let MenuEntry::Transition(_, _, k, state) = opt {
                if *k == keycode {
                    self.goto_state = Some(state.clone());
                    break;
                }
            }
        }
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        println!("UI received quit event! Returning false to quit");
        false // Quitting
    }
}

impl State for UI {
    fn next_state(&mut self) -> Option<States> {
        self.goto_state.take()
    }
}

fn random_enemy_position(width: f32) -> f32 {
    let mut rng = thread_rng();
    rng.gen_range(width * 0.1, width * 0.9) as f32
}

pub struct Game<P: WordProducer> {
    goto_state: Option<States>,
    level: usize,
    time: std::time::Instant,
    players: Vec<Player>,
    enemies: Vec<Enemy>,
    target: Option<Enemy>, // The current enemy being targeted
    sequence: usize,       // Where do we start?
    producer: P,
    font_size: f32,
    //background: graphics::Image,
}

impl<P: WordProducer> Game<P> {
    pub fn new(name: String, word_producer: P, font_size: f32, _ctx: &mut Context) -> Game<P> {
        // TODO text can be prepared here so it's faster in-game
        Game {
            goto_state: None,
            level: 0,
            time: std::time::Instant::now(),
            players: vec![Player {
                pos_x: 300.0,
                pos_y: 700.0,
                name,
                font_size,
            }],
            enemies: Vec::new(),
            target: None,
            sequence: 0,
            producer: word_producer,
            font_size,
            //background: graphics::Image::new(ctx, "/background.png")
            //    .expect("Cannot load background"),
        }
    }
}

impl<P: WordProducer> EventHandler for Game<P> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Move enemies a bit towards me
        for en in &mut self.enemies {
            en.pos_y += en.speed * (1 + self.level) as f32;
        }
        if let Some(en) = self.target.as_mut() {
            en.pos_y += en.speed * (1 + self.level) as f32;
        }

        match self.producer.next_word(self.level, self.sequence) {
            Enemies::GameComplete => {
                if self.enemies.is_empty() && self.target.is_none() {
                    self.goto_state = Some(States::Victory);
                } else {
                    // There are still enemies to kill...
                }
            }
            Enemies::LevelComplete => {
                if self.enemies.is_empty() && self.target.is_none() {
                    self.level += 1;
                    self.sequence = 0;
                } else {
                    // There are still enemies to kill...
                }
            }
            Enemies::Some(word) => {
                // Add enemies if none is present or enough time passed
                if self.time.elapsed().as_secs() >= 2 || self.enemies.is_empty() {
                    //let elap = self.start.elapsed().as_millis();
                    let (width, _) = graphics::size(ctx);
                    self.enemies.push(Enemy {
                        pos_x: random_enemy_position(width),
                        pos_y: -1.0,
                        speed: 0.5,
                        word,
                        font_size: self.font_size,
                    });
                    self.time = std::time::Instant::now();
                    self.sequence += 1;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // graphics::clear(ctx, graphics::BLACK);
        //graphics::draw(ctx, &self.background, graphics::DrawParam::default())?;

        let enemy_color = (0xd2, 0xd5, 0x3b).into();
        let target_color = (0xe0, 0x56, 0x2c).into();
        let player_color = (0xff, 0x00, 0xff).into();

        let (width, height) = graphics::size(ctx);

        // Draw text
        // TODO(low): creating enemies every time is expensive, cache it!
        for en in &self.enemies {
            en.draw(ctx, enemy_color)?;
        }
        if let Some(en) = &self.target {
            en.draw(ctx, target_color)?;
        }

        // Draw yourself
        for pl in &self.players {
            pl.draw(ctx, player_color)?;
        }

        let lvl = graphics::Text::new(format!("Level {}", self.level));
        let (w, h) = lvl.dimensions(ctx);
        graphics::draw(
            ctx,
            &lvl,
            (na::Point2::new(width - w as f32, height - h as f32),),
        )?;

        graphics::present(ctx)
    }

    fn text_input_event(&mut self, _ctx: &mut Context, ch: char) {
        /*
        // Just to see what has been pressed
        if ch == '\u{0008}' {
            // If backspace is pressed, remove the character
            self.players[0].word.pop();
        } else {
            self.players[0].word.push(ch);
        }
        */

        if let Some(enemy) = self.target.as_mut() {
            // There's a current target, hit that one
            if let Some(c) = enemy.word.chars().next() {
                // Make sure input is valid
                if c == ch {
                    enemy.word.remove(0);
                } else {
                    // TODO Show user error (e.g. color enemy)
                    // TODO "correction mode": when input is wrong, add it to enemy making it
                    // stronger
                }
            }
        } else {
            // Pick enemy to attack
            let mut i = 0;
            while i != self.enemies.len() {
                if let Some(c) = self.enemies[i].word.chars().next() {
                    if c == ch {
                        let mut enemy = self.enemies.remove(i);
                        enemy.word.remove(0);
                        self.target = Some(enemy);
                        break;
                    }
                }
                i += 1;
            }
        }
        // The old enemy might have been killer, as well as 1-char enemies
        if let Some(enemy) = self.target.as_mut() {
            if enemy.word.is_empty() {
                self.target = None;
                // TODO destroy animation? How?
                // self.players[0].word.clear();
            }
        }
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        //println!("KEY DOWN {:?}", keycode);
        // When Escape is pressed, don't quit the app immediately (default key_down_event), but
        // go to pause state
        if keycode == KeyCode::Escape {
            self.goto_state = Some(States::Pause);
        }
    }
}

impl<P: WordProducer> State for Game<P> {
    fn next_state(&mut self) -> Option<States> {
        self.goto_state.take()
    }
}

pub struct Paint {
    // Colored triangles used to draw
    ct: ColoredTriangles,
    // Current color (index) for triangles being built
    cur_color: usize,
    // Triangle being built
    poly: Vec<na::Point2<f32>>,
    // Current mouse position
    m_pos: na::Point2<f32>,
    // An optional background image
    background: Option<graphics::Image>,
    // Whether to normalize upon save
    write_absolute: bool,
    // Margin of drawing area
    margin: f32,
    // Snap distance
    snap_dist: f32,
    // Snap enabled
    use_snap: bool,
    // Draw with wireframes
    wireframe: bool,
    // File to read and write
    filename: String,
}

impl Paint {
    pub fn new(ctx: &mut Context, margin: f32, colors: Vec<String>, filename: String, read_absolute: bool, write_absolute: bool) -> Self {
        // Read file as string to build colored triangles
        let data = if filename.is_empty() {
            "".to_owned()
        } else {
            std::fs::read_to_string(&filename).unwrap_or_else(|_| "".to_owned())
        };
        let mut ct: ColoredTriangles = data.as_str().into();
        // Scale data if reading is relative
        if !read_absolute {
            let (width, height) = graphics::size(ctx);
            ct.scale(width - 2.0 * margin, height - 2.0 * margin);
        }
        // Add colors to ct, if some were missing
        colors.iter().for_each(|c| ct.add_color(c));

        // TODO load background as reference
        //let background =
        //    graphics::Image::new(ctx, "/background.png").expect("Cannot find background.png");

        Paint {
            ct,
            poly: vec![],
            cur_color: 0,
            m_pos: na::Point2::new(0.0, 0.0),
            background: None, //Some(background),
            snap_dist: 10.0,
            use_snap: true,
            filename,
            wireframe: false,
            margin,
            write_absolute,
        }
    }

    /// Shift points according to margin
    fn mshift(&self, points: &[na::Point2<f32>]) -> Vec<na::Point2<f32>> {
        // Compute margin vector
        let margin = na::Point2::new(self.margin, self.margin).coords;
        // Offset each point by margin
        points.iter().map(|p| p + margin).collect::<Vec<_>>()
    }
}

impl EventHandler for Paint {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(img) = &self.background {
            graphics::draw(
                ctx,
                img,
                graphics::DrawParam::default().dest(na::Point2::new(self.margin, self.margin)),
            )?;
        } else {
            graphics::clear(ctx, graphics::BLACK);
        }

        let mut builder = graphics::MeshBuilder::new();
        for i in 0..self.ct.colors.len() {
            if !self.ct.triangles[i].is_empty() {
                // Offset triangle points by margin
                let tri = self.mshift(&self.ct.triangles[i]);
                if self.wireframe {
                    builder.polyline(graphics::DrawMode::stroke(1.0), &tri, self.ct.colors[i])?;
                } else {
                    builder.triangles(&tri, self.ct.colors[i])?;
                }
            }
        }
        match self.poly.len() {
            1 => {
                let mut poly = self.poly.clone();
                poly.push(self.m_pos);
                // Lines and polylines are crashing right now, so I'm making
                // a small triangle to simulate that
                poly.push(na::Point2::new(poly[0].x, poly[0].y - 5.0));
                // Offset poly by margin
                let poly = self.mshift(&poly);
                if self.wireframe {
                    builder.polyline(
                        graphics::DrawMode::stroke(1.0),
                        &poly,
                        self.ct.colors[self.cur_color],
                    )?;
                } else {
                    builder.triangles(&poly, self.ct.colors[self.cur_color])?;
                }
            }
            2 => {
                let mut poly = self.poly.clone();
                poly.push(self.m_pos);
                // Offset poly by margin
                let poly = self.mshift(&poly);
                if self.wireframe {
                    builder.polyline(
                        graphics::DrawMode::stroke(1.0),
                        &poly,
                        self.ct.colors[self.cur_color],
                    )?;
                } else {
                    builder.triangles(&poly, self.ct.colors[self.cur_color])?;
                }
            }
            _ => {}
        }

        // Draw a triangle as a pointer, over other objects for better viewing
        let mut pointer = vec![
            self.m_pos,
            na::Point2::new(self.m_pos.x + self.snap_dist, self.m_pos.y),
            na::Point2::new(self.m_pos.x, self.m_pos.y + self.snap_dist),
        ];
        if self.use_snap {
            pointer.append(&mut vec![
                self.m_pos,
                na::Point2::new(self.m_pos.x - self.snap_dist, self.m_pos.y),
                na::Point2::new(self.m_pos.x, self.m_pos.y - self.snap_dist),
            ]);
        }
        // Offset by margin
        let pointer = self.mshift(&pointer);

        builder.triangles(&pointer, self.ct.colors[self.cur_color])?;

        // At least the cursor is being drawn
        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;

        graphics::present(ctx)
    }
    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        // Correct cursor position using margin
        self.m_pos = na::Point2::new(x - self.margin, y - self.margin);
    }
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let add = match button {
            MouseButton::Left => true,
            _ => false,
        };
        // Get mutable access to x and y, with margins removed, for snapping
        let mut x = x - self.margin;
        let mut y = y - self.margin;
        if self.use_snap {
            // Snap mouse coords to existing points
            let nearest = self.ct.nearest(1, &na::Point2::new(x, y));
            if !nearest.is_empty() {
                let (col_i, pt_i, sqd) = nearest[0];
                if sqd <= self.snap_dist * self.snap_dist {
                    let p = self.ct.triangles[col_i][pt_i];
                    x = p.x;
                    y = p.y;
                }
            }
        }
        match self.poly.len() {
            0 | 1 if add => {
                self.poly.push(na::Point2::new(x, y));
            }
            2 if add => {
                self.poly.push(na::Point2::new(x, y));
                self.ct.triangles[self.cur_color].append(&mut self.poly);
            }
            _ => {
                // Do nothing, this case shouldn't even happen
            }
        }
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => {
                // If a triangle is being created, remove it
                if !self.poly.is_empty() {
                    self.poly.clear();
                } else {
                    println!(
                        "Saving list of triangles by color over file {}, absolute coords: {}",
                        self.filename, self.write_absolute
                    );
                    if !self.write_absolute {
                        let (width, height) = graphics::size(ctx);
                        let width = width - self.margin * 2.0;
                        let height = height - self.margin * 2.0;
                        self.ct.scale(1.0 / width, 1.0 / height);
                    }
                    let mut f = std::fs::File::create(&self.filename).expect("Cannot create file");
                    for i in 0..self.ct.colors.len() {
                        writeln!(
                            &mut f,
                            "{} {}",
                            self.ct.colors[i].to_rgb_u32(),
                            self.ct.triangles[i]
                                .iter()
                                .map(|p| format!("{},{}", p.x, p.y))
                                .collect::<Vec<String>>()
                                .join(" ")
                        )
                        .expect("Unable to write to file");
                    }
                    ggez::event::quit(ctx);
                }
            }
            KeyCode::N => {
                let nc = (self.cur_color + 1) % self.ct.colors.len();
                self.cur_color = nc;
                println!("Current color {:?}", self.ct.colors[nc]);
            }
            KeyCode::S => {
                self.use_snap = !self.use_snap;
                println!(
                    "Toggling snap. Snap is now {}",
                    if self.use_snap { "on" } else { "off" }
                );
            }
            KeyCode::W => {
                self.wireframe = !self.wireframe;
            }
            KeyCode::X => {
                // Get triangle under cursor and delete it
                if let Some((c, i)) = self.ct.colliding(&self.m_pos) {
                    println!("Deleting triangle {} {}", c, i);
                    self.ct.remove_triangle(c, i);
                }
            }
            KeyCode::Equals => {
                self.snap_dist += 1.0;
            }
            KeyCode::Subtract => {
                self.snap_dist -= 1.0;
            }
            KeyCode::H => { self.ct.translate(-self.snap_dist, 0.0); }
            KeyCode::J => { self.ct.translate(0.0, self.snap_dist); }
            KeyCode::K => { self.ct.translate(0.0, -self.snap_dist); }
            KeyCode::L => { self.ct.translate(self.snap_dist, 0.0); }
            KeyCode::I => { self.ct.scale(1.25, 1.25); }
            KeyCode::O => { self.ct.scale(1.0 / 1.25, 1.0 / 1.25); }
            k => {
                println!("Key code pressed {:?}", k);
            }
        }
    }
}

impl State for Paint {
    fn next_state(&mut self) -> Option<States> {
        None
    }
}
