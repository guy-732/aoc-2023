use core::fmt;
use itertools::Itertools;
use std::{collections::VecDeque, error::Error, fs, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    #[inline]
    pub(crate) fn translate_coordinates(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<(usize, usize)> {
        use Direction::*;
        Some(match self {
            North => (row_index.checked_sub(1)?, col_index),
            South => (row_index + 1, col_index),
            East => (row_index, col_index + 1),
            West => (row_index, col_index.checked_sub(1)?),
        })
    }

    #[inline]
    pub(crate) const fn opposite(&self) -> Self {
        use Direction::*;
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SplitterVariant {
    Vertical,
    Horizontal,
}

impl SplitterVariant {
    #[inline]
    pub(crate) const fn need_to_split(
        &self,
        beam_from: Direction,
    ) -> Option<(Direction, Direction)> {
        match self {
            Self::Vertical => {
                if matches!(beam_from, Direction::East | Direction::West) {
                    Some((Direction::North, Direction::South))
                } else {
                    None
                }
            }
            Self::Horizontal => {
                if matches!(beam_from, Direction::North | Direction::South) {
                    Some((Direction::East, Direction::West))
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for SplitterVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Horizontal => '-',
                Self::Vertical => '|',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MirrorVariant {
    ForwardSlash,
    Backslash,
}

impl MirrorVariant {
    #[inline]
    pub(crate) const fn reflect(&self, beam_from: Direction) -> Direction {
        use MirrorVariant::*;
        match beam_from {
            Direction::North => match self {
                ForwardSlash => Direction::West,
                Backslash => Direction::East,
            },
            Direction::South => match self {
                ForwardSlash => Direction::East,
                Backslash => Direction::West,
            },
            Direction::East => match self {
                ForwardSlash => Direction::South,
                Backslash => Direction::North,
            },
            Direction::West => match self {
                ForwardSlash => Direction::North,
                Backslash => Direction::South,
            },
        }
    }
}

impl fmt::Display for MirrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ForwardSlash => '/',
                Self::Backslash => '\\',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    Splitter(SplitterVariant),
    Mirror(MirrorVariant),
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Mirror(mirror) => write!(f, "{}", mirror),
            Self::Splitter(splitter) => write!(f, "{}", splitter),
        }
    }
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        use MirrorVariant::*;
        use SplitterVariant::*;
        match value {
            '.' => Self::Empty,
            '-' => Self::Splitter(Horizontal),
            '|' => Self::Splitter(Vertical),
            '/' => Self::Mirror(ForwardSlash),
            '\\' => Self::Mirror(Backslash),
            other => panic!("Unrecognized tile char {:?}", other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EnergizedTile {
    tile: Tile,
    from_north: bool,
    from_south: bool,
    from_east: bool,
    from_west: bool,
}

impl EnergizedTile {
    #[inline]
    pub(crate) fn reset(&mut self) {
        *self = self.tile().into()
    }

    #[inline]
    pub(crate) fn is_energized(&self) -> bool {
        self.from_north || self.from_south || self.from_east || self.from_west
    }

    #[inline]
    pub(crate) fn tile(&self) -> Tile {
        self.tile
    }

    /// return value indicates wether the value changed (true) or was already set (false)
    #[inline]
    pub(crate) fn mark_as_energized(&mut self, beam_from: Direction) -> bool {
        match beam_from {
            Direction::North => {
                if self.from_north {
                    false
                } else {
                    self.from_north = true;
                    true
                }
            }
            Direction::South => {
                if self.from_south {
                    false
                } else {
                    self.from_south = true;
                    true
                }
            }
            Direction::East => {
                if self.from_east {
                    false
                } else {
                    self.from_east = true;
                    true
                }
            }
            Direction::West => {
                if self.from_west {
                    false
                } else {
                    self.from_west = true;
                    true
                }
            }
        }
    }
}

impl From<Tile> for EnergizedTile {
    fn from(tile: Tile) -> Self {
        Self {
            tile,
            from_north: false,
            from_south: false,
            from_east: false,
            from_west: false,
        }
    }
}

impl From<char> for EnergizedTile {
    fn from(value: char) -> Self {
        Tile::from(value).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Grid {
    array: Box<[Box<[EnergizedTile]>]>,
}

impl Grid {
    pub(crate) fn print_tiles(&self) {
        println!("{}", self);
    }

    pub(crate) fn print_energized(&self) {
        for row in self.array.iter() {
            for tile in row.iter() {
                print!("{}", if tile.is_energized() { '#' } else { '.' });
            }

            println!();
        }
    }

    pub(crate) fn reset(&mut self) {
        self.array
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|tile| tile.reset()));
    }

    pub(crate) fn count_energized(&self) -> u64 {
        self.array
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&tile| tile.is_energized())
            .count() as u64
    }

    pub(crate) fn energize(&mut self, initial: (usize, usize, Direction)) {
        let mut directions = vec![initial];
        while let Some((row_index, col_index, beam_from)) = directions.pop() {
            let tile = &mut self.array[row_index][col_index];
            if !tile.mark_as_energized(beam_from) {
                continue;
            }

            match tile.tile() {
                Tile::Empty => {
                    if let Some((next_row, next_col)) = beam_from
                        .opposite()
                        .translate_coordinates(row_index, col_index)
                    {
                        if next_row < self.array.len() && next_col < self.array[0].len() {
                            directions.push((next_row, next_col, beam_from));
                        }
                    }
                }
                Tile::Mirror(variant) => {
                    let new_direction = variant.reflect(beam_from);
                    if let Some((next_row, next_col)) =
                        new_direction.translate_coordinates(row_index, col_index)
                    {
                        if next_row < self.array.len() && next_col < self.array[0].len() {
                            directions.push((next_row, next_col, new_direction.opposite()));
                        }
                    }
                }
                Tile::Splitter(variant) => {
                    if let Some((direct1, direct2)) = variant.need_to_split(beam_from) {
                        if let Some((next_row, next_col)) =
                            direct1.translate_coordinates(row_index, col_index)
                        {
                            if next_row < self.array.len() && next_col < self.array[0].len() {
                                directions.push((next_row, next_col, direct1.opposite()));
                            }
                        }

                        if let Some((next_row, next_col)) =
                            direct2.translate_coordinates(row_index, col_index)
                        {
                            if next_row < self.array.len() && next_col < self.array[0].len() {
                                directions.push((next_row, next_col, direct2.opposite()));
                            }
                        }
                    } else {
                        // just like an empty tile
                        if let Some((next_row, next_col)) = beam_from
                            .opposite()
                            .translate_coordinates(row_index, col_index)
                        {
                            if next_row < self.array.len() && next_col < self.array[0].len() {
                                directions.push((next_row, next_col, beam_from));
                            }
                        }
                    }
                }
            }
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.array.iter() {
            for tile in row.iter() {
                write!(f, "{}", tile.tile())?;
            }

            writeln!(f)?;
        }

        Ok(())
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
    let mut grid: Grid = input.lines().collect();
    // grid.print_tiles();

    let start = Instant::now();

    grid.energize((0, 0, Direction::West));
    let mut answer = grid.count_energized();

    println!("Time to process part 1: {:?}", start.elapsed());
    println!("Part 1 answer: {}", answer);

    // grid.print_energized();

    for i in 1..grid.array.len() {
        grid.reset();
        grid.energize((i, 0, Direction::West));
        answer = answer.max(grid.count_energized());
    }

    for i in 0..grid.array.len() {
        grid.reset();
        grid.energize((i, grid.array.len() - 1, Direction::East));
        answer = answer.max(grid.count_energized());
    }

    for i in 0..grid.array[0].len() {
        grid.reset();
        grid.energize((0, i, Direction::North));
        answer = answer.max(grid.count_energized());
    }

    for i in 0..grid.array[0].len() {
        grid.reset();
        grid.energize((grid.array[0].len() - 1, i, Direction::South));
        answer = answer.max(grid.count_energized());
    }

    println!("Time to process part 2: {:?}", start.elapsed());
    Ok(answer)
}
