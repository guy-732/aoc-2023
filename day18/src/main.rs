use itertools::Itertools;
use std::{
    collections::HashSet,
    error::Error,
    fs,
    num::ParseIntError,
    ops::{Index, Neg},
    str::FromStr,
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Rect([i32; 4]);

type Point = (i32, i32);
type Segment = [Point; 2];
type RectsGrid = Vec<Vec<Rect>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            'U' => Self::Up,
            'D' => Self::Down,
            'L' => Self::Left,
            'R' => Self::Right,
            other => panic!("Char wasn't a direction ({:?})", other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RGBValue(u32);

impl From<u32> for RGBValue {
    fn from(value: u32) -> Self {
        RGBValue(value)
    }
}

impl FromStr for RGBValue {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches(&['(', '#']);
        let s = s.trim_end_matches(')');
        Ok(u32::from_str_radix(s, 16)?.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DigInstruction {
    direction: Direction,
    distance: u32,
    rgb: RGBValue,
}

impl DigInstruction {
    #[inline]
    pub(crate) const fn direction(&self) -> Direction {
        self.direction
    }

    #[inline]
    pub(crate) const fn distance(&self) -> u32 {
        self.distance
    }

    #[inline]
    pub(crate) const fn rgb(&self) -> RGBValue {
        self.rgb
    }
}

impl FromStr for DigInstruction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: [&str; 3] =
            s.split_whitespace()
                .collect_vec()
                .try_into()
                .or_else(|vec: Vec<&str>| {
                    Err(format!(
                        "Could not split string {:?} into 3 parts (was split into {} parts)",
                        s,
                        vec.len()
                    ))
                })?;

        Ok(Self {
            direction: parts[0].chars().next().unwrap().into(),
            distance: parts[1].parse()?,
            rgb: parts[2].parse()?,
        })
    }
}

impl From<RGBValue> for DigInstruction {
    fn from(value: RGBValue) -> Self {
        let distance: u32 = value.0 >> 4;
        let direction = match value.0 & 0xF {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            other => panic!("Last digit wasn't any of [0, 1, 2, 3] ({})", other),
        };

        Self {
            distance,
            direction,
            rgb: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Dimensions {
    max_up: u64,
    max_down: u64,
    max_left: u64,
    max_right: u64,
}

impl Dimensions {
    #[inline]
    pub(crate) const fn row_count_required(&self) -> u64 {
        self.max_up + self.max_down + 1
    }

    #[inline]
    pub(crate) const fn column_count_required(&self) -> u64 {
        self.max_left + self.max_right + 1
    }

    #[inline]
    pub(crate) const fn starting_row(&self) -> u64 {
        self.max_up
    }

    #[inline]
    pub(crate) const fn starting_column(&self) -> u64 {
        self.max_left
    }

    #[inline]
    pub(crate) fn create_grid(&self) -> Vec<Vec<bool>> {
        let rows = Vec::from_iter((0..self.column_count_required()).map(|_| false));
        let mut result = Vec::from_iter((0..(self.row_count_required() - 1)).map(|_| rows.clone()));
        result.push(rows);
        result
    }
}

impl<'d> FromIterator<&'d DigInstruction> for Dimensions {
    fn from_iter<T: IntoIterator<Item = &'d DigInstruction>>(iter: T) -> Self {
        let mut current_dim = Self::default();
        let mut current_row = 0_i64;
        let mut current_col = 0_i64;

        for instr in iter {
            match instr.direction() {
                Direction::Up => {
                    current_row -= instr.distance() as i64;
                    if current_row < 0 {
                        current_dim.max_up = current_dim.max_up.max(current_row.neg() as u64);
                    }
                }
                Direction::Down => {
                    current_row += instr.distance() as i64;
                    if current_row > 0 {
                        current_dim.max_down = current_dim.max_down.max(current_row as u64);
                    }
                }
                Direction::Left => {
                    current_col -= instr.distance() as i64;
                    if current_col < 0 {
                        current_dim.max_left = current_dim.max_left.max(current_col.neg() as u64);
                    }
                }
                Direction::Right => {
                    current_col += instr.distance() as i64;
                    if current_col > 0 {
                        current_dim.max_right = current_dim.max_right.max(current_col as u64);
                    }
                }
            }
        }

        current_dim
    }
}

impl Index<Direction> for Dimensions {
    type Output = u64;

    #[inline]
    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Up => &self.max_up,
            Direction::Down => &self.max_down,
            Direction::Left => &self.max_left,
            Direction::Right => &self.max_right,
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
    let instructions: Vec<_> = input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.parse::<DigInstruction>())
            }
        })
        .try_collect()?;

    let start = Instant::now();

    let dimensions: Dimensions = instructions.iter().collect();
    // println!("{:#?}", dimensions);

    let mut grid = dimensions.create_grid();
    execute_dig_instructions(
        instructions.iter(),
        &mut grid,
        dimensions.starting_row(),
        dimensions.starting_column(),
    );

    // print_grid(&grid);

    fill_inside_loop(&mut grid);

    // print_grid(&grid);

    let part1_answ = grid.iter().flatten().filter(|&&b| b).count() as u64;

    let part1 = start.elapsed();

    drop(grid);

    // Part 2 start
    let instructions = instructions
        .iter()
        .map(|instr| DigInstruction::from(instr.rgb()))
        .collect_vec();

    let points = read_ngon(&instructions)?;
    let (rects_grid, segments) = rectangular_parts(&points);
    let outside = get_outside(&rects_grid, &segments);
    let part2_answ = get_inside_area(&rects_grid, &outside);

    let part2 = start.elapsed();

    println!("Time to part 1: {:?}", part1);
    println!("Time to part 2: {:?}", part2);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}

