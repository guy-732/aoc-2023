use fnv::FnvHashSet;
use itertools::Itertools;
use std::{borrow::Borrow, collections::VecDeque, error::Error, fs, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    GardenPlot(bool),
    Rock,
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        match value {
            'S' => Self::GardenPlot(true),
            '.' => Self::GardenPlot(false),
            '#' => Self::Rock,
            other => panic!("{:?} was not any of ['S', '.', '#']", other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NeighbourIterator {
    current_pos: (usize, usize),
    done_north: bool,
    done_south: bool,
    done_east: bool,
    done_west: bool,
}

impl NeighbourIterator {
    #[inline]
    pub(crate) const fn new(current_pos: (usize, usize)) -> Self {
        Self {
            current_pos,
            done_north: false,
            done_south: false,
            done_east: false,
            done_west: false,
        }
    }

    #[inline]
    fn do_north(&mut self) -> Option<(usize, usize)> {
        if self.done_north {
            None
        } else {
            self.done_north = true;
            Some((self.current_pos.0.checked_sub(1)?, self.current_pos.1))
        }
    }

    #[inline]
    fn do_south(&mut self) -> Option<(usize, usize)> {
        if self.done_south {
            None
        } else {
            self.done_south = true;
            Some((self.current_pos.0.checked_add(1)?, self.current_pos.1))
        }
    }

    #[inline]
    fn do_east(&mut self) -> Option<(usize, usize)> {
        if self.done_east {
            None
        } else {
            self.done_east = true;
            Some((self.current_pos.0, self.current_pos.1.checked_add(1)?))
        }
    }

    #[inline]
    fn do_west(&mut self) -> Option<(usize, usize)> {
        if self.done_west {
            None
        } else {
            self.done_west = true;
            Some((self.current_pos.0, self.current_pos.1.checked_sub(1)?))
        }
    }
}

impl Iterator for NeighbourIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.do_north()
            .or_else(|| self.do_south())
            .or_else(|| self.do_east())
            .or_else(|| self.do_west())
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 2 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    const PART2_STEPS_REQUIRED: u32 = 26501365;

    let input = fs::read_to_string(input)?;
    let grid = input
        .lines()
        .map(|line| line.trim().chars().map_into::<Tile>().collect_vec())
        .collect_vec();

    let start = Instant::now();

    let part1_answ = solve_steps_part1(&grid, 64);

    let part1_time = start.elapsed();

    let start = Instant::now();

    let part2_answ = solve_part2(&grid, PART2_STEPS_REQUIRED as usize);

    let part2_time = start.elapsed();

    println!("Time to part 1: {:?}", part1_time);
    println!("Time to part 2: {:?}", part2_time);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}

#[inline]
fn find_start_pos<R>(grid: &[R]) -> (usize, usize)
where
    R: Borrow<[Tile]>,
{
    for (row_index, row) in grid.iter().enumerate() {
        for (column_index, tile) in row.borrow().iter().enumerate() {
            if matches!(tile, Tile::GardenPlot(true)) {
                return (row_index, column_index);
            }
        }
    }

    panic!("Could not find 'S' in the grid");
}

#[inline]
fn solve_steps_part1(grid: &[Vec<Tile>], steps: u32) -> u64 {
    let start_pos = find_start_pos(grid);
    if steps == 0 {
        return 1;
    }

    let mut non_valid_positions = FnvHashSet::default();
    let mut valid_positions = FnvHashSet::default();
    if steps % 2 == 0 {
        valid_positions.insert(start_pos);
    } else {
        non_valid_positions.insert(start_pos);
    }

    let mut queue = VecDeque::new();
    queue.push_back((start_pos, 0));
    while let Some((position, step)) = queue.pop_front() {
        if step >= steps {
            continue;
        }

        let new_step = step + 1;
        for new_pos in NeighbourIterator::new(position) {
            if let Some(Tile::GardenPlot(_)) =
                grid.get(new_pos.0).and_then(|row| row.get(new_pos.1))
            {
                if new_step % 2 == steps % 2 {
                    if valid_positions.insert(new_pos) {
                        queue.push_back((new_pos, new_step));
                    }
                } else {
                    if non_valid_positions.insert(new_pos) {
                        queue.push_back((new_pos, new_step));
                    }
                }
            }
        }
    }

    valid_positions.len() as u64
}

#[inline]
fn count_positions(map: &[Vec<Tile>], start: (usize, usize), steps: usize) -> usize {
    let mut positions = FnvHashSet::default();
    positions.insert(start);

    for _ in 0..steps {
        let mut new_positions = FnvHashSet::default();
        for position in positions {
            let (y, x) = position;
            if y > 0 && map[y - 1][x] != Tile::Rock {
                new_positions.insert((y - 1, x));
            }
            if y < map.len() - 1 && map[y + 1][x] != Tile::Rock {
                new_positions.insert((y + 1, x));
            }
            if x > 0 && map[y][x - 1] != Tile::Rock {
                new_positions.insert((y, x - 1));
            }
            if x < map[y].len() - 1 && map[y][x + 1] != Tile::Rock {
                new_positions.insert((y, x + 1));
            }
        }
        positions = new_positions;
    }
    positions.len()
}

#[inline]
fn solve_part2(map: &[Vec<Tile>], steps: usize) -> u64 {
    let starting_point = find_start_pos(map);

    let map_size = map.len();
    let grid_size = steps / map_size - 1;

    let even_maps_in_grid = ((grid_size + 1) / 2 * 2).pow(2);
    let odd_maps_in_grid = (grid_size / 2 * 2 + 1).pow(2);

    let odd_points_in_map = count_positions(&map, starting_point, map_size * 2 + 1);
    let even_points_in_map = count_positions(&map, starting_point, map_size * 2);

    let total_points_fully_in_grid =
        odd_points_in_map * odd_maps_in_grid + even_points_in_map * even_maps_in_grid;

    let corner_top = count_positions(&map, (map_size - 1, starting_point.1), map_size - 1);
    let corner_right = count_positions(&map, (starting_point.0, 0), map_size - 1);
    let corner_bottom = count_positions(&map, (0, starting_point.1), map_size - 1);
    let corner_left = count_positions(&map, (starting_point.0, map_size - 1), map_size - 1);

    let total_points_in_grid_corners = corner_top + corner_right + corner_bottom + corner_left;

    let small_diag_top_right = count_positions(&map, (map_size - 1, 0), map_size / 2 - 1);
    let small_diag_bottom_right = count_positions(&map, (0, 0), map_size / 2 - 1);
    let small_diag_bottom_left = count_positions(&map, (0, map_size - 1), map_size / 2 - 1);
    let small_diag_top_left = count_positions(&map, (map_size - 1, map_size - 1), map_size / 2 - 1);

    let total_points_in_small_diags = (grid_size + 1)
        * (small_diag_top_right
            + small_diag_bottom_right
            + small_diag_bottom_left
            + small_diag_top_left);

    let big_diag_top_right = count_positions(&map, (map_size - 1, 0), map_size * 3 / 2 - 1);
    let big_diag_bottom_right = count_positions(&map, (0, 0), map_size * 3 / 2 - 1);
    let big_diag_bottom_left = count_positions(&map, (0, map_size - 1), map_size * 3 / 2 - 1);
    let big_diag_top_left =
        count_positions(&map, (map_size - 1, map_size - 1), map_size * 3 / 2 - 1);

    let total_points_in_big_diags = grid_size
        * (big_diag_top_right + big_diag_bottom_right + big_diag_bottom_left + big_diag_top_left);

    let total_points_in_diag = total_points_in_small_diags + total_points_in_big_diags;

    (total_points_fully_in_grid + total_points_in_grid_corners + total_points_in_diag) as u64
}
