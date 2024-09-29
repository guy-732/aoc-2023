use std::{
    error::Error,
    fmt, fs,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /*
    const ALL_DIRECTIONS: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];
    */

    fn translate_coordinates(&self, row_num: usize, column_num: usize) -> Option<(usize, usize)> {
        use Direction::*;
        Some(match self {
            North => (row_num.checked_sub(1)?, column_num),
            South => (row_num.checked_add(1)?, column_num),
            East => (row_num, column_num.checked_add(1)?),
            West => (row_num, column_num.checked_sub(1)?),
        })
    }

    fn opposite(&self) -> Self {
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
enum ConnectionVariant {
    Vertical,
    Horizontal,
    CornerNE,
    CornerNW,
    CornerSW,
    CornerSE,
    StartingPoint,
    Ground,
}

impl ConnectionVariant {
    //! It is safe to do for ANY of those
    //! > variant.connected_to.unwrap()
    const CONNECTED_VARIANTS: [ConnectionVariant; 6] = [
        Self::Vertical,
        Self::Horizontal,
        Self::CornerNE,
        Self::CornerNW,
        Self::CornerSE,
        Self::CornerSW,
    ];

    fn connected_to(&self) -> Option<(Direction, Direction)> {
        use Direction::*;
        Some(match self {
            Self::Vertical => (North, South),
            Self::Horizontal => (West, East),
            Self::CornerNE => (North, East),
            Self::CornerNW => (North, West),
            Self::CornerSW => (South, West),
            Self::CornerSE => (South, East),
            Self::Ground | Self::StartingPoint => None?,
        })
    }
}

impl TryFrom<char> for ConnectionVariant {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use ConnectionVariant::*;
        Ok(match value {
            '|' => Vertical,
            '-' => Horizontal,
            'L' => CornerNE,
            'J' => CornerNW,
            '7' => CornerSW,
            'F' => CornerSE,
            '.' => Ground,
            'S' => StartingPoint,
            other => Err(format!("Unrecognized character for pipe grid: {other:?}"))?,
        })
    }
}

impl fmt::Display for ConnectionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConnectionVariant::*;
        write!(
            f,
            "{}",
            match self {
                Vertical => '║',
                Horizontal => '═',
                CornerNE => '╚',
                CornerNW => '╝',
                CornerSW => '╗',
                CornerSE => '╔',
                Ground => '.',
                StartingPoint => 'S',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connection {
    grid_position: (usize, usize),
    variant: ConnectionVariant,
}

impl From<(ConnectionVariant, usize, usize)> for Connection {
    fn from(value: (ConnectionVariant, usize, usize)) -> Self {
        let (variant, row_num, col_num) = value;
        Self {
            grid_position: (row_num, col_num),
            variant,
        }
    }
}

impl Connection {
    fn connected_to(&self) -> Option<(Direction, Direction)> {
        self.variant.connected_to()
    }

    fn is_other_connected(&self, grid: &Grid, direction: Direction) -> bool {
        let (row, col) = self.grid_position;
        if let Some((row, col)) = direction.translate_coordinates(row, col) {
            if let Some((direct_1, direct_2)) = grid
                .grid
                .get(row)
                .and_then(|row| row.get(col))
                .and_then(|connection| connection.connected_to())
            {
                direct_1.opposite() == direction || direct_2.opposite() == direction
            } else {
                false
            }
        } else {
            false
        }
    }

    fn equivalent_connection(&self, grid: &Grid) -> Result<ConnectionVariant, &'static str> {
        match self.variant {
            ConnectionVariant::StartingPoint => ConnectionVariant::CONNECTED_VARIANTS
                .into_iter()
                .find(|variant| {
                    let (direct_1, direct_2) = variant
                        .connected_to()
                        .expect("(CONNECTED_VARIANT member).connected_to() returned None");

                    self.is_other_connected(grid, direct_1)
                        && self.is_other_connected(grid, direct_2)
                })
                .ok_or("Could not find a corresponding connection variant for StartingPoint"),
            others => Ok(others),
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Grid {
    grid: Box<[Box<[Connection]>]>,
    start_row: usize,
    start_col: usize,
    start_replaced_by_equivalent: bool,
}

impl Grid {
    fn check_grid_integrity(&self) -> bool {
        let mut status = true;
        for (row_index, row) in self.grid.iter().enumerate() {
            for (col_index, val) in row.iter().enumerate() {
                if val.grid_position != (row_index, col_index) {
                    eprintln!(
                        "Expected val.grid_position to be {:?}: was {:?}",
                        (row_index, col_index),
                        val.grid_position
                    );
                    status = false; // don't return, check the rest for logging
                }
            }
        }

        let start_variant = self[(self.start_row, self.start_col)].variant;
        if !self.start_replaced_by_equivalent && start_variant != ConnectionVariant::StartingPoint {
            eprintln!(
                "Expected a starting point at {:?}: found {} ({:?})",
                (self.start_row, self.start_col),
                start_variant,
                start_variant
            );
            status = false;
        }

        status
    }

    fn make_start_into_equivalent(
        &mut self,
    ) -> Result<(Connection, ConnectionVariant), &'static str> {
        let connection = self[(self.start_row, self.start_col)];
        let equivalent = connection.equivalent_connection(self)?;
        let index = (self.start_row, self.start_col);
        self[index] = Connection {
            variant: equivalent,
            ..connection
        };
        self.start_replaced_by_equivalent = true;
        Ok((connection, equivalent))
    }

    fn loop_length(&self) -> usize {
        LoopIterator::new(self)
            //.inspect(|dir| eprintln!("{} ({:?})", dir, dir))
            .count()
    }

    fn get(&self, coord: (usize, usize)) -> Option<&Connection> {
        self.grid.get(coord.0).and_then(|row| row.get(coord.1))
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(for row in self.grid.iter() {
            for conn in row.iter() {
                write!(f, "{}", conn)?;
            }

            writeln!(f)?
        })
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Connection;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.grid[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.grid[index.0][index.1]
    }
}

impl<I> FromIterator<I> for Grid
where
    I: IntoIterator<Item = ConnectionVariant>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut start_row = usize::MAX;
        let mut start_col = usize::MAX;
        let grid = iter
            .into_iter()
            .enumerate()
            .map(|(row_index, inner)| {
                inner
                    .into_iter()
                    .enumerate()
                    .map(|(col_index, connection)| {
                        if connection == ConnectionVariant::StartingPoint {
                            if start_row != usize::MAX || start_col != usize::MAX {
                                panic!("Multiple starting points");
                            }

                            start_row = row_index;
                            start_col = col_index;
                        }

                        Connection::from((connection, row_index, col_index))
                    })
                    .collect()
            })
            .collect();

        if start_row == usize::MAX || start_col == usize::MAX {
            panic!("No starting points found");
        }

        Self {
            grid,
            start_row,
            start_col,
            start_replaced_by_equivalent: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LoopIterator<'g> {
    grid: &'g Grid,
    current_position: (usize, usize),
    from: Direction,
    left_start: bool,
}

impl<'g> Iterator for LoopIterator<'g> {
    type Item = Connection;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left_start && self.current_position == (self.grid.start_row, self.grid.start_col) {
            None
        } else {
            let (direct_1, direct_2) = self.grid[self.current_position]
                .variant
                .connected_to()
                .expect("LoopIterator: Encountered ground, not a closed loop");
            if !(self.move_towards(direct_1) || self.move_towards(direct_2)) {
                panic!("LoopIterator: Cannot move from current position")
            }

            Some(self.grid[self.current_position])
        }
    }
}

impl<'g> LoopIterator<'g> {
    fn new(grid: &'g Grid) -> Self {
        Self {
            grid,
            current_position: (grid.start_row, grid.start_col),
            from: Direction::North, // doesn't matter anyways
            left_start: false,
        }
    }

    fn move_towards(&mut self, direction: Direction) -> bool {
        if self.from == direction {
            false
        } else if let Some(translated) =
            direction.translate_coordinates(self.current_position.0, self.current_position.1)
        {
            if self.grid.get(translated).is_some()
                && self.grid[self.current_position].is_other_connected(self.grid, direction)
            {
                self.current_position = translated;
                self.from = direction.opposite();
                self.left_start = true;
                true
            } else {
                false
            }
        } else {
            false
        }
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
    let grid = input
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                None
            } else {
                Some(
                    line.trim()
                        .chars()
                        .map(ConnectionVariant::try_from)
                        .collect::<Result<Vec<_>, _>>(),
                )
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut grid: Grid = grid.into_iter().collect();
    println!("Grid:\n{}", grid);
    let integrity = grid.check_grid_integrity();
    println!("Grid integrity check: {}", integrity);
    if !integrity {
        return Err("grid.check_grid_integrity() failed".into());
    }

    let (conn, new_variant) = grid.make_start_into_equivalent()?;
    println!("Grid:\n{}", grid);
    println!(
        "Starting Connection: {:?}, new variant: {} ({:?})",
        conn, new_variant, new_variant
    );

    Ok((dbg!(grid.loop_length()) / 2) as u64)
}
