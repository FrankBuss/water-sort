use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::*;

use crossterm::style::*;
use crossterm::*;

type Glasses = Vec<Vec<u8>>;
pub struct Level {
    loaded: Glasses,
    current: Glasses,
}

fn hex_to_color(color: u32) -> Color {
    Color::Rgb {
        r: ((color >> 16) & 0xff) as u8,
        g: ((color >> 8) & 0xff) as u8,
        b: (color & 0xff) as u8,
    }
}

fn get_color(name: u8) -> std::result::Result<Color, &'static str> {
    match name {
        b'r' => Ok(hex_to_color(0xff0000)), // red
        b'b' => Ok(hex_to_color(0x4040ff)), // blue
        b'o' => Ok(hex_to_color(0xED872D)), // orange
        b'p' => Ok(hex_to_color(0xF19CBB)), // pink
        b'g' => Ok(hex_to_color(0x54626F)), // gray
        b'a' => Ok(hex_to_color(0x8DB600)), // apple green
        b'l' => Ok(hex_to_color(0xBFFF00)), // light green
        b'c' => Ok(hex_to_color(0x00FFFF)), // cyan
        b'v' => Ok(hex_to_color(0x9400D3)), // violet
        0 => Ok(hex_to_color(0)),           // background
        _ => Err("unknown color name"),
    }
}

impl Level {
    pub fn show(&self, selected: u8) {
        execute!(stdout(), cursor::MoveTo(0, 0)).unwrap();

        // show level
        let rows = if self.current.len() > 4 { 2 } else { 1 };
        let mut first_row = self.current.len() / rows;
        if rows > 1 && self.current.len() - first_row != first_row {
            first_row += 1;
        }

        // https://www.youtube.com/watch?v=QPLgSgklwyk
        let mut i = 0;
        let mut yofs = 0;
        execute!(stdout(), Print("\r\n"),).unwrap();
        for row in 0..rows {
            for y in 0..4 {
                for y2 in 0..1 {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::White),
                        SetBackgroundColor(Color::Black),
                        Print(" "),
                    )
                    .unwrap();
                    for x in 0..first_row {
                        let color = get_color(self.current[i + x][3 - y]).unwrap();
                        execute!(
                            stdout(),
                            SetForegroundColor(Color::White),
                            SetBackgroundColor(Color::Black),
                            Print("│"),
                            SetForegroundColor(Color::Black),
                            SetBackgroundColor(color),
                            Print("   "),
                            SetForegroundColor(Color::White),
                            SetBackgroundColor(Color::Black),
                            Print("│"),
                        )
                        .unwrap();
                    }
                    execute!(stdout(), Print("\r\n"),).unwrap();
                }
            }
            execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::Black),
                Print(" "),
            )
            .unwrap();
            for x in 0..first_row {
                execute!(
                    stdout(),
                    SetForegroundColor(Color::White),
                    SetBackgroundColor(Color::Black),
                    Print("└───┘"),
                )
                .unwrap();
            }
            execute!(stdout(), Print("\r\n "),).unwrap();
            for x in 0..first_row {
                let foreground;
                let background;
                if selected == (i + x) as u8 {
                    foreground = Color::Black;
                    background = Color::White;
                } else {
                    foreground = Color::White;
                    background = Color::Black;
                }
                execute!(
                    stdout(),
                    SetForegroundColor(foreground),
                    SetBackgroundColor(background),
                    Print("  "),
                    Print((i + x + (b'A' as usize)) as u8 as char),
                    Print("  "),
                )
                .unwrap();
            }
            i += first_row;
            first_row = self.current.len() - first_row;
            yofs += 11;
            execute!(stdout(), Print("\r\n\r\n"),).unwrap();
        }
    }

    pub fn load(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let lines = io::BufReader::new(file).lines();
        let level = Level {
            loaded: Vec::new(),
            current: Vec::new(),
        };
        for line in lines {
            let line = line.unwrap();
            let line = line.trim();
            if line.len() > 0 {
                let mut glass: Vec<u8> = Vec::new();
                let split: Vec<&str> = line.split("=").collect();
                let colors = split[1].as_bytes();
                for i in 0..4 {
                    if colors.len() > i {
                        glass.push(colors[i]);
                    } else {
                        glass.push(0);
                    }
                }
                level.loaded.push(glass);
            }
        }
        level
    }

    pub fn move_water(&self, from: usize, to: usize) -> bool {
        // test if there is something to move
        if self.current[from][0] == 0 {
            return false;
        }

        // get last color from where to move
        let mut i = 0;
        let mut from_color = 0;
        while i < 4 {
            if self.current[from][i] > 0 {
                from_color = self.current[from][i];
            } else {
                break;
            }
            i += 1;
        }
        i -= 1;
        let mut from_top = i;

        // count how many to move
        let mut count = 0;
        loop {
            if self.current[from][i] == from_color {
                count += 1;
            } else {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }

        // get last color of destination glass
        i = 0;
        let mut to_color = 0;
        while i < 4 {
            if self.current[to][i] > 0 {
                to_color = self.current[to][i];
            } else {
                break;
            }
            i += 1;
        }

        // move, if target is empty, or if it is the same color and if there is enough room
        if to_color == 0 || to_color == from_color && 4 - i >= count {
            loop {
                self.current[from][from_top] = 0;
                self.current[to][i] = from_color;
                count -= 1;
                if count == 0 {
                    break;
                }
                from_top -= 1;
                i += 1;
            }
            true
        } else {
            false
        }
    }

    pub fn test_win(&self) -> bool {
        // test if all self.current have the same color
        for glass in self.current {
            let c0 = glass[0];
            for c in glass {
                if c0 != *c {
                    return false;
                }
            }
        }
        true
    }

    pub fn restart(&self) {
        self.current = self.loaded;
    }

    pub fn number_of_glasses(&self) -> usize {
        self.current.len()
    }
}
