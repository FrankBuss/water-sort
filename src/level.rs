/*
file format, per level:
u8 number of glasses
vec<vec<u8>> level data
*/
use bevy::prelude::*;
use rand::prelude::SliceRandom;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, prelude::*, SeekFrom};

use crate::level;

type Glass = Vec<u8>;

const MAX_COLORS: usize = 12;
const MAX_GLASSES_PER_LEVEL: usize = 16;

pub struct Move {
    pub from: usize,
    pub to: usize,
}

trait GlassExt {
    fn info(&self) -> (u8, usize, usize);
    fn is_empty(&self) -> bool;
    fn is_one_color(&self) -> bool;
    fn to_number(&self) -> u64;
}

impl GlassExt for Glass {
    /// get the top color, the count for this this color, and the number of empty positions
    fn info(&self) -> (u8, usize, usize) {
        let mut top_color = 0;
        let mut top_count = 0;
        let mut empty_count = 0;
        let mut count = true;
        for i in (0..self.len()).rev() {
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
        self.iter().any(|val| *val > 0)
    }

    /// return true, if the glass is full with one color
    fn is_one_color(&self) -> bool {
        let color = self[0];
        if self[0] == 0 {
            return false;
        }
        for i in 1..self.len() {
            if self[i] != color {
                return false;
            }
        }
        true
    }

    /// convert the elements of this glass to one number for sorting
    fn to_number(&self) -> u64 {
        let mut result: u64 = 0;
        for i in 0..self.len() {
            result <<= 8;
            result |= self[i] as u64;
        }
        result
    }
}

type Glasses = Vec<Glass>;

pub struct Level {
    loaded: Glasses,
    current: Glasses,
    pub number: usize,
    pub glass_height: usize,
    pub moves: Vec<Move>,
    pub move_counts: Vec<usize>,
}

fn hex_to_color(color: u32) -> Color {
    Color::rgb(
        (((color >> 16) & 0xff) as f32) / 255.0,
        (((color >> 8) & 0xff) as f32) / 255.0,
        ((color & 0xff) as f32) / 255.0,
    )
}

impl Level {
    pub fn new(level_number: usize, glass_height: usize) -> Self {
        Level {
            loaded: Vec::new(),
            current: Vec::new(),
            number: level_number,
            glass_height: glass_height,
            moves: Vec::new(),
            move_counts: Vec::new(),
        }
    }

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
        (index > 0).then(|| colors[(index - 1) as usize])
    }

    pub fn load_next(&mut self) {
        self.number += 1;
        let level = Level::load(self.number, self.glass_height);
        self.current = level.current.clone();
        self.loaded = level.loaded.clone();
    }

    fn is_one_glass_same_color(&mut self) -> bool {
        self.current.iter().any(Glass::is_one_color)
    }

    pub fn load(level_number: usize, glass_height: usize) -> Self {
        let mut level = Level::new(level_number, glass_height);

        let filename = Level::create_levels_filename(glass_height);
        let mut file = File::open(filename).unwrap();
        let ofs = ((MAX_GLASSES_PER_LEVEL * glass_height + 1) * level_number) as u64;
        file.seek(SeekFrom::Start(
            ofs,
        ))
        .unwrap();

        let mut byte = [0u8; 1];
        file.read_exact(&mut byte).unwrap();
        for _ in 0..byte[0] as usize {
            let mut glass = vec![0; glass_height];
            file.read_exact(&mut glass).unwrap();
            level.loaded.push(glass);
        }

        level.restart();
        level
    }

