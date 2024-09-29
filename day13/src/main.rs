use core::fmt;
use itertools::Itertools;
use rayon::prelude::*;
use std::{error::Error, fs, iter::Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PatternCell {
    Ash,
    Rock,
}

// impl PatternCell {
// fn other(&self) -> Self {
// match self {
// Self::Ash => Self::Rock,
// Self::Rock => Self::Ash,
// }
// }
// }

impl From<char> for PatternCell {
    fn from(value: char) -> Self {
        match value {
            '.' => Self::Ash,
            '#' => Self::Rock,
            other => panic!("Character was neither '.' nor '#' ({:?})", other),
        }
    }
}

impl fmt::Display for PatternCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ash => write!(f, "."),
            Self::Rock => write!(f, "#"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pattern {
    list: Box<[Box<[PatternCell]>]>,
}

impl Pattern {
    fn is_empty(&self) -> bool {
        return self.list.len() == 0;
    }

    fn determine_mirror_pos_part_1(&self) -> MirrorPos {
        self.try_get_mirror_pos_vertical_part_1()
            .map(MirrorPos::Vertical)
            .or_else(|| {
                self.try_get_mirror_pos_horizontal_part_1()
                    .map(MirrorPos::Horizontal)
            })
            .unwrap_or_else(|| panic!("{}Could not find a place to put a mirror.", self))
    }

    fn try_get_mirror_pos_vertical_part_1(&self) -> Option<u64> {
        if self.is_empty() {
            return None;
        }

        let column_count = self.list[0].len();
        for i in 1..column_count {
            let cols_to_compare = i.min(column_count - i);
            let mut is_mirrored = true;
            'row_loop: for row in self.list.iter() {
                for comp in 0..cols_to_compare {
                    if row[i - comp - 1] != row[i + comp] {
                        is_mirrored = false;
                        break 'row_loop;
                    }
                }
            }

            if is_mirrored {
                return Some(i as u64);
            }
        }

        None
    }

    fn try_get_mirror_pos_horizontal_part_1(&self) -> Option<u64> {
        let row_count = self.list.len();
        for i in 1..row_count {
            let rows_to_compare = i.min(row_count - i);
            if (0..rows_to_compare).all(|comp| self.list[i - comp - 1] == self.list[i + comp]) {
                return Some(i as u64);
            }
        }

        None
    }

    fn determine_mirror_pos_part_2(&self) -> MirrorPos {
        self.try_get_mirror_pos_vertical_part_2()
            .map(MirrorPos::Vertical)
            .or_else(|| {
                self.try_get_mirror_pos_horizontal_part_2()
                    .map(MirrorPos::Horizontal)
            })
            .unwrap_or_else(|| panic!("{}Could not find a place to put a mirror.", self))
    }

    fn try_get_mirror_pos_vertical_part_2(&self) -> Option<u64> {
        if self.is_empty() {
            return None;
        }

        let column_count = self.list[0].len();
        for i in 1..column_count {
            let cols_to_compare = i.min(column_count - i);
            let mut is_mirrored = false;
            let mut has_one_mistake = false;
            'row_loop: for row in self.list.iter() {
                for comp in 0..cols_to_compare {
                    if row[i - comp - 1] != row[i + comp] {
                        if !has_one_mistake {
                            has_one_mistake = true;
                            is_mirrored = true
                        } else {
                            is_mirrored = false;
                            break 'row_loop;
                        }
                    }
                }
            }

            if is_mirrored {
                return Some(i as u64);
            }
        }

        None
    }

    fn try_get_mirror_pos_horizontal_part_2(&self) -> Option<u64> {
        let row_count = self.list.len();
        for i in 1..row_count {
            let rows_to_compare = i.min(row_count - i);
            let mut has_one_mistake = false;
            'comp_loop: for comp in 0..rows_to_compare {
                for (&val1, &val2) in self.list[i - comp - 1].iter().zip_eq(self.list[i + comp].iter()) {
                    if val1 != val2 {
                        if has_one_mistake {
                            has_one_mistake = false; // since we break out of the loop the condition will ignore this and skip to the next
                            break 'comp_loop;
                        } else {
                            has_one_mistake = true;
                        }
                    }
                }
            }

            if has_one_mistake {
                return Some(i as u64);
            }
        }

        None
    }
}

impl<'s> FromIterator<&'s str> for Pattern {
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        Self {
            list: iter
                .into_iter()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() {
                        None
                    } else {
                        Some(line.chars().map_into().collect())
                    }
                })
                .collect(),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.list.iter() {
            for &cell in row.iter() {
                write!(f, "{}", cell)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MirrorPos {
    /// u64 is how many columns are present BEFORE the mirror
    Vertical(u64),
    /// u64 is how many rows are present BEFORE the mirror
    Horizontal(u64),
}

impl From<MirrorPos> for u64 {
    fn from(value: MirrorPos) -> Self {
        match value {
            MirrorPos::Horizontal(rows) => rows * 100,
            MirrorPos::Vertical(cols) => cols,
        }
    }
}

impl Sum<MirrorPos> for u64 {
    fn sum<I: Iterator<Item = MirrorPos>>(iter: I) -> Self {
        iter.map_into::<u64>().sum()
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
    let mut lines = input.lines();
    let mut patterns = vec![];
    loop {
        let pattern: Pattern = lines
            .by_ref()
            .take_while(|&line| !line.trim().is_empty())
            .collect();

        if pattern.is_empty() {
            break;
        }

        patterns.push(pattern);
    }

    Ok(patterns
        .into_par_iter()
        .map(|pattern| {
            let mirror = pattern.determine_mirror_pos_part_2();
            // println!("{pattern}----> {mirror:?}\n");
            mirror
        })
        .sum())
}
