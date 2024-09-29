#![feature(map_try_insert)]

use fnv::FnvHashMap;
use itertools::Itertools;
use std::{
    cmp,
    collections::BinaryHeap,
    error::Error,
    fs,
    ops::{Index, IndexMut},
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CityBlock {
    weight: u8,
}

impl From<char> for CityBlock {
    fn from(value: char) -> Self {
        CityBlock {
            weight: value
                .to_digit(10)
                .unwrap_or_else(|| panic!("char was not a digit ({:?})", value))
                as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Grid {
    array: Box<[Box<[CityBlock]>]>,
}

impl Grid {
    pub(crate) fn dijkstra(&self, ultra: bool) -> u64 {
        let mut queue = BinaryHeap::new();
        let mut visited = FnvHashMap::default();
        let rows = self.array.len();
        let cols = self.array[0].len();

        queue.push((cmp::Reverse(0), 0, 0, 0u8, Direction::East));

        while let Some((cmp::Reverse(prio), row, col, straight_steps, direction)) = queue.pop() {
            if (row, col) == (rows - 1, cols - 1) {
                return prio;
            }

            if let Err(mut err) = visited.try_insert((row, col, direction), straight_steps) {
                if *err.entry.get() <= straight_steps {
                    continue;
                }
                err.entry.insert(straight_steps);
            }

            let can_move_straight = if ultra {
                straight_steps < 10
            } else {
                straight_steps < 3
            };

            let north_move = ((can_move_straight || direction != Direction::North)
                && direction != Direction::South
                && row > 0
                && (!ultra || direction == Direction::North || row > 4))
                .then(|| {
                    if ultra && direction != Direction::North {
                        (row - 4, col, Direction::North)
                    } else {
                        (row - 1, col, Direction::North)
                    }
                });

            let south_move = ((direction != Direction::South || can_move_straight)
                && direction != Direction::North
                && (row < rows - 1)
                && (!ultra || direction == Direction::South || row < rows - 4))
                .then(|| {
                    if ultra && direction != Direction::South {
                        (row + 4, col, Direction::South)
                    } else {
                        (row + 1, col, Direction::South)
                    }
                });

            let east_move = ((direction != Direction::East || can_move_straight)
                && direction != Direction::West
                && (col < cols - 1)
                && (!ultra || (row, col) == (0, 0) || direction == Direction::East || col < cols - 4))
                .then(|| {
                    if ultra && (direction != Direction::East || (row, col) == (0, 0)) {
                        (row, col + 4, Direction::East)
                    } else {
                        (row, col + 1, Direction::East)
                    }
                });

            let west_move = ((can_move_straight || direction != Direction::West)
                && direction != Direction::East
                && col > 0
                && (!ultra || direction == Direction::West || col > 4))
                .then(|| {
                    if ultra && direction != Direction::West {
                        (row, col - 4, Direction::West)
                    } else {
                        (row, col - 1, Direction::West)
                    }
                });

            [north_move, south_move, east_move, west_move]
                .into_iter()
                .flatten()
                .for_each(|(new_row, new_col, new_direction)| {
                    let prio = if ultra && (new_direction != direction || (row, col) == (0, 0)) {
                        match new_direction {
                            Direction::North => {
                                (0..4).map(|i| self.array[new_row + i][new_col].weight).sum::<u8>() as u64
                            }
                            Direction::West => {
                                (0..4).map(|i| self.array[new_row][new_col + i].weight).sum::<u8>() as u64
                            }
                            Direction::South => {
                                (0..4).map(|i| self.array[new_row - i][new_col].weight).sum::<u8>() as u64
                            }
                            Direction::East => {
                                (0..4).map(|i| self.array[new_row][new_col - i].weight).sum::<u8>() as u64
                            }
                        }
                    } else {
                        (self.array[new_row][new_col].weight) as u64
                    } + prio;
                    let straight_steps = match new_direction {
                        _ if ultra && (new_direction != direction || (col, row) == (0, 0)) => 4,
                        _ if new_direction != direction => 1,
                        _ => straight_steps + 1,
                    };

                    queue.push((cmp::Reverse(prio), new_row, new_col, straight_steps, new_direction));
                });
        }

        panic!("Unreachable");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Index<(usize, usize)> for Grid {
    type Output = CityBlock;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.array[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.array[index.0][index.1]
    }
}

impl<'s> FromIterator<&'s str> for Grid {
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        Self {
            array: iter
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

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 2 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;
    let grid: Grid = input.lines().collect();

    let start = Instant::now();

    let part1 = grid.dijkstra(false);
    let part1_time = start.elapsed();

    let res = grid.dijkstra(true);
    let part2_time = start.elapsed();

    println!("Time to part 1: {:?}\nTime to part 2: {:?}", part1_time, part2_time);
    println!("Part 1 answer: {}", part1);
    Ok(res)
}
