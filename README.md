# RS-Type - Typing game

This is a game to practice typing. It is inspired to [zty.pe](https://zty.pe/),
but this game is written from scratch in [rust](https://www.rust-lang.org/) using
[ggez](https://ggez.rs/) (which I'm using for the first time, so please be patient
if I'm not using it correctly, e.g. with ECS or something else).

While ztype has been a source of inspiration, the game aims to incorporates other
ideas as well:

1. load [KTouch courses](https://github.com/KDE/ktouch/tree/master/data/courses)
2. (TODO) losing conditions. Basically you can only win or give up :D
3. (TODO) optional hints on which finger to use.
4. (TODO) various modes to practice Esc, Backspace, Enter, Caps, etc.
5. (TODO) correct mode, requires player to correct typing errors to go on

Tested with rust 1.41 and 1.45 nightly.

## Playing

Start the game downloading the courses you want from KTouch, for example `us.xml`

    $ curl -O https://raw.githubusercontent.com/KDE/ktouch/master/data/courses/us.xml
    $ cargo run --bin game -- us.xml 

The game is not really complete (and maybe it will never be), take that into
consideration :)

![Game screenshot](https://github.com/akiross/bors/raw/master/screenshots/rs-type/game.png)

## Painting tool

To draw the background, I made a simple tool that helps drawing with triangles.
It's very rudimentary: you give a fixed palette, a save/load file and it will
automatically load it on start and save to it on exit (by pressing Escape).
I'll describe it here because it's not really user friendly.

![Paint tool](https://github.com/akiross/bors/raw/master/screenshots/rs-type/paint.png)

It's not a serious tool, but you might have fun with it or improve it. Run with

    $ cargo run --bin paint -- io.txt --paint-colors "#ff0000" "#00ff00" "#0000ff"

This will open a window where you can paint drawing triangles. Please note that
triangles are drawn in batches, ordered by colors, so first color triangles are
below second color triangles, etc.

Use the keyboard to change current state:

 - Press N to cycle through colors
 - Press X to remove the triangle under cursor
 - Press S to toggle snap (on by default)
 - Press - and = to increase and decrease snap distance
 - Press W to toggle wireframes (there are rendering issues, expect errors)
 - Press Esc to cancel triangle being drawn or, if none, to quit
 - Press HJKL to translate the scene and IO to zoom

Snap, when enabled, will pick the nearest point within a radius from the cursor.
The current snap size is represented by the width/height of the cursor triangle(s).
When enabled, the cursor is two triangles pointing to cursor position, else one.

The file path provided will be used to load colors and triangles (if exists) and
will be used as output target to save the triangles and colors.
When Esc is pressed to quit, the current triangles and colors will be written on
it. If you close by quit event (that is, close the window on your WM), it will
discard current edits and don't write to file.

If the file is empty, `--paint-colors` is required with at least one color,
otherwise the program will fail. If it's not empty, colors are loaded from it,
and if `--paint-colors` are specified, they will be merged.