    pub fn create(level_number: usize, glass_height: usize) -> Self {
        // init new level struct
        let mut level = Level {
            loaded: Vec::new(),
            current: Vec::new(),
            number: level_number,
            glass_height: glass_height,
            moves: Vec::new(),
            move_counts: Vec::new(),
        };

        // special tutorial level
        if level_number == 0 {
            let mut glass1 = vec![0; level.glass_height];
            let half1 = level.glass_height / 2;
            for i in 0..half1 {
                glass1[i] = 1;
            }
            level.loaded.push(glass1);

            let half2 = level.glass_height - half1;
            let mut glass2 = vec![0; level.glass_height];
            for i in 0..half2 {
                glass2[i] = 1;
            }
            level.loaded.push(glass2);

            level.restart();
            return level;
        }

        // otherwise create random level, with level number as seed
        let mut rng = Pcg64::seed_from_u64(level_number as u64);
        let color_count = match level_number {
            0..=3 => level_number + 1,
            _ => {
                let c = (level_number / 7 + 3).min(MAX_COLORS);
                (c > 8).then(|| c - rng.gen_range(0..=3)).unwrap_or(c)
            }
        };

        // add empty glasses until the level is solvable
        let mut empty_count = 1;
        let glass_empty: Glass = vec![0; level.glass_height];
        if color_count == MAX_COLORS {
            empty_count = 2;
        }
        loop {
            // create linear array with the amount of one glass for each color
            let mut mixed = (0..color_count)
                .flat_map(|c| vec![c as u8 + 1; level.glass_height])
                .collect::<Vec<_>>();

            // shuffle
            mixed.shuffle(&mut rng);

            // fill level with shuffled colors
            level.loaded = Vec::new();
            for c in 0..color_count {
                let mut glass: Glass = vec![0; level.glass_height];
                for i in 0..level.glass_height {
                    glass[i] = mixed[c * level.glass_height + i];
                }
                level.loaded.push(glass);
            }

            // add empty glasses until the level is solvable
            level.loaded.extend(vec![glass_empty.clone(); empty_count]);

            // break if a solution was found
            level.current = level.loaded.clone();
            let (keys, _size) = level.solve();
            if keys.len() > 0 && !level.is_one_glass_same_color() {
                break;
            } else if empty_count < 5 {
                empty_count += 1;
            }
        }
        level.restart();
        level
    }

    pub fn move_water(&mut self, from: usize, to: usize, save_move: bool) -> bool {
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
            if save_move {
                let last_move = Move { from: from, to: to };
                self.moves.push(last_move);
                self.move_counts.push(from_top_count);
            }
            for i in 0..from_top_count {
                self.current[from][self.glass_height - 1 - from_empty_count - i] = 0;
                self.current[to][self.glass_height - to_empty_count + i] = from_top_color;
            }
            true
        } else {
            false
        }
    }

    pub fn move_reverse(&mut self, from: usize, to: usize, count: usize) -> bool {
        // get "from" glass info
        let (from_top_color, from_top_count, from_empty_count) = self.current[from].info();

        // test if there is something to move
        if from_empty_count == self.glass_height {
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

        // if glass is not empty, test if there remains at least one item of the same color
        if from_empty_count + from_top_count < self.glass_height && from_top_count == count {
            return false;
        }

        // move water
        for i in 0..count {
            self.current[from][self.glass_height - 1 - from_empty_count - i] = 0;
            self.current[to][self.glass_height - to_empty_count + i] = from_top_color;
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
        // check all possible moves
        let last = self.current.clone();
        let last_solution = solution.clone();
        for from in 0..self.current.len() {
            for to in 0..self.current.len() {
                if from == to {
                    continue;
                }
                if self.move_water(from, to, false) {
                    // add move
                    solution.push(from as u8);
                    solution.push(to as u8);

                    // test if winning solution
                    if self.test_win() {
                        solutions.push(solution.clone());
                    } else {
                        // sort copy of glasses
                        let mut copy = self.current.clone();
                        copy.sort_by_key(|a| a.to_number());

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

    pub fn solve(&mut self) -> (String, usize) {
        let mut tested: HashSet<Glasses> = HashSet::new();
        let mut solutions: Vec<Vec<u8>> = Vec::new();
        let mut solution: Vec<u8> = Vec::new();
        self.solve_impl(&mut tested, &mut solutions, &mut solution);
        let combinations = tested.len() + 1;
        if solutions.is_empty() {
            ("".to_string(), combinations)
        } else {
            solutions.sort_by_key(|a| a.len());
            (
                solutions[0].iter().map(|&c| (c + b'a') as char).collect(),
                combinations,
            )
        }
    }

    pub fn resize(&mut self, size: usize) {
        let level2 = Level::load(self.number, size);
        self.glass_height = size;
        self.loaded = level2.loaded.clone();
        self.restart();
    }

    pub fn undo(&mut self) -> bool {
        let moves = self.moves.len();
        if moves > 0 {
            let last_move = self.moves.remove(moves - 1);
            let last_move_count = self.move_counts.remove(moves - 1);
            return self.move_reverse(last_move.to, last_move.from, last_move_count);
        }
        false
    }

    pub fn save_to_file(&self, file: &mut File) {
        file.write_all(&[self.loaded.len() as u8]).unwrap();
        for glass in &self.loaded {
            file.write_all(glass.as_ref()).unwrap();
        }
        let empty = vec![0; self.glass_height];
        for _ in 0..MAX_GLASSES_PER_LEVEL - self.loaded.len() {
            file.write_all(empty.as_ref()).unwrap();
        }
    }

    pub fn create_levels_filename(glass_height: usize) -> String {
        format!("levels{}.bin", glass_height)
    }
}
