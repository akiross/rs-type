/* This is a game inspired by ztype to learn how to digit
 *
 * - vim mode: exercise esc as well!
 * - backspace mode: when you make a mistake, that is added to enemy's life
 *
 * */
use ggez::{conf, event, input::keyboard::KeyCode, ContextBuilder};
use std::path::PathBuf;
use structopt::StructOpt;

use rs_type::{wording, Game, Manager, MenuEntry, Quit, StateMap, States, UI};

#[derive(StructOpt, Debug)]
#[structopt(name = "stype")]
struct Options {
    #[structopt(short, long, default_value = "30.0")]
    font_size: f32,

    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 800.0;

fn main() {
    let opt = Options::from_args();
    println!("Options: {:#?}", opt);

    let (mut ctx, mut event_loop) = ContextBuilder::new("rs-type", "Alessandro Re")
        .window_setup(conf::WindowSetup::default().title("RS-Type"))
        .window_mode(conf::WindowMode::default().dimensions(WIDTH, HEIGHT))
        //.add_resource_path(resource_dir)
        .build()
        .expect("aieee, could not create ggez context!");

    // Create a map of states
    let mut states: StateMap = std::collections::HashMap::new();
    let main_menu = vec![
        MenuEntry::VSpace(120.0),
        MenuEntry::Message(60.0, "RS-Type".to_owned()),
        MenuEntry::VSpace(60.0),
        MenuEntry::Transition(40.0, "[S]tart game".to_owned(), KeyCode::S, States::Game),
        MenuEntry::Transition(
            40.0,
            "[Esc] to quit".to_owned(),
            KeyCode::Escape,
            States::Quit,
        ),
    ];

    states.insert(States::StartScreen, Box::new(UI::new(&mut ctx, main_menu)));
    states.insert(
        States::Game,
        Box::new(Game::new(
            whoami::user(),
            wording::KTouchParser::new(opt.file.as_path()),
            opt.font_size,
            &mut ctx,
        )),
    );
    states.insert(
        States::Pause,
        Box::new(UI::new(
            &mut ctx,
            vec![
                MenuEntry::VSpace(400.0),
                MenuEntry::Message(70.0, "PAUSED".to_owned()),
                MenuEntry::Transition(
                    30.0,
                    "[Esc] to unpause".to_owned(),
                    KeyCode::Escape,
                    States::Game,
                ),
                MenuEntry::Transition(
                    30.0,
                    "[Q] to quit game".to_owned(),
                    KeyCode::Q,
                    States::Quit,
                ),
            ],
        )),
    );
    states.insert(
        States::NextLevel,
        Box::new(UI::new(
            &mut ctx,
            vec![
                MenuEntry::VSpace(300.0),
                MenuEntry::Message(70.0, "Level".to_owned()),
                MenuEntry::Message(70.0, "Complete".to_owned()),
            ],
        )),
    );
    states.insert(
        States::Victory,
        Box::new(UI::new(
            &mut ctx,
            vec![
                MenuEntry::VSpace(300.0),
                MenuEntry::Message(70.0, "YOU".to_owned()),
                MenuEntry::Message(70.0, "WON".to_owned()),
            ],
        )),
    );
    states.insert(States::Quit, Box::new(Quit {}));

    let mut state = Manager::new(&mut ctx, States::StartScreen, states);

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
