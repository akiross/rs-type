/* This is a game inspired by ztype to learn how to digit
 *
 * - vim mode: exercise esc as well!
 * - backspace mode: when you make a mistake, that is added to enemy's life
 *
 * */
use ggez::{conf, event, ContextBuilder};
use structopt::StructOpt;

use rs_type::{Manager, Paint, StateMap, States};

#[derive(StructOpt, Debug)]
#[structopt(name = "stype")]
struct Options {
    #[structopt()]
    paint_file: String,

    #[structopt(long, name = "colors", min_values = 1)]
    paint_colors: Vec<String>,

    #[structopt(long, default_value = "15.0")]
    margin: f32,

    #[structopt(long, short, default_value = "600.0")]
    width: f32,

    #[structopt(long, short, default_value = "800.0")]
    height: f32,

    #[structopt(long)]
    read_absolute: bool,

    #[structopt(long)]
    write_absolute: bool,
}

fn main() {
    let opt = Options::from_args();
    println!("Options: {:#?}", opt);

    let (mut ctx, mut event_loop) = ContextBuilder::new("rs-type", "Alessandro Re")
        .window_setup(conf::WindowSetup::default().title("Paint Triangles"))
        .window_mode(
            conf::WindowMode::default()
                .dimensions(opt.width + 2.0 * opt.margin, opt.height + 2.0 * opt.margin),
        )
        .build()
        .expect("aieee, could not create ggez context!");

    ggez::input::mouse::set_cursor_hidden(&mut ctx, true);

    // Create a map of states
    let mut states: StateMap = std::collections::HashMap::new();
    states.insert(
        States::Paint,
        Box::new(Paint::new(
            &mut ctx,
            opt.margin,
            opt.paint_colors,
            opt.paint_file,
            opt.read_absolute,
            opt.write_absolute,
        )),
    );

    let mut state = Manager::new(&mut ctx, States::Paint, states);

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
