use core::fmt;
use itertools::Itertools;
use std::{error::Error, fs, str::FromStr, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
struct LinearEquation {
    slope: f64,
    constant: f64,
}

impl LinearEquation {
    #[inline]
    fn new(vx: f64, vy: f64, px: f64, py: f64) -> Self {
        let m = vy / vx;
        Self {
            slope: m,
            constant: py - (m * px),
            // constant: px.mul_add(-m, py), // slower here...
        }
    }

    #[inline]
    fn apply(&self, x: f64) -> f64 {
        // (self.slope * x) + self.constant
        self.slope.mul_add(x, self.constant) // def faster
    }

    /// Returns either:
    /// - `Ok((x, y))` => (x, y) coordinates at which the lines intersect
    /// - `Err(false)` => The lines are parallel but not the same
    /// - `Err(true)` => The lines are the same
    #[inline]
    fn solve_eq(&self, other: &LinearEquation) -> Result<(f64, f64), bool> {
        let slope = self.slope - other.slope; // m_3 = m_1 - m_2
        let constant = other.constant - self.constant; // c_3 = c_2 - c_1
        if slope == 0. {
            Err(constant == 0.)
        } else {
            let x = constant / slope; // slope is non-zero
            let y = self.apply(x);
            // debug_assert_eq!(y, other.apply(x)); // often not equal due to rounding error
            Ok((x, y))
        }
    }
}

impl fmt::Display for LinearEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x + {}", self.slope, self.constant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HailStonePath {
    px: i64,
    py: i64,
    pz: i64,

    vx: i64,
    vy: i64,
    vz: i64,

    z_zero_line: LinearEquation,
}

impl HailStonePath {
    #[inline]
    fn new(px: i64, py: i64, pz: i64, vx: i64, vy: i64, vz: i64) -> Self {
        Self {
            px,
            py,
            pz,
            vx,
            vy,
            vz,
            z_zero_line: LinearEquation::new(vx as f64, vy as f64, px as f64, py as f64),
        }
    }

    #[inline]
    fn contains_x_value(&self, x: f64) -> bool {
        if self.vx.is_negative() {
            x <= self.px as f64
        } else {
            x >= self.px as f64
        }
    }
}

impl FromStr for HailStonePath {
    type Err = Box<dyn Error>;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((position, speed)) = s.split_once('@') else {
            return Err(format!("Could not split {:?} on '@'", s).into());
        };

        let Some((px, py, pz)) = position.split(',').collect_tuple() else {
            return Err(format!("Could not split {:?} on ',' into 3 fields", position).into());
        };

        let Some((vx, vy, vz)) = speed.split(',').collect_tuple() else {
            return Err(format!("Could not split {:?} on ',' into 3 fields", speed).into());
        };

        Ok(Self::new(
            px.trim().parse()?,
            py.trim().parse()?,
            pz.trim().parse()?,
            vx.trim().parse()?,
            vy.trim().parse()?,
            vz.trim().parse()?,
        ))
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 2 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    const LOWER_BOUND_PART_1: f64 = 200000000000000.;
    const UPPER_BOUND_PART_1: f64 = 400000000000000.;

    // const LOWER_BOUND_PART_1: f64 = 7.;
    // const UPPER_BOUND_PART_1: f64 = 27.;

    let input = fs::read_to_string(input)?;
    let hailstones: Vec<HailStonePath> = input.lines().map(|line| line.parse()).try_collect()?;

    let start = Instant::now();

    let part1_answ = hailstones
        .iter()
        .tuple_combinations()
        .map(|(slope_1, slope_2)| {
            (
                slope_1,
                slope_2,
                slope_1.z_zero_line.solve_eq(&slope_2.z_zero_line),
            )
        })
        .filter_map(|(slope_1, slope_2, result)| {
            let result = result.map_or_else(
                |b| {
                    if !b {
                        None
                    } else {
                        // never happened on my input...
                        // so I don't have to implement something to check if they REALLY intersect
                        panic!(
                            "{} and {} are the same",
                            slope_1.z_zero_line, slope_2.z_zero_line
                        );
                    }
                },
                |(x, y)| {
                    (slope_1.contains_x_value(x) && slope_2.contains_x_value(x)).then_some((x, y))
                },
            );

            // if let Some(pair) = result {
            //     eprintln!(
            //         "{} = {} ===> {:?}",
            //         slope_1.z_zero_line, slope_2.z_zero_line, pair
            //     );
            // }
            result
        })
        .filter(|&(x, y)| {
            LOWER_BOUND_PART_1 <= x
                && x <= UPPER_BOUND_PART_1
                && LOWER_BOUND_PART_1 <= y
                && y <= UPPER_BOUND_PART_1
        })
        // .inspect(|v| eprintln!("{:?}", v))
        .count();

    let part1_time = start.elapsed();

    println!("Time for part 1: {:?}", part1_time);
    println!("Part 1 answer: {}", part1_answ);
    todo!()
}
