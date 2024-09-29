use core::fmt;
use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use std::{collections::VecDeque, error::Error, fs, time::Instant, io::{Write, self}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// self is the slope's direction
    #[inline]
    fn can_go_on_slope_from(&self, from: Self) -> bool {
        match self {
            Self::North => matches!(from, Self::South),
            Self::South => matches!(from, Self::North),
            Self::East => matches!(from, Self::West),
            Self::West => matches!(from, Self::East),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Path,
    Forest,
    Slope(Direction),
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Forest => write!(f, "#"),
            Self::Path => write!(f, "."),
            Self::Slope(direction) => match direction {
                Direction::North => write!(f, "^"),
                Direction::South => write!(f, "v"),
                Direction::East => write!(f, ">"),
                Direction::West => write!(f, "<"),
            },
        }
    }
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        match value {
            '.' => Self::Path,
            '#' => Self::Forest,
            '^' => Self::Slope(Direction::North),
            '>' => Self::Slope(Direction::East),
            'v' => Self::Slope(Direction::South),
            '<' => Self::Slope(Direction::West),
            other => panic!("Unrecognized tile char {:?}", other),
        }
    }
}

type Position = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Graph {
    adj_list: FnvHashMap<Position, FnvHashMap<Position, u64>>,
}

impl Graph {
    #[allow(dead_code)]
    #[inline]
    fn write_as_gv<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "digraph {{")?;
        writeln!(writer, "    layout=\"dot\"\n")?;

        for key in self.adj_list.keys() {
            writeln!(writer, "    v{}_{} [label=\"{:?}\"]", key.0, key.1, key)?;
        }

        writeln!(writer)?;

        for (key, value) in self.adj_list.iter() {
            for (dest, distance) in value.iter() {
                writeln!(writer, "    v{}_{} -> v{}_{} [label=\"{}\"]", key.0, key.1, dest.0, dest.1, distance)?;
            }
        }

        writeln!(writer, "}}")
    }

    #[inline]
    fn new_from_grid_slopes(tile_grid: &[Vec<Tile>], start: Position, end: Position) -> Self {
        let mut graph = Self {
            adj_list: FnvHashMap::default(),
        };
        let mut queue = VecDeque::from([(start, start, Direction::North)]);

        'outer: while let Some((intersect, original_pos, original_from)) = queue.pop_front() {
            let mut pos = original_pos;
            let mut from = original_from;
            let mut has_slope = false;
            let mut distance = 0;

            let neighbours = loop {
                if pos == end {
                    break vec![];
                }

                if let Tile::Slope(slope_dir) = tile_grid[pos.0][pos.1] {
                    if !slope_dir.can_go_on_slope_from(from) {
                        continue 'outer;
                    }

                    has_slope = true;
                }

                let neighbours = NeighbourIterator::new(pos, from)
                    .filter(|(pos, _)| {
                        tile_grid
                            .get(pos.0)
                            .and_then(|row| row.get(pos.1))
                            .is_some_and(|tile| !matches!(tile, Tile::Forest))
                    })
                    .collect_vec();

                if neighbours.is_empty() {
                    eprintln!(
                        "Tile at {:?} ('{}') does not have any neighbours apart from the one from {:?}",
                        pos,
                        tile_grid[pos.0][pos.1],
                        from,
                    );
                    continue 'outer; // ... welp
                }

                if neighbours.len() > 1 {
                    break neighbours;
                }

                let (new_pos, new_from) = neighbours[0];
                distance += 1;
                pos = new_pos;
                from = new_from;
            };

            if intersect != original_pos {
                distance += 1;
            }

            if !graph.adj_list.contains_key(&intersect) {
                graph.adj_list.insert(intersect, FnvHashMap::default());
            }

            let adj = graph.adj_list.get_mut(&intersect).unwrap();
            // pos is an intersection
            if !adj.contains_key(&pos) {
                adj.insert(pos, distance);
                let mut pos_map = FnvHashMap::default();
                if !has_slope {
                    pos_map.insert(intersect, distance);
                }

                graph.adj_list.insert(pos, pos_map);

                for (neighbour, neighbour_from) in neighbours {
                    if let Tile::Slope(slope_dir) = tile_grid[neighbour.0][neighbour.1] {
                        if !slope_dir.can_go_on_slope_from(neighbour_from) {
                            continue;
                        }
                    }

                    queue.push_back((pos, neighbour, neighbour_from));
                }
            } else {
                // eprintln!(
                //     "{:?} to {:?} already exists (stored distance: {}, found distance: {})",
                //     pos,
                //     intersect,
                //     adj.get(&pos).unwrap(),
                //     distance
                // );
            }
        }

        graph
    }

    #[inline]
    fn new_from_grid_ignore_slopes(tile_grid: &[Vec<Tile>], start: Position, end: Position) -> Self {
        let mut graph = Self {
            adj_list: FnvHashMap::default(),
        };
        let mut queue = VecDeque::from([(start, start, Direction::North)]);

        'outer: while let Some((intersect, original_pos, original_from)) = queue.pop_front() {
            let mut pos = original_pos;
            let mut from = original_from;
            let mut distance = 0;

            let neighbours = loop {
                if pos == end {
                    break vec![];
                }

                let neighbours = NeighbourIterator::new(pos, from)
                    .filter(|(pos, _)| {
                        tile_grid
                            .get(pos.0)
                            .and_then(|row| row.get(pos.1))
                            .is_some_and(|tile| !matches!(tile, Tile::Forest))
                    })
                    .collect_vec();

                if neighbours.is_empty() {
                    eprintln!(
                        "Tile at {:?} ('{}') does not have any neighbours apart from the one from {:?}",
                        pos,
                        tile_grid[pos.0][pos.1],
                        from,
                    );
                    continue 'outer; // ... welp
                }

                if neighbours.len() > 1 {
                    break neighbours;
                }

                let (new_pos, new_from) = neighbours[0];
                distance += 1;
                pos = new_pos;
                from = new_from;
            };

            if intersect != original_pos {
                distance += 1;
            }

            if !graph.adj_list.contains_key(&intersect) {
                graph.adj_list.insert(intersect, FnvHashMap::default());
            }

            let adj = graph.adj_list.get_mut(&intersect).unwrap();
            // pos is an intersection
            if !adj.contains_key(&pos) {
                adj.insert(pos, distance);

                if !graph.adj_list.contains_key(&pos) {
                    let mut pos_map = FnvHashMap::default();
                    pos_map.insert(intersect, distance);

                    graph.adj_list.insert(pos, pos_map);
                } else {
                    graph.adj_list.get_mut(&pos).unwrap().insert(intersect, distance);
                }

                for (neighbour, neighbour_from) in neighbours {
                    queue.push_back((pos, neighbour, neighbour_from));
                }
            } else {
                // eprintln!(
                //     "{:?} to {:?} already exists (stored distance: {}, found distance: {})",
                //     pos,
                //     intersect,
                //     adj.get(&pos).unwrap(),
                //     distance
                // );
            }
        }

        graph
    }

    #[inline]
    fn longest_simple_path(&self, start: Position, end: Position) -> u64 {
        self.longest_simple_path_impl(start, end, &mut FnvHashSet::default()).unwrap()
    }

    fn longest_simple_path_impl(&self, current: Position, end: Position, visited: &mut FnvHashSet<Position>) -> Option<u64> {
        if current == end {
            return Some(0);
        }

        visited.insert(current);
        self.adj_list.get(&current).unwrap().iter()
            .filter_map(|(key, distance)| {
                if visited.contains(key) {
                    // eprintln!("{:?} already visited, skipping", key);
                    None
                } else if key == &end {
                    Some(*distance)
                } else {
                    Some(
                        distance
                        + self.longest_simple_path_impl(*key, end, &mut visited.clone())?
                    )
                }
            })
            .max()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NeighbourIterator {
    position: Position,
    done_north: bool,
    done_south: bool,
    done_east: bool,
    done_west: bool,
}

impl NeighbourIterator {
    #[inline]
    fn new(position: Position, from: Direction) -> Self {
        let mut result = Self {
            position,
            done_north: false,
            done_south: false,
            done_east: false,
            done_west: false,
        };

        match from {
            Direction::North => result.done_north = true,
            Direction::South => result.done_south = true,
            Direction::East => result.done_east = true,
            Direction::West => result.done_west = true,
        }

        result
    }

    #[inline]
    fn north(&mut self) -> Option<(Position, Direction)> {
        if self.done_north {
            None
        } else {
            self.done_north = true;
            Some((
                (self.position.0.checked_sub(1)?, self.position.1),
                Direction::South,
            ))
        }
    }

    #[inline]
    fn south(&mut self) -> Option<(Position, Direction)> {
        if self.done_south {
            None
        } else {
            self.done_south = true;
            Some((
                (self.position.0.checked_add(1)?, self.position.1),
                Direction::North,
            ))
        }
    }

    #[inline]
    fn east(&mut self) -> Option<(Position, Direction)> {
        if self.done_east {
            None
        } else {
            self.done_east = true;
            Some((
                (self.position.0, self.position.1.checked_add(1)?),
                Direction::West,
            ))
        }
    }

    #[inline]
    fn west(&mut self) -> Option<(Position, Direction)> {
        if self.done_west {
            None
        } else {
            self.done_west = true;
            Some((
                (self.position.0, self.position.1.checked_sub(1)?),
                Direction::East,
            ))
        }
    }
}

impl Iterator for NeighbourIterator {
    type Item = (Position, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        self.north()
            .or_else(|| self.south())
            .or_else(|| self.east())
            .or_else(|| self.west())
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

    let grid = input
        .lines()
        .map(|line| line.chars().map_into::<Tile>().collect_vec())
        .collect_vec();

    let start_pos = (
        0,
        grid[0]
            .iter()
            .find_position(|&tile| matches!(tile, Tile::Path))
            .unwrap()
            .0,
    );
    let end_pos = (
        grid.len() - 1,
        grid[grid.len() - 1]
            .iter()
            .find_position(|&tile| matches!(tile, Tile::Path))
            .unwrap()
            .0,
    );

    let start = Instant::now();
    let graph = Graph::new_from_grid_slopes(&grid, start_pos, end_pos);
    let parse_to_graph_time = start.elapsed();

    println!("Time to parse into a graph (taking slopes into account): {:?}", parse_to_graph_time);
    // graph.write_as_gv(&mut io::stdout())?;

    let start = Instant::now();

    let part1_answ = graph.longest_simple_path(start_pos, end_pos);
    let part1_time = start.elapsed();

    drop(graph);

    let start = Instant::now();
    let graph = Graph::new_from_grid_ignore_slopes(&grid, start_pos, end_pos);
    let parse_to_graph_time = start.elapsed();
    println!("Time to parse into a graph (without taking slopes into account): {:?}", parse_to_graph_time);
    // graph.write_as_gv(&mut io::stdout())?;

    println!("Time for part 1: {:?}", part1_time);

    let start = Instant::now();
    let part2_answ = graph.longest_simple_path(start_pos, end_pos);
    let part2_time = start.elapsed();

    println!("Time for part 2: {:?}", part2_time);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}
