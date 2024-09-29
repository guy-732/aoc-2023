use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use rayon::prelude::*;
use std::{error::Error, fs, ops, str::FromStr, time::Instant};

type PositionMember = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: PositionMember,
    y: PositionMember,
    z: PositionMember,
}

impl Position {
    #[inline]
    fn create_x_range(&self, other: &Position) -> ops::RangeInclusive<PositionMember> {
        if self.x > other.x {
            other.x..=self.x
        } else {
            self.x..=other.x
        }
    }

    #[inline]
    fn create_y_range(&self, other: &Position) -> ops::RangeInclusive<PositionMember> {
        if self.y > other.y {
            other.y..=self.y
        } else {
            self.y..=other.y
        }
    }

    #[inline]
    fn create_z_range(&self, other: &Position) -> ops::RangeInclusive<PositionMember> {
        if self.z > other.z {
            other.z..=self.z
        } else {
            self.z..=other.z
        }
    }
}

impl FromStr for Position {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(',');
        Ok(Self {
            x: split
                .next()
                .ok_or_else(|| format!("split iterator is empty???"))?
                .parse()?,
            y: split
                .next()
                .ok_or_else(|| format!("{:?} did not contain 2 ','", s))?
                .parse()?,
            z: split
                .next()
                .ok_or_else(|| format!("{:?} did not contain 2 ','", s))?
                .parse()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Brick {
    brick_ends: (Position, Position),
}

impl Brick {
    #[inline]
    fn create_x_range(&self) -> ops::RangeInclusive<PositionMember> {
        let (left, right) = &self.brick_ends;
        left.create_x_range(right)
    }

    #[inline]
    fn create_y_range(&self) -> ops::RangeInclusive<PositionMember> {
        let (left, right) = &self.brick_ends;
        left.create_y_range(right)
    }

    #[inline]
    #[allow(dead_code)]
    fn create_z_range(&self) -> ops::RangeInclusive<PositionMember> {
        let (left, right) = &self.brick_ends;
        left.create_z_range(right)
    }

    #[inline]
    fn lower_z_position(&self) -> PositionMember {
        let (left, right) = &self.brick_ends;
        left.z.min(right.z)
    }

    #[inline]
    fn higher_z_position(&self) -> PositionMember {
        let (left, right) = &self.brick_ends;
        left.z.max(right.z)
    }

    #[inline]
    fn sort_by_lower_height_key(&self) -> PositionMember {
        self.lower_z_position()
    }

    #[inline]
    fn sort_by_upper_height_key(&self) -> PositionMember {
        self.higher_z_position()
    }

    #[inline]
    fn fall_to_lower_z(&mut self, target_lower_z: PositionMember) {
        let (ref mut left, ref mut right) = self.brick_ends;
        let z_diff = left.z.abs_diff(right.z);
        if left.z > right.z {
            right.z = target_lower_z;
            left.z = target_lower_z + z_diff;
        } else {
            left.z = target_lower_z;
            right.z = target_lower_z + z_diff;
        }
    }

    /// changes position of itself
    fn fall_on_bricks(&mut self, pile: &[Brick]) {
        let target_lower_z = pile
            .iter()
            .rev()
            .find(|&brick| brick.are_aligned_z(self))
            .map(|brick| brick.higher_z_position() + 1)
            .unwrap_or(1);

        self.fall_to_lower_z(target_lower_z);
        // dbg!(target_lower_z, self);
    }

    fn supporting_bricks(&self, pile: &[Brick]) -> FnvHashSet<Brick> {
        let mut result = FnvHashSet::default();
        let relevant_height = self.lower_z_position() - 1;
        if relevant_height == 0 {
            return result;
        }

        for brick in pile.iter().rev() {
            if brick.higher_z_position() < relevant_height {
                break;
            }

            if brick.are_aligned_z(self) {
                result.insert(brick.clone());
            }
        }

        result
    }

    /// check if a brick aligns with another on at least 1 block
    fn are_aligned_z(&self, other: &Brick) -> bool {
        let mut result = false;
        let other_x_range = other.create_x_range();
        let other_y_range = other.create_y_range();
        for x in self.create_x_range() {
            for y in self.create_y_range() {
                if other_x_range.contains(&x) && other_y_range.contains(&y) {
                    result = true;
                    break;
                }
            }
        }

        // eprintln!("are_aligned_z({:?}, {:?}) => {}", self.brick_ends, other.brick_ends, result);
        result
    }

    fn can_safely_remove(&self, supported_by_map: &FnvHashMap<Brick, FnvHashSet<Brick>>) -> bool {
        for set in supported_by_map.values() {
            if set.len() == 1 && set.contains(self) {
                return false;
            }
        }

        true
    }

    fn bricks_falling(
        &self,
        supported_by_map: &FnvHashMap<Brick, FnvHashSet<Brick>>,
        fell: &mut FnvHashSet<Brick>,
    ) -> usize {
        let mut count = 0;
        fell.insert(self.clone());
        for (brick, set) in supported_by_map.iter() {
            if set.contains(self) && set.iter().all(|b| fell.contains(b)) {
                count += brick.bricks_falling(supported_by_map, fell) + 1;
            }
        }

        count
    }
}

impl FromStr for Brick {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split_once('~')
            .ok_or_else(|| format!("{:?} could not be split on '~'", s))?;
        Ok(Self {
            brick_ends: (left.parse()?, right.parse()?),
        })
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 2 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<usize, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;
    let mut raw_bricks: Vec<Brick> = input.lines().map(|line| line.parse()).try_collect()?;

    let start = Instant::now();

    raw_bricks.sort_by_key(Brick::sort_by_lower_height_key);

    let mut supported_by = FnvHashMap::default();
    let mut pile = vec![];
    for mut brick in raw_bricks {
        brick.fall_on_bricks(&pile);
        supported_by.insert(brick.clone(), brick.supporting_bricks(&pile));
        let index = pile
            .binary_search_by_key(
                &brick.sort_by_upper_height_key(),
                Brick::sort_by_upper_height_key,
            )
            .unwrap_or_else(|e| e);

        pile.insert(index, brick);
    }

    // dbg!(pile);
    // dbg!(supported_by);

    let part1_answ = pile
        .iter()
        .filter(|&brick| brick.can_safely_remove(&supported_by))
        .count();
    let part1_time = start.elapsed();

    let part2_answ = pile
        .into_par_iter()
        // .enumerate()
        // .inspect(|(i, _)| println!("Iteration {} starts: {:?}", i, start.elapsed()))
        .map(|brick| brick.bricks_falling(&supported_by, &mut FnvHashSet::default()))
        .sum();

    let part2_time = start.elapsed();

    println!("Time to part 1: {:?}", part1_time);
    println!("Time to part 2: {:?}", part2_time);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}
