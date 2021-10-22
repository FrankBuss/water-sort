use core::time;
use std::io::prelude::*;
use std::io::*;

use crossterm::event::{Event, KeyCode};
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};

mod level;
use crate::level::Level;

fn main() {
    // solution from https://www.youtube.com/watch?v=QPLgSgklwyk
    // "akckfaicdfdjhjgjehbebgbdbkebedgegbcgcicgacakabhahchdfhihicigfa";
    // solution calcuated by this program:
    // "ajakfkdfgdjadjhjehbebgdbadeaebgeagfahfhkbhcbiciaikcicacibd";
    let mut solve_keys = "".to_string();
    let mut solve = false;
    let mut solve_index = 0;
    let mut selected: u8 = 255;

    // load level
    let mut level = Level::load("level2.txt");
    level.restart();

    // timing test
    /*
    let start = Instant::now();
    let solution = level.solve();
    let duration = start.elapsed();
    println!("solution: {}", solution);
    println!("time: {:?}", duration);
    process::exit(0);
    */

    // enable raw mode for keyboard input, and clear screen and hide cursor
    enable_raw_mode().unwrap();
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    // main game loop
    loop {
        level.show(selected);
        execute!(
            stdout(),
            SetForegroundColor(Color::White),
            SetBackgroundColor(Color::Black),
            Print("\r\nA, B ...: move water\r\nR: restart level\r\nS: solve level\r\nESC: end game\r\n"),
        )
        .unwrap();

        if level.test_win() {
            execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::Black),
                Print("You won!\r\n"),
            )
            .unwrap();
            break;
        }
        let mut key = 0;
        if solve {
            key = solve_keys.as_bytes()[solve_index];
            std::thread::sleep(time::Duration::from_millis(200));
            if solve_index < solve_keys.len() - 1 {
                solve_index += 1;
            }
        } else {
            match crossterm::event::read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char(c) => {
                        key = c as u8;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        match key {
            b'r' => {
                level.restart();
                selected = 255;
            }
            b's' => {
                solve_keys = level.solve();
                //println!("{}", solve_keys);
                //break;
                if solve_keys.len() > 0 {
                    level.restart();
                    selected = 255;
                    solve = true;
                }
            }
            _ => {
                if key >= b'a' {
                    let key = key - b'a';
                    if key < level.number_of_glasses() as u8 {
                        if selected == key {
                            selected = 255;
                        } else {
                            if selected < 255 {
                                if level.move_water(selected as usize, key as usize) {
                                    selected = 255;
                                } else {
                                    selected = key;
                                }
                            } else {
                                selected = key;
                            }
                        }
                    }
                }
            }
        }
    }

    execute!(stdout(), cursor::Show,).unwrap();
    disable_raw_mode().unwrap();
}
