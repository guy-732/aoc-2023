use std::{
    collections::HashMap,
    error::Error,
    fs,
    ops::{Deref, Index},
};

const INPUT: &'static str = "input";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'l' | 'L' => Ok(Self::Left),
            'r' | 'R' => Ok(Self::Right),
            other => Err(format!("Character ({other:?}) was neither 'L' nor 'R'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapValue<'a> {
    left: &'a str,
    right: &'a str,
}

impl<'a> Index<Direction> for MapValue<'a> {
    type Output = &'a str;

    #[inline]
    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

impl<'a, T> Index<T> for MapValue<'a>
where
    T: Deref<Target = Direction>,
{
    type Output = &'a str;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        self.index(*index)
    }
}

#[inline]
fn is_space_or_parentheses(c: char) -> bool {
    c.is_whitespace() || c == '(' || c == ')'
}

impl<'a> TryFrom<&'a str> for MapValue<'a> {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (left, right) = value
            .trim()
            .split_once(',')
            .ok_or("Not a comma separated list of values")?;
        Ok(MapValue {
            left: left.trim_matches(is_space_or_parentheses),
            right: right.trim_matches(is_space_or_parentheses),
        })
    }
}

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {answer}"),
        Err(err) => eprintln!("Error occurred: {err:?}"),
    }
}

fn solve() -> Result<usize, Box<dyn Error>> {
    let input = fs::read_to_string(INPUT)?;
    let mut input = input.lines().filter(|&line| !line.trim().is_empty());
    let directions = input
        .next()
        .ok_or_else(|| format!("File {INPUT:?} does not have a single line"))?
        .chars()
        .filter_map(|c| {
            Direction::try_from(c).map_or_else(
                |err| {
                    eprintln!("Conversion to Direction failed (ignored): {err}");
                    None
                },
                Some,
            )
        })
        .cycle();

    let mut starting_points = Vec::new();
    let map = input
        .map(|line| {
            let (mut prefix, suffix) = line.split_once('=').ok_or("Line did not have char '='")?;
            prefix = prefix.trim();
            if prefix.ends_with('A') {
                starting_points.push(prefix);
            }
            Ok::<_, &'static str>((prefix, MapValue::try_from(suffix)?))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    println!("Directions: {directions:?}");
    println!("Map: {map:#?}");

    let cycles: Box<[usize]> = starting_points
        .into_iter()
        .map(|mut key| {
            directions
                .clone()
                .take_while(|direction| {
                    if key.ends_with('Z') {
                        false
                    } else {
                        key = map[key][direction];
                        true
                    }
                })
                .count()
        })
        .collect();

    println!("Cycles list {cycles:#?}");

    Ok(lcm(&cycles))
}

fn lcm(numbers: &[usize]) -> usize {
    numbers
        .into_iter()
        .fold(1, |acc, &v| acc * (v / gcd(acc, v)))
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else if a > b {
        gcd(b, a % b)
    } else {
        gcd(a, b % a)
    }
}
