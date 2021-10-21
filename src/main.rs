use std::io::prelude::*;
use std::io::*;

use crossterm::event::*;
use crossterm::style::*;
use crossterm::terminal::*;
use crossterm::*;

mod level;
use crate::level::*;

fn main() -> crossterm::Result<()> {
    enable_raw_mode().unwrap();
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0)
    )?;

    let level = Level::load("level2.txt");
    level.restart();

    let mut selected: u8 = 255;
    loop {
        level.show(selected);
        execute!(
            stdout(),
            SetForegroundColor(Color::White),
            SetBackgroundColor(Color::Black),
            Print("\r\nA, B ...: move water\r\nR: restart level\r\nESC: end game\r\n"),
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
        match key {
            b'r' => {
                level.restart();
                selected = 255;
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

    execute!(
        stdout(),
        cursor::Show,
    )?;
    disable_raw_mode().unwrap();

    Ok(())
}