#[inline]
fn execute_dig_instructions<'d, T>(
    instructions: T,
    grid: &mut [Vec<bool>],
    starting_row: u64,
    starting_column: u64,
) where
    T: IntoIterator<Item = &'d DigInstruction>,
{
    let mut current_row = starting_row as usize;
    let mut current_column = starting_column as usize;
    grid[current_row][current_column] = true;

    for instr in instructions {
        match instr.direction() {
            Direction::Up => {
                for _ in 0..instr.distance() {
                    current_row -= 1;
                    grid[current_row][current_column] = true;
                }
            }
            Direction::Down => {
                for _ in 0..instr.distance() {
                    current_row += 1;
                    grid[current_row][current_column] = true;
                }
            }
            Direction::Left => {
                for _ in 0..instr.distance() {
                    current_column -= 1;
                    grid[current_row][current_column] = true;
                }
            }
            Direction::Right => {
                for _ in 0..instr.distance() {
                    current_column += 1;
                    grid[current_row][current_column] = true;
                }
            }
        }
    }
}

#[inline]
fn fill_inside_loop(grid: &mut [Vec<bool>]) {
    let mut is_inside = false;
    for row in 0..(grid.len() - 1) {
        for col in 0..grid[row].len() {
            if grid[row][col] && grid[row + 1][col] {
                is_inside = !is_inside;
            }

            if is_inside && !grid[row][col] {
                grid[row][col] = true;
            }
        }
    }
}

#[inline]
fn print_grid(grid: &[Vec<bool>]) {
    for row in grid {
        for &cell in row {
            print!("{}", if cell { '#' } else { '.' });
        }

        println!();
    }

    println!();
}

fn read_ngon(data: &[DigInstruction]) -> Result<Vec<Point>, Box<dyn Error>> {
    let mut pts = Vec::with_capacity(data.len());
    let end = data.iter().fold((0, 0), |(r, c), instr| {
        pts.push((r, c));
        match instr.direction() {
            Direction::Up => (r - instr.distance() as i32, c),
            Direction::Down => (r + instr.distance() as i32, c),
            Direction::Left => (r, c - instr.distance() as i32),
            Direction::Right => (r, c + instr.distance() as i32),
        }
    });
    (end == (0, 0))
        .then_some(pts)
        .ok_or("The polygon does not end where it started!".into())
}

