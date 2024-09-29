#[macro_use]
extern crate lazy_static;

use core::panic;
use std::{error::Error, fs};
use regex::{Regex, RegexBuilder};


const INPUT_FILE: &str = "input";

lazy_static! {
    static ref START_OF_LINE: Regex = RegexBuilder::new(r#"^game\s*(\d+)\s*:\s*"#)
        .case_insensitive(true)
        .build()
        .unwrap();
}

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {answer}"),
        Err(err) => eprintln!("Error occurred: {:?}", err),
    }
}

fn solve() -> Result<u32, Box<dyn Error>> {
    Ok(fs::read_to_string(INPUT_FILE)?
        .lines()
        .map(|line| get_game_value(line).unwrap_or(0))
        .sum())
}

fn get_game_value(line: &str) -> Option<u32> {
    check_cubes(dbg!(&line[START_OF_LINE.find(line)?.end()..]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue,
}

fn parse_into_u32_color(count_color_pair: &str) -> (u32, Color) {
    let (num, color) = count_color_pair
        .trim()
        .split_once(' ')
        .expect("Could not split string");

    (
        num.parse().expect("Could not parse"),
        match color.trim() {
            "red" => Color::Red,
            "green" => Color::Green,
            "blue" => Color::Blue,
            other => panic!("Color was neither red, green not blue: {other:?}"),
        },
    )
}

macro_rules! max_assign {
    ($lhs:ident, $rhs:expr) => {
        $lhs = $lhs.max($rhs)
    };
}

fn check_cubes(line: &str) -> Option<u32> {
    let mut max_red = 0;
    let mut max_green = 0;
    let mut max_blue = 0;

    for part in line.split(';') {
        let mut red_count = 0;
        let mut green_count = 0;
        let mut blue_count = 0;

        for pairs in part.split(',') {
            match parse_into_u32_color(pairs) {
                (r, Color::Red) => {
                    red_count += r;
                }

                (g, Color::Green) => {
                    green_count += g;
                }

                (b, Color::Blue) => {
                    blue_count += b;
                }
            }
        }

        max_assign!(max_red, red_count);
        max_assign!(max_green, green_count);
        max_assign!(max_blue, blue_count);
    }

    Some(max_red * max_green * max_blue)
}
