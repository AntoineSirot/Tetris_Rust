// Adding Librairies

extern crate piston_window;
extern crate rand;

const LARGEUR : u32 = 720;
const HAUTEUR : u32 = 1280;

use piston_window::*;

use rand::seq::SliceRandom;
use rand::thread_rng;

use std::io::BufReader;
use std::fs::File;


// Tetrimino

type Well = [[u8; 10]; 24];

#[derive(Copy, Clone, PartialEq)]
enum TetriminoKind { I, J, L, O, S, T, Z }


#[derive(Copy, Clone)]
struct Tetrimino 
{
    kind: TetriminoKind,
    color: [f32; 4], 
    shape: [[u8; 4]; 4]
}

struct GameState
{
    game_over: bool,
    fall_counter: u32,
    well: Well,
    ttmo_bag: Vec<Tetrimino>,    // 7 Tetriminos aléatoires
    curr_ttmo: Tetrimino,
    next_ttmo: Tetrimino,
    ttmo_row: i32,        
    ttmo_col: i32,
    movment: [bool; 4]    // Gauche, Droite, Rotation, Descente Rapide
}   


impl Tetrimino
{
    const fn new(kind: TetriminoKind) -> Self
    {
        match kind
        {
            //White
            TetriminoKind::I => Tetrimino { kind: TetriminoKind::I, color: [ 1.0, 1.0, 1.0, 1.0 ], shape: [[ 0, 0, 1, 0 ], [ 0, 0, 1, 0 ], [ 0, 0, 1, 0 ], [ 0, 0, 1, 0 ]] },

            //Blue
            TetriminoKind::J => Tetrimino { kind: TetriminoKind::J, color: [ 0.0, 0.0, 1.0, 1.0 ], shape: [[ 1, 0, 0, 0 ], [ 1, 1, 1, 0 ], [ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ]] },

            //Cyan
            TetriminoKind::L => Tetrimino { kind: TetriminoKind::L, color: [ 0.0, 1.0, 1.0, 1.0 ], shape: [[ 0, 0, 1, 0 ], [ 1, 1, 1, 0 ], [ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ]] },

            //Magenta
            TetriminoKind::S => Tetrimino { kind: TetriminoKind::S, color: [ 1.0, 0.0, 1.0, 1.0 ], shape: [[ 0, 1, 1, 0 ], [ 1, 1, 0, 0 ], [ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ]] },

            //Red
            TetriminoKind::Z => Tetrimino { kind: TetriminoKind::Z, color: [ 1.0, 0.0, 0.0, 1.0 ], shape: [[ 1, 1, 0, 0 ], [ 0, 1, 1, 0 ], [ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ]] },

            //Green
            TetriminoKind::O => Tetrimino { kind: TetriminoKind::O, color: [ 0.0, 1.0, 0.0, 1.0 ], shape: [[ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ], [ 0, 1, 1, 0 ], [ 0, 1, 1, 0 ]] },

            //Yellow
            TetriminoKind::T => Tetrimino { kind: TetriminoKind::T, color: [ 1.0, 1.0, 0.0, 1.0 ], shape: [[ 0, 1, 0, 0 ], [ 1, 1, 1, 0 ], [ 0, 0, 0, 0 ], [ 0, 0, 0, 0 ]] }
        }
    }
}


fn game_update(game_state: &mut GameState)
{

    if game_state.fall_counter < 20 {
        game_state.fall_counter += 1;    
    }
    else    
    {
        game_state.fall_counter = 0;

        if would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
        {
            freeze_to_well(&game_state.curr_ttmo, &mut game_state.well, &game_state.ttmo_row, &game_state.ttmo_col);
            game_state.well = clear_complete_rows(game_state.well);

            if game_state.ttmo_bag.is_empty() { game_state.ttmo_bag = bag_aleatoire(); }
            game_state.curr_ttmo = game_state.next_ttmo;
            game_state.next_ttmo = game_state.ttmo_bag.pop().unwrap();

            game_state.ttmo_row = 2;    
            game_state.ttmo_col = 3;   

            if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col)
            {
                game_state.game_over = true;
            }
        }
          
        else { game_state.ttmo_row += 1; }    
    }


    // Gauche
    if game_state.movment[0] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col - 1))
        { game_state.ttmo_col -= 1; }

    // Droite
    if game_state.movment[1] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col + 1))
        { game_state.ttmo_col += 1; }

    // RotateCCW
    if game_state.movment[2] {
        rotate_tetrimino(&mut game_state.curr_ttmo, false);
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, true);
        }
    }

    // Descente
    if game_state.movment[3] && !would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
        { game_state.ttmo_row += 1; }


    game_state.movment = [false; 4];                // Unpressinging key
}

