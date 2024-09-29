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

    let map = input
        .map(|line| {
            let (prefix, suffix) = line.split_once('=').ok_or("Line did not have char '='")?;
            Ok::<_, &'static str>((prefix.trim(), MapValue::try_from(suffix)?))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    println!("Directions: {directions:?}");
    println!("Map: {map:#?}");

    let mut current_key = "AAA";
    Ok(directions
        .take_while(|direction| {
            if current_key == "ZZZ" {
                false
            } else {
                let val = map[current_key];
                current_key = val[direction];
                true
            }
        })
        .count())
}
