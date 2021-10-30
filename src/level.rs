use bevy::prelude::*;
use rand::prelude::SliceRandom;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::cmp;
use std::collections::HashSet;

type Glass = [u8; 4];

trait GlassExt {
    fn info(&self) -> (u8, usize, usize);
    fn is_empty(&self) -> bool;
    fn is_one_color(&self) -> bool;
}

impl GlassExt for Glass {
    /// get the top color, the count for this this color, and the number of empty positions
    fn info(&self) -> (u8, usize, usize) {
        let mut top_color = 0;
        let mut top_count = 0;
        let mut empty_count = 0;
        let mut count = true;
        for i in (0..4).rev() {
            if self[i] > 0 {
                if top_color == 0 {
                    top_color = self[i];
                    top_count = 1;
                } else {
                    if self[i] == top_color {
                        if count {
                            top_count += 1;
                        }
                    } else {
                        // count only first continguous colors
                        count = false;
                    }
                }
            } else {
                empty_count += 1;
            }
        }
        return (top_color, top_count, empty_count);
    }

    /// return true, if the glass is empty
    fn is_empty(&self) -> bool {
        for i in 0..4 {
            if self[i] > 0 {
                return false;
            }
        }
        true
    }

    /// return true, if the glass is full with one color
    fn is_one_color(&self) -> bool {
        let color = self[0];
        if self[0] == 0 {
            return false;
        }
        for i in 1..4 {
            if self[i] != color {
                return false;
            }
        }
        true
    }
}

type Glasses = Vec<Glass>;

pub struct Level {
    loaded: Glasses,
    current: Glasses,
    pub number: usize,
}

fn hex_to_color(color: u32) -> Color {
    Color::rgb(
        (((color >> 16) & 0xff) as f32) / 256.0,
        (((color >> 8) & 0xff) as f32) / 256.0,
        ((color & 0xff) as f32) / 256.0,
    )
}

impl Level {
    pub fn get_color(&self, x: usize, y: usize) -> Option<Color> {
        let index = self.current[x][y];
        let colors = [
            hex_to_color(0xff0000), // red
            hex_to_color(0x4040ff), // blue
            hex_to_color(0xED872D), // orange
            hex_to_color(0xF19CBB), // pink
            hex_to_color(0x54626F), // gray
            hex_to_color(0x343434), // dark gray
            hex_to_color(0x8DB600), // apple green
            hex_to_color(0xBFFF00), // light green
            hex_to_color(0x2e8b57), // sea green
            hex_to_color(0x00FFFF), // cyan
            hex_to_color(0xFFFF00), // yellow
            hex_to_color(0x9400D3), // violet
        ];
        if index == 0 {
            None
        } else {
            Some(colors[(index - 1) as usize])
        }
    }

    pub fn load_next(&mut self) {
        self.number += 1;
        let level = Level::load(self.number);
        self.current = level.current.clone();
        self.loaded = level.loaded.clone();
    }

    fn is_one_glass_same_color(&mut self) -> bool {
        for i in 0..self.current.len() {
            if self.current[i].is_one_color() {
                return true;
            }
        }
        false
    }

    pub fn load(level_number: usize) -> Self {
        // init new level struct
        let mut level = Level {
            loaded: Vec::new(),
            current: Vec::new(),
            number: level_number,
        };

        // special tutorial level
        if level_number == 0 {
            let mut glass = Glass::default();
            glass[0] = 1;
            glass[1] = 1;
            level.loaded.push(glass);
            glass[0] = 1;
            glass[1] = 1;
            level.loaded.push(glass);
            level.restart();
            return level;
        }

        // otherwise create random level, with level number as seed
        let mut rng = Pcg64::seed_from_u64(level_number as u64);
        let max_colors = 12;
        let mut color_count = if level_number < 3 {
            level_number + 1
        } else {
            cmp::min(level_number / 7 + 3, max_colors)
        };
        if color_count > 8 {
            color_count -= rng.gen_range(0..=3);
        }

        // add empty glasses until the level is solvable
        let mut empty_count = 1;
        let mut glass_empty: Glass = Default::default();
        for i in 0..4 {
            glass_empty[i] = 0;
        }
        if color_count == 12 {
            empty_count = 2;
        }
        loop {
            // create linear array with the amount of one glass for each color
            let mut mixed = Vec::new();
            for c in 0..color_count {
                for _ in 0..4 {
                    mixed.push((c + 1) as u8);
                }
            }

            // add empty glasses until the level is solvable
            mixed.shuffle(&mut rng);
            level.loaded = Vec::new();
            for c in 0..color_count {
                let mut glass: Glass = Default::default();
                for i in 0..4 {
                    glass[i] = mixed[c * 4 + i];
                }
                level.loaded.push(glass);
            }
            for _ in 0..empty_count {
                level.loaded.push(glass_empty);
            }

            // break if a solution was found
            level.current = level.loaded.clone();
            let (keys, _size) = level.solve();
            if keys.len() > 0 && !level.is_one_glass_same_color() {
                break;
            } else {
                if empty_count < 2 {
                    empty_count += 1;
                }
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
        let (from_top_color, from_top_count, from_empty_count) = self.current[from].info();

        // get last color of destination glass
        let (to_top_color, _to_top_count, to_empty_count) = self.current[to].info();

        // move, if target is empty, or if it is the same color and if there is enough room
        if to_top_color == 0 || to_top_color == from_top_color && from_top_count <= to_empty_count {
            for i in 0..from_top_count {
                self.current[from][3 - from_empty_count - i] = 0;
                self.current[to][4 - to_empty_count + i] = from_top_color;
            }
            true
        } else {
            false
        }
    }

    pub fn _move_reverse(&mut self, from: usize, to: usize, count: usize) -> bool {
        // get "from" glass info
        let (from_top_color, from_top_count, from_empty_count) = self.current[from].info();

        // test if there is something to move
        if from_empty_count == 4 {
            return false;
        }

        // get "to" glass info
        let (_to_top_color, _to_top_count, to_empty_count) = self.current[to].info();

        // test if there are enough water at the "from" position
        if count > from_top_count {
            return false;
        }

        // test if there is enough room at the target glass
        if to_empty_count < count {
            return false;
        }

        // test if glass is empty after movement
        if from_empty_count + from_top_count < 4 {
            // glass is not empty, test if there remains at least one item of the same color
            if from_top_count == count {
                return false;
            }
        }

        // move water
        for i in 0..count {
            self.current[from][3 - from_empty_count - i] = 0;
            self.current[to][4 - to_empty_count + i] = from_top_color;
        }
        true
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

    pub fn solve(&mut self) -> (String, usize) {
        let mut tested: HashSet<Glasses> = HashSet::new();
        let mut solutions: Vec<Vec<u8>> = Vec::new();
        let mut solution: Vec<u8> = Vec::new();
        self.solve_impl(&mut tested, &mut solutions, &mut solution);
        solutions.sort_by_key(|a| a.len());
        let combinations = tested.len() + 1;
        if solutions.is_empty() {
            ("".to_string(), combinations)
        } else {
            (
                solutions[0].iter().map(|&c| (c + b'a') as char).collect(),
                combinations,
            )
        }
    }
}