fn rotate_tetrimino(ttmo: &mut Tetrimino, clockwise: bool)
{
    if ttmo.kind == TetriminoKind::O { return; }

    let source = ttmo.shape;
    let mut rotated: [[u8; 4]; 4] = [[0; 4]; 4];

    let matrix_size: usize;
    if ttmo.kind == TetriminoKind::I { matrix_size = 4; } else { matrix_size = 3; }

    for row in 0..matrix_size
    {
        if clockwise {
            for col in 0..matrix_size {
                rotated[col][(matrix_size - 1) - row] = source[row][col];    
            }
        }

        else {
            for col in 0..matrix_size {
                rotated[(matrix_size - 1) - col][row] = source[row][col];
            }
        }
    }

    ttmo.shape = rotated;    
}

fn useable_keys(movment: &mut [bool; 4], touches: ButtonArgs)
{
    match touches.button    
    {
        Button::Keyboard(Key::Left)  => movment[0] = true,    // Gauche
        Button::Keyboard(Key::Right) => movment[1] = true,    // Droite
        Button::Keyboard(Key::Up)    => movment[2] = true,    // Rotation
        Button::Keyboard(Key::Down)  => movment[3] = true,    // Descente Rapide
        _ => ()                                               
    }
}



fn bag_aleatoire() -> Vec<Tetrimino>
{
    let mut tetrimino_bag: Vec<Tetrimino> = vec![ Tetrimino::new(TetriminoKind::I), Tetrimino::new(TetriminoKind::J), Tetrimino::new(TetriminoKind::L), Tetrimino::new(TetriminoKind::O), Tetrimino::new(TetriminoKind::S), Tetrimino::new(TetriminoKind::T), Tetrimino::new(TetriminoKind::Z)  ];
    tetrimino_bag.shuffle(&mut thread_rng()); // 1 seul shuffle par manque de compréhension des 3
    tetrimino_bag
}


fn would_collide(ttmo: &Tetrimino, well: &Well, row: &i32, col: &i32) -> bool
{
    let mut well_row: i32;
    let mut well_col: i32;

    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {

            if ttmo.shape[ttmo_row][ttmo_col] == 0 { continue; }

            well_row = ttmo_row as i32 + *row;
            well_col = ttmo_col as i32 + *col;

            if well_col < 0 { return true; }
            if well_col > 9 { return true; }
            if well_row > 23 { return true; }
    
            if well[well_row as usize][well_col as usize] != 0 { return true; }
        }
    }

    false
}


fn freeze_to_well(ttmo: &Tetrimino, well: &mut Well, well_row: &i32, well_col: &i32)
{
    for row in 0..4 {
        for col in 0..4 {
            if ttmo.shape[row][col] == 0 { continue; }
            well[(*well_row + row as i32) as usize][(*well_col + col as i32) as usize] = ttmo.shape[row][col];
        }
    }
}


fn clear_complete_rows(well: Well) -> Well
{
    let mut new_well: Well = [[0; 10]; 24];
    let mut new_well_row: usize = 23;

    for old_well_row in (0..24).rev()    
    {
        let mut pop_count = 0;
        for col in 0..10 {
            if well[old_well_row][col] != 0 { pop_count += 1; }    
        }

        if pop_count == 0 || pop_count == 10 { continue; }
        if well[old_well_row].iter().sum::<u8>() > 0    
        {    
            new_well[new_well_row] = well[old_well_row];    
            new_well_row -= 1;
        }
    }
    new_well
}

