use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use bevy::prelude::*;

type Glass = [u8; 4];
type Glasses = Vec<Glass>;
pub struct Level {
    loaded: Glasses,
    current: Glasses,
}


fn hex_to_color(color: u32) -> Color {
    Color::rgb (
        (((color >> 16) & 0xff) as f32) / 256.0,
        (((color >> 8) & 0xff) as f32) / 256.0,
        ((color & 0xff) as f32) / 256.0
    )
}

impl Level {
    pub fn get_color(&self, x:usize,y:usize) -> Option<Color> {
        let name = self.current[x][y];
        match name {
            b'r' => Some(hex_to_color(0xff0000)), // red
            b'b' => Some(hex_to_color(0x4040ff)), // blue
            b'o' => Some(hex_to_color(0xED872D)), // orange
            b'p' => Some(hex_to_color(0xF19CBB)), // pink
            b'g' => Some(hex_to_color(0x54626F)), // gray
            b'a' => Some(hex_to_color(0x8DB600)), // apple green
            b'l' => Some(hex_to_color(0xBFFF00)), // light green
            b'c' => Some(hex_to_color(0x00FFFF)), // cyan
            b'v' => Some(hex_to_color(0x9400D3)), // violet
            _ => None
        }
    }

    pub fn load(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let lines = io::BufReader::new(file).lines();
        let mut level = Level {
            loaded: Vec::new(),
            current: Vec::new(),
        };
        for line in lines {
            let line = line.unwrap();
            let line = line.trim();
            if line.len() > 0 {
                let mut glass: Glass = Default::default();
                let split: Vec<&str> = line.split("=").collect();
                let colors = split[1].as_bytes();
                for i in 0..4 {
                    if colors.len() > i {
                        glass[i] = colors[i];
                    }
                }
                level.loaded.push(glass);
            }
        }
        level.restart();
        level
    }

    pub fn move_water(&mut self, from: usize, to: usize) -> bool {
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
        for glass in &self.current {
            let c0 = glass[0];
            for c in glass {
                if c0 != *c {
                    return false;
                }
            }
        }
        true
    }

    pub fn restart(&mut self) {
        self.current = self.loaded.clone();
    }

    pub fn number_of_glasses(&self) -> usize {
        self.current.len()
    }

    fn solve_impl(
        &mut self,
        tested: &mut HashSet<Glasses>,
        solutions: &mut Vec<Vec<u8>>,
        solution: &mut Vec<u8>,
    ) {
        fn glass_to_u32(glass: &[u8]) -> u32 {
            (glass[0] as u32)
                | ((glass[1] as u32) << 8)
                | ((glass[2] as u32) << 16)
                | ((glass[3] as u32) << 24)
        }

        // check all possible moves
        let last = self.current.clone();
        let last_solution = solution.clone();
        for from in 0..self.current.len() {
            for to in 0..self.current.len() {
                if from != to {
                    if self.move_water(from, to) {
                        // add move
                        solution.push(from as u8);
                        solution.push(to as u8);

                        // test if winning solution
                        if self.test_win() {
                            solutions.push(solution.clone());
                        } else {
                            // sort copy of glasses
                            let mut copy = self.current.clone();
                            copy.sort_by_key(|a| glass_to_u32(a));

                            // if not already tested, then test it recursively
                            if !tested.contains(&copy) {
                                tested.insert(copy);
                                self.solve_impl(tested, solutions, solution);
                            }
                        }

                        // restore last position
                        self.current = last.clone();
                        *solution = last_solution.clone();
                    }
                }
            }
        }
    }

    pub fn solve(&mut self) -> String {
        let mut tested: HashSet<Glasses> = HashSet::new();
        let mut solutions: Vec<Vec<u8>> = Vec::new();
        let mut solution: Vec<u8> = Vec::new();
        self.solve_impl(&mut tested, &mut solutions, &mut solution);
        solutions.sort_by_key(|a| a.len());
        if solutions.is_empty() {
            "".to_string()
        } else {
            solutions[0].iter().map(|&c| (c + b'a') as char).collect()
        }
    }
}
