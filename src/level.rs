use bevy::prelude::*;
use rand::Rng;
use std::cmp;
use std::collections::HashSet;

type Glass = [u8; 4];

trait GlassExt {
    fn info(&self) -> (u8, usize, usize);
    fn is_empty(&self) -> bool;
    fn is_one_color(&self) -> bool;
}

static MAX_DEPTH: usize = 12;

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
        let name = self.current[x][y];
        match name {
            b'r' => Some(hex_to_color(0xff0000)), // red
            b'b' => Some(hex_to_color(0x4040ff)), // blue
            b'o' => Some(hex_to_color(0xED872D)), // orange
            b'p' => Some(hex_to_color(0xF19CBB)), // pink
            b'g' => Some(hex_to_color(0x54626F)), // gray
            b'd' => Some(hex_to_color(0x343434)), // dark gray
            b'a' => Some(hex_to_color(0x8DB600)), // apple green
            b'l' => Some(hex_to_color(0xBFFF00)), // light green
            b's' => Some(hex_to_color(0x2e8b57)), // sea green
            b'c' => Some(hex_to_color(0x00FFFF)), // cyan
            b'y' => Some(hex_to_color(0xFFFF00)), // yellow
            b'v' => Some(hex_to_color(0x9400D3)), // violet
            _ => None,
        }
    }

    pub fn load_next(&mut self) {
        self.number = (self.number + 1) % 3;
        let level = Level::load(self.number);
        self.current = level.current.clone();
        self.loaded = level.loaded.clone();
    }

    pub fn load(number: usize) -> Self {
        let level0 = r#"
        g=bb
        g=bb
            "#;
        let level1 = r#"
        g=orbr
        g=rapo
        g=gcgr
        g=allc
        g=paov
        g=vvcb
        g=oppl
        g=abvl
        g=gbcg
        g=
        g=
            "#;
        let level2 = r#"
        g=bpvr
        g=ccbv
        g=dcla
        g=ordc
        g=podl
        g=lysy
        g=grby
        g=bogv
        g=ysag
        g=rppg
        g=svld
        g=aaso
        g=
        g=
            "#;

        let levels = vec![level0, level1, level2];
        let level = levels[number];
        let lines = level.split("\n");
        let mut level = Level {
            loaded: Vec::new(),
            current: Vec::new(),
            number: number,
        };
        for line in lines {
            //let line = line.unwrap();
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

    /// get last color and fill level of glass
    fn get_color_and_level(&self, glass_index: usize) -> (u8, usize) {
        let mut i = 0;
        let mut color = 0;
        while i < 4 {
            if self.current[glass_index][i] > 0 {
                color = self.current[glass_index][i];
            } else {
                break;
            }
            i += 1;
        }
        (color, i)
    }

    /// count number of same water color, starting at the given index and iterating down
    fn count_water_color(&self, glass_index: usize, color: u8, start_index: usize) -> usize {
        let mut count = 0;
        let mut i = start_index;
        loop {
            if self.current[glass_index][i] == color {
                count += 1;
            } else {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        count
    }

    pub fn move_water(&mut self, from: usize, to: usize) -> bool {
        // test if there is something to move
        if self.current[from][0] == 0 {
            return false;
        }

        // get last color from where to move
        let (from_color, mut i) = self.get_color_and_level(from);
        i -= 1;
        let mut from_top = i;

        // count how many to move
        let mut count = self.count_water_color(from, from_color, from_top);

        // get last color of destination glass
        let (to_color, mut i) = self.get_color_and_level(to);

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

    pub fn move_reverse(&mut self, from: usize, to: usize, count: usize) -> bool {
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

    fn is_mixed(&mut self) -> bool {
        for i in 0..self.current.len() - 2 {
            if self.current[i].is_one_color() {
                return false;
            }
        }
        let last = self.current.len() - 1;
        self.current[last].is_empty() && self.current[last - 1].is_empty()
    }

    fn mix_impl(
        &mut self,
        tested: &mut HashSet<Glasses>,
        depth: usize,
        solutions: &mut Vec<Glasses>,
    ) {
        fn glass_to_u32(glass: &[u8]) -> u32 {
            (glass[0] as u32)
                | ((glass[1] as u32) << 8)
                | ((glass[2] as u32) << 16)
                | ((glass[3] as u32) << 24)
        }

        // check all possible moves
        let last = self.current.clone();
        for from in 0..self.current.len() {
            for to in 0..self.current.len() {
                if from != to {
                    for count in 1..=3 {
                        if self.move_reverse(from, to, count) {
                            // test if winning solution
                            if self.is_mixed() {
                                solutions.push(self.current.clone());
                            } else {
                                if depth < MAX_DEPTH {
                                    // sort copy of glasses
                                    let mut copy = self.current.clone();
                                    copy.sort_by_key(|a| glass_to_u32(a));

                                    // if not already tested, then test it recursively
                                    if !tested.contains(&copy) {
                                        tested.insert(copy);
                                        self.mix_impl(tested, depth + 1, solutions);
                                    }
                                }
                            }

                            // restore last position
                            self.current = last.clone();
                        }
                    }
                }
            }
        }
    }

    // mixes the water for a new level
    pub fn mix(&mut self) {
        // create target
        //let colors = "orbaplcvg";
        let colors = "orba";
        let mut glass_empty: Glass = Default::default();
        for i in 0..4 {
            glass_empty[i] = 0;
        }
        self.loaded = Vec::new();
        for c in 0..colors.len() {
            let mut glass: Glass = Default::default();
            for i in 0..4 {
                glass[i] = colors.as_bytes()[c] as u8;
            }
            self.loaded.push(glass);
        }
        self.loaded.push(glass_empty);
        self.loaded.push(glass_empty);

        // recursively move backwards from target for the current level
        let mut solutions: Vec<Glasses> = Vec::new();
        let mut tested: HashSet<Glasses> = HashSet::new();
        self.current = self.loaded.clone();
        self.mix_impl(&mut tested, 0, &mut solutions);

        // set current level as new loaded level
        println!("{} configurations tested", tested.len());
        if solutions.len() > 0 {
            println!("{} solutions found", solutions.len());
            let mut max: usize = 0;
            let mut max_solution = solutions[0].clone();
            for s in solutions {
                self.current = s.clone();
                let (keys, _size) = self.solve();
                if keys.len() > max {
                    max_solution = s;
                    max = keys.len();
                }
            }
            self.loaded = max_solution;
            println!("solution length: {}", max);
        } else {
            println!("no solutions found");
        }
        self.restart();
    }

    /*
    pub fn mix_once(&mut self) {
        let mut rng = rand::thread_rng();

        // search a random glass with water in it
        let mut from_pos: Vec<usize> = Vec::new();
        for (i, glass) in self.current.iter().enumerate() {
            if glass[0] > 0 {
                from_pos.push(i);
            }
        }
        let from = from_pos[rng.gen_range(0..from_pos.len())];

        // get last color from where to move
        let (from_color, mut i) = self.get_color_and_level(from);
        i -= 1;
        let mut from_top = i;

        // count how many are possible to move max
        let count = self.count_water_color(from, from_color, from_top);

        // search possible target: empty glass or different color, and not "from", and glass not full
        let mut to_pos: Vec<usize> = Vec::new();
        for (i, glass) in self.current.iter().enumerate() {
            let (to_color, to_level) = self.get_color_and_level(i);
            if (glass[0] == 0 || from_color != to_color) && i != from && to_level < 4 {
                to_pos.push(i);
            }
        }

        // choose a target glass
        let to = to_pos[rng.gen_range(0..to_pos.len())];

        // calculate how much water can be moved max
        let (_to_color, mut to_level) = self.get_color_and_level(to);
        let to_free = 4 - to_level;
        let max_move = cmp::min(to_free, count);

        // calcuate random value how much water to move
        let move_count = rng.gen_range(1..=max_move);

        // move water
        for _i in 0..move_count {
            self.current[to][to_level] = self.current[from][from_top];
            to_level += 1;
            self.current[from][from_top] = 0;
            from_top = from_top.saturating_sub(1);
        }
    }
    */
}