fn render(win: &mut PistonWindow, re: &Event, row: &i32, col: &i32, curr: &Tetrimino, next: &Tetrimino, well: &Well)
{
    win.draw_2d(re, |_context, graphics, _device| { clear([0.5; 4], graphics); } );

    win.draw_2d(re, |context, graphics, _device| { rectangle([0.0, 0.0, 0.0, 1.0], [463.0, -140.0, 354.0, 842.0], context.transform, graphics); } );

    draw_well_blocks(win, re, well);                      // Draw the contents of the playfield.
    draw_tetrimino_well(win, re, row, col, curr);         // Draw the currently falling tetrimino.
}


fn draw_tetrimino_well(win: &mut PistonWindow, re: &Event, well_row: &i32, well_col: &i32, ttmo: &Tetrimino)
{
    let (x, y) = well_to_pixel(*well_row, *well_col);
    draw_tetrimino_pixel(win, re, x, y, ttmo);
}

fn draw_tetrimino_pixel(win: &mut PistonWindow, e: &Event, px: f64, py: f64, ttmo: &Tetrimino)
{

    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {
            
            if ttmo.shape[ttmo_row][ttmo_col] == 0 { continue; }    // No square to be drawn here.

            let x_offs = px + 35.0 * ttmo_col as f64;    // Each square in the Tetrimino is 35x35 pixels.
            let y_offs = py + 35.0 * ttmo_row as f64;    // Pixel Y coords increase downward.

            win.draw_2d(e,
                |context, graphics, _device| {
                    rectangle(ttmo.color, [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
                }
            );
        }
    }
}

fn draw_well_blocks(win: &mut PistonWindow, e: &Event, well: &Well)
{
    for row in 0..24 {
        for col in 0..10 {
            
            if well[row][col] == 0 { continue; }    

            let (x_offs, y_offs) = well_to_pixel(row as i32, col as i32);
            win.draw_2d(e,
                |context, graphics, _device| {
                    rectangle( [1.0, 1.0, 1.0, 1.0], [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
                }
            );
        }
    }
}

fn well_to_pixel(row: i32, col: i32) -> (f64, f64)
{
    ( (col as f64) * 35.0 + 465.0, (row as f64) * 35.0 - 140.0 )
}


fn main()
{
    let mut window: PistonWindow =
        WindowSettings::new("Tetris in Rust", [HAUTEUR, LARGEUR])  
        .vsync(true)
        .build().unwrap();

    window.events.set_ups(30);

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let mut blink_counter = 0;

    let mut starter_bag = bag_aleatoire();
    let starter_first_ttmo = starter_bag.pop().unwrap();
    let starter_second_ttmo = starter_bag.pop().unwrap();

    let mut game_state = GameState {
        game_over: false,
        fall_counter: 0,
        well: [[0u8 ; 10]; 24],
        ttmo_bag: starter_bag,
        curr_ttmo: starter_first_ttmo,
        next_ttmo: starter_second_ttmo,
        ttmo_row: 2,
        ttmo_col: 3,
        movment: [false; 4]
    };


    while let Some(event) = window.next()
    {
        match event
        {
            Event::Loop(Loop::Render(_args_not_used)) => {
                render(&mut window, &event,
                       &game_state.ttmo_row, &game_state.ttmo_col, &game_state.curr_ttmo,
                       &game_state.next_ttmo, &mut game_state.well);
            }

            Event::Loop(Loop::Update(_args_also_not_used)) =>
            {
                if game_state.game_over
                {                    
                    if blink_counter == 15 {
                        game_state.well = [[0u8; 10]; 24];
                    }
                    if blink_counter == 30 {
                        game_state.well = [[1u8; 10]; 24];
                        blink_counter = 0;
                    }
                    blink_counter += 1;
                }
                else {
                    game_update(&mut game_state);
                }
            }

            Event::Input(Input::Button(button_args), _time_stamp) =>
            {                
                if button_args.state == ButtonState::Press {    
                    useable_keys(&mut game_state.movment, button_args);
                }
            }

            _ => {
                ()
            }
        }    
    }    

}   