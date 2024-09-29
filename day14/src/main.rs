use core::fmt;
use itertools::Itertools;
use std::{error::Error, fs, time::Instant};

macro_rules! repeat_twice {
    ($expr:expr) => {
        $expr;
        $expr;
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PlatformCell {
    RollingRock,
    StationaryRock,
    Empty,
}

impl From<char> for PlatformCell {
    #[inline]
    fn from(value: char) -> Self {
        match value {
            '.' => Self::Empty,
            '#' => Self::StationaryRock,
            'O' => Self::RollingRock,
            other => panic!("char was not any of '.', '#' or 'O', was {:?}", other),
        }
    }
}

impl fmt::Display for PlatformCell {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::StationaryRock => write!(f, "#"),
            Self::RollingRock => write!(f, "O"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Platform {
    grid: Box<[Box<[PlatformCell]>]>,
}

impl Platform {
    #[inline]
    pub(crate) fn spin_cycle(&mut self) {
        self.slide_rolling_to_north();
        // println!("North:\n{}\n", self);
        self.slide_rolling_to_west();
        // println!("West:\n{}\n", self);
        self.slide_rolling_to_south();
        // println!("South:\n{}\n", self);
        self.slide_rolling_to_east();
        // println!("East:\n{}\n", self);
    }

    #[inline]
    fn slide_rolling_to_south(&mut self) {
        for i in (0..(self.grid.len() - 1)).rev() {
            for j in 0..self.grid[0].len() {
                if matches!(self.grid[i][j], PlatformCell::RollingRock) {
                    if let Some(k) = ((i + 1)..self.grid.len())
                        .take_while(|&k| matches!(self.grid[k][j], PlatformCell::Empty))
                        .last()
                    {
                        self.grid[k][j] = PlatformCell::RollingRock;
                        self.grid[i][j] = PlatformCell::Empty;
                    }
                }
            }
        }
    }

    #[inline]
    fn slide_rolling_to_west(&mut self) {
        for j in 1..self.grid[0].len() {
            for i in 0..self.grid.len() {
                if matches!(self.grid[i][j], PlatformCell::RollingRock) {
                    if let Some(k) = (0..j)
                        .rev()
                        .take_while(|&k| matches!(self.grid[i][k], PlatformCell::Empty))
                        .last()
                    {
                        self.grid[i][k] = PlatformCell::RollingRock;
                        self.grid[i][j] = PlatformCell::Empty;
                    }
                }
            }
        }
    }

    #[inline]
    fn slide_rolling_to_east(&mut self) {
        for j in (0..(self.grid[0].len() - 1)).rev() {
            for i in 0..self.grid.len() {
                if matches!(self.grid[i][j], PlatformCell::RollingRock) {
                    if let Some(k) = ((j + 1)..self.grid[0].len())
                        .take_while(|&k| matches!(self.grid[i][k], PlatformCell::Empty))
                        .last()
                    {
                        self.grid[i][k] = PlatformCell::RollingRock;
                        self.grid[i][j] = PlatformCell::Empty;
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn slide_rolling_to_north(&mut self) {
        for i in 1..self.grid.len() {
            for j in 0..self.grid[0].len() {
                if matches!(self.grid[i][j], PlatformCell::RollingRock) {
                    if let Some(k) = (0..i)
                        .rev()
                        .take_while(|&k| matches!(self.grid[k][j], PlatformCell::Empty))
                        .last()
                    {
                        self.grid[k][j] = PlatformCell::RollingRock;
                        self.grid[i][j] = PlatformCell::Empty;
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn load_on_north_beam(&self) -> u64 {
        self.grid
            .iter()
            .rev()
            .zip(1..)
            .map(|(row, weight)| {
                weight
                    * (row
                        .iter()
                        .filter(|&cell| matches!(cell, PlatformCell::RollingRock))
                        .count() as u64)
            })
            .sum()
    }

    #[inline]
    pub(crate) fn solve_part_2(mut self) -> u64 {
        let mut turtoise = self.clone();
        let mut hare = self.clone();
        turtoise.spin_cycle();
        repeat_twice!(hare.spin_cycle());

        // let mut iteration_count = 1;
        while turtoise != hare {
            turtoise.spin_cycle();
            repeat_twice!(hare.spin_cycle());
            // iteration_count += 1;
        }

        // println!("After {} iterations, both states are the same:\n{}", iteration_count, turtoise);

        let mut cycle_start = 0;
        while turtoise != self {
            turtoise.spin_cycle();
            self.spin_cycle();
            cycle_start += 1;
        }

        // println!("After {} iterations, start of cycle found:\n{}", cycle_start, turtoise);

        let mut cycle_length = 1;
        turtoise.spin_cycle();
        while turtoise != self {
            turtoise.spin_cycle();
            cycle_length += 1;
        }


        let remaining_spins = (PART_2_SPIN_COUNT - cycle_start) % cycle_length;
        println!("Cycle start: {}", cycle_start);
        println!("Cycle length: {}", cycle_length);
        println!("Remaining spins: {}", remaining_spins);
        for _ in 0..remaining_spins {
            turtoise.spin_cycle();
        }

        turtoise.load_on_north_beam()
    }
}

const PART_2_SPIN_COUNT: u64 = 1_000_000_000;

impl<'s> FromIterator<&'s str> for Platform {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        Self {
            grid: iter
                .into_iter()
                .map(|line| {
                    let line = line.trim();
                    line.chars().map_into().collect()
                })
                .collect(),
        }
    }
}

impl fmt::Display for Platform {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.grid.iter() {
            for cell in row.iter() {
                write!(f, "{}", cell)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;
    let platform: Platform = input
        .lines()
        .take_while(|&line| !line.trim().is_empty())
        .collect();

    // println!("{}", platform);
    // platform.slide_rolling_to_north();
    // println!("{}", platform);

    // Ok(platform.load_on_north_beam())

    let start = Instant::now();

    let res = platform.solve_part_2();
    // for _i in 0..1000 {
    //     platform.spin_cycle();
    //     // println!("After {} cycles:\n{}\n", _i, platform);
    //     // if _i % 10_000_000 == 0 {
    //     //     println!("{}: Elapsed: {:?}", _i, start.elapsed());
    //     // }
    // }

    println!("Finished after {:?}", start.elapsed());
    Ok(res)
}