/// Split the ground into (big) rectangles and cut polygon segments on border accordingly.
fn rectangular_parts(pts: &[Point]) -> (RectsGrid, HashSet<Segment>) {
    // Both `rs` and `cs` are reasonably small, leading to a not too big 2D grid.
    let mut rs = pts.iter().map(|(r, _c)| *r).sorted().dedup().collect_vec();
    let mut cs = pts.iter().map(|(_r, c)| *c).sorted().dedup().collect_vec();
    // Add rects on the outside.
    rs.insert(0, rs[0] - 1);
    rs.push(*rs.last().expect("Empty data") + 1);
    cs.insert(0, cs[0] - 1);
    cs.push(*cs.last().expect("Empty data") + 1);
    let segments = pts
        .iter()
        .copied()
        .circular_tuple_windows()
        .flat_map(|((mut r0, mut c0), (mut r1, mut c1))| {
            assert!(r0 == r1 || c0 == c1, "Diagonal?!");
            if r0 == r1 {
                if c0 > c1 {
                    (c0, c1) = (c1, c0);
                }
                cs.iter()
                    .copied()
                    .filter(|&c| c0 <= c && c <= c1)
                    .tuple_windows()
                    .map(|(u, v)| [(r0, u), (r0, v)])
                    .collect_vec()
            } else {
                if r0 > r1 {
                    (r0, r1) = (r1, r0);
                }
                rs.iter()
                    .copied()
                    .filter(|&r| r0 <= r && r <= r1)
                    .tuple_windows()
                    .map(|(u, v)| [(u, c0), (v, c0)])
                    .collect_vec()
            }
        })
        .collect();
    let rects_grid = rs
        .iter()
        .tuple_windows()
        .map(|(r0, r1)| {
            cs.iter()
                .tuple_windows()
                .map(|(c0, c1)| Rect([*r0, *r1, *c0, *c1]))
                .collect()
        })
        .collect();
    (rects_grid, segments)
}

impl Rect {
    const fn border(&self, dir: Direction) -> Segment {
        let a = &self.0;
        match dir {
            Direction::Up => [(a[0], a[2]), (a[0], a[3])],
            Direction::Down => [(a[1], a[2]), (a[1], a[3])],
            Direction::Left => [(a[0], a[2]), (a[1], a[2])],
            Direction::Right => [(a[0], a[3]), (a[1], a[3])],
        }
    }

    fn area(&self) -> u64 {
        let a = &self.0;
        u64::try_from(a[1] - a[0]).expect("Positive length")
            * u64::try_from(a[3] - a[2]).expect("Positive length")
    }
}

fn get_outside(rects_grid: &RectsGrid, segments: &HashSet<Segment>) -> HashSet<(usize, usize)> {
    let nrows = rects_grid.len();
    let ncols = rects_grid[0].len();
    // Since I previously added some space around the polygon,
    // (0, 0) is outside and all the outside is accessible from it.
    let mut stack = vec![(0usize, 0usize)];
    let mut outside = HashSet::new();
    while let Some((r, c)) = stack.pop() {
        if !outside.insert((r, c)) {
            continue; // Visited already.
        }
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let next_loc = match dir {
                Direction::Up => r.checked_sub(1).map(|i| (i, c)),
                Direction::Down => (r + 1 < nrows).then_some((r + 1, c)),
                Direction::Left => c.checked_sub(1).map(|i| (r, i)),
                Direction::Right => (c + 1 < ncols).then_some((r, c + 1)),
            };
            let Some(loc) = next_loc else {
                continue; // Outside the grid.
            };
            if segments.contains(&rects_grid[r][c].border(dir)) {
                continue; // Inside the digged zone.
            }
            stack.push(loc);
        }
    }
    outside
}

fn get_inside_area(rects_grid: &RectsGrid, outside: &HashSet<(usize, usize)>) -> u64 {
    let nrows = rects_grid.len();
    let ncols = rects_grid[0].len();
    let mut total = 0;
    for (r, c) in itertools::iproduct!(0..nrows, 0..ncols) {
        if outside.contains(&(r, c)) {
            continue;
        }
        let rect = &rects_grid[r][c];
        total += rect.area();
        let mut south_east_corner: u8 = 0;
        for (dir, (r0, c0)) in [
            (Direction::Down, (r + 1, c)),
            (Direction::Right, (r, c + 1)),
        ] {
            if r0 < nrows && c0 < ncols && outside.contains(&(r0, c0)) {
                let [p0, p1] = rect.border(dir);
                let segment_length = p1.0 - p0.0 + p1.1 - p0.1;
                total += u64::try_from(segment_length).expect("Positive length");
                if dir == Direction::Right && !outside.contains(&(r - 1, c + 1)) {
                    total -= 1; // Counted twice.
                }
                south_east_corner += 1;
            }
        }
        if south_east_corner == 2 && outside.contains(&(r + 1, c + 1)) {
            total += 1; // Not counted yet.
        }
    }
    total
}
