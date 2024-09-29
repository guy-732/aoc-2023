use core::fmt;
use itertools::Itertools;
use rayon::prelude::*;
use std::{error::Error, fs, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SpringState {
    Operational,
    Broken,
    Unknown,
}

impl TryFrom<char> for SpringState {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Broken),
            '?' => Ok(Self::Unknown),
            other => Err(format!(
                "Spring state was not any of '.', '#' or '?' ({:?})",
                other
            )),
        }
    }
}

impl fmt::Display for SpringState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Operational => '.',
                Self::Broken => '#',
                Self::Unknown => '?',
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpringLine {
    states: Box<[SpringState]>,
    damaged_groups: Box<[usize]>,
}

impl SpringLine {
    pub(crate) fn _count_arrangements_part_2(&self) -> u64 {
        let mut states = self.states.iter().cloned().collect_vec();
        for _ in 1..5 {
            states.push(SpringState::Unknown);
            states.extend(self.states.iter());
        }

        let copy = Self {
            damaged_groups: self
                .damaged_groups
                .iter()
                .cloned()
                .cycle()
                .take(self.damaged_groups.len() * 5)
                .collect(),
            states: states.into_boxed_slice(),
        };

        // println!("{} => {}", self, &copy);

        let res = copy.count_arrangements();
        println!("{} => {}", self, res);
        res
        // 0
    }

    pub(crate) fn count_arrangements(&self) -> u64 {
        let res = self.count_arrangements_recursive(0, 0);
        // let res = count_arrangements_impl_drag_adapted(self, 0);
        // println!("{} => {}", self, res);
        res
    }

    fn count_arrangements_recursive(&self, state_pos: usize, group_pos: usize) -> u64 {
        let Some(states) = self.states.get(state_pos..) else {
            return if self.damaged_groups.get(group_pos).is_none() {
                1
            } else {
                0
            };
        };

        let Some(&group) = self.damaged_groups.get(group_pos) else {
            return if states
                .iter()
                .any(|state| matches!(state, SpringState::Broken))
            {
                0
            } else {
                1
            };
        };

        let Some(first_possibly_broken) = states
            .iter()
            .position(|state| !matches!(state, SpringState::Operational))
        else {
            return 0;
        };

        let states = &states[first_possibly_broken..];
        if states.len() < group {
            return 0; // can't possibly have enough slots to fit that group
        }

        debug_assert!(
            matches!(states[0], SpringState::Unknown | SpringState::Broken),
            "states[0] wasn't a '?' nor a '#' {:?}",
            states[0]
        );

        let result = if states
            .iter()
            .take(group)
            .any(|state| matches!(state, SpringState::Operational))
        {
            // if any of the spring in the group are operational, it's wrong
            0
        } else if states
            // next one cannot be broken
            .get(group)
            .map_or(false, |state| matches!(state, SpringState::Broken))
        {
            0
        } else {
            self.count_arrangements_recursive(
                state_pos + first_possibly_broken + group + 1,
                group_pos + 1,
            )
        };

        if matches!(states[0], SpringState::Broken) {
            return result;
        }

        debug_assert!(
            matches!(states[0], SpringState::Unknown),
            "states[0] wasn't a '?' {:?}",
            states[0]
        );

        result + self.count_arrangements_recursive(state_pos + first_possibly_broken + 1, group_pos)
    }

    fn _has_unknown(&self, from: usize) -> Option<usize> {
        for i in from..self.states.len() {
            if matches!(self.states[i], SpringState::Unknown) {
                return Some(i);
            }
        }

        None
    }

    fn _replace_first_unknown_with(mut self, state: SpringState) -> Self {
        for item in self.states.iter_mut() {
            if matches!(item, SpringState::Unknown) {
                *item = state;
                break;
            }
        }

        self
    }

    fn _check_data_matching(&self) -> bool {
        let mut current_group = 0;
        let mut damaged_streak = 0;
        let mut was_last_broken = false;
        for item in self.states.iter() {
            match item {
                SpringState::Broken => {
                    was_last_broken = true;
                    if let Some(&group) = self.damaged_groups.get(current_group) {
                        damaged_streak += 1;
                        if group < damaged_streak {
                            return false; // we went over
                        }
                    } else {
                        return false; // Broken when we already should have crossed all of them
                    }
                }
                _ => {
                    was_last_broken = false;
                    if damaged_streak != 0 {
                        if let Some(&group) = self.damaged_groups.get(current_group) {
                            if damaged_streak != group {
                                return false; // if it's lower then it's wrong
                            }
                        } else {
                            return false; // should not get here, but just in case
                        }

                        // next group
                        damaged_streak = 0;
                        current_group += 1;
                    }
                }
            }
        }

        if was_last_broken {
            if let Some(&group) = self.damaged_groups.get(current_group) {
                (current_group + 1) == self.damaged_groups.len() && group == damaged_streak
            } else {
                false
            }
        } else {
            self.damaged_groups.len() == current_group
        }
    }
}

impl FromStr for SpringLine {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (states, damaged_groups) = s
            .trim()
            .split_once(' ')
            .ok_or("Could not split at ' ' once")?;
        Ok(Self {
            states: states.chars().map(SpringState::try_from).try_collect()?,
            damaged_groups: damaged_groups
                .split(',')
                .map(usize::from_str)
                .try_collect()?,
        })
    }
}

impl fmt::Display for SpringLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for state in self.states.iter() {
            write!(f, "{}", state)?;
        }

        write!(f, " {}", self.damaged_groups.iter().join(","))
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
    let springs: Box<[SpringLine]> = input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.parse::<SpringLine>())
            }
        })
        .try_collect()?;

    Ok(springs
        .par_iter()
        .map(|spring| spring.count_arrangements())
        .sum())
}

// fn print_debug(line: &SpringLine, arrangements: &Vec<(usize, Vec<SpringState>)>) {
//     println!("{line} => [");
//     for (offset, arr) in arrangements.iter() {
//         print!(
//             "{}",
//             std::iter::repeat(' ').take(*offset).collect::<String>()
//         );
//         for state in arr.iter() {
//             print!("{}", state);
//         }

//         println!(",");
//     }

//     print!("]");
// }

fn _count_arrangements_impl_drag_adapted(row: &SpringLine, start_pos: usize) -> u64 {
    if let Some(first_unknown) = row._has_unknown(start_pos) {
        let combos = _count_arrangements_impl_drag_adapted(
            &row.clone()
                ._replace_first_unknown_with(SpringState::Operational),
            first_unknown,
        );
        _count_arrangements_impl_drag_adapted(
            &row.clone()._replace_first_unknown_with(SpringState::Broken),
            first_unknown,
        ) + combos
    } else {
        if row._check_data_matching() {
            // println!("{row}");
            1
        } else {
            0
        }
    }
}
