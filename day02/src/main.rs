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

const MAX_RED_CUBES: u32 = 12;
const MAX_GREEN_CUBES: u32 = 13;
const MAX_BLUE_CUBES: u32 = 14;

fn solve() -> Result<u32, Box<dyn Error>> {
    Ok(fs::read_to_string(INPUT_FILE)?
        .lines()
        .map(|line| get_game_value(line).unwrap_or(0))
        .sum())
}

fn get_game_value(line: &str) -> Option<u32> {
    let capture = START_OF_LINE.captures(line)?;
    let game_number = capture
        .get(1)?
        .as_str()
        .parse::<u32>()
        .expect("Failed to parse a \\d+ regex match");

    check_cubes(dbg!(&line[capture.get(0).unwrap().end()..]))?;

    Some(game_number)
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

fn check_cubes(line: &str) -> Option<()> {
    for part in line.split(';') {
        let mut red_count = 0;
        let mut green_count = 0;
        let mut blue_count = 0;

        for pairs in part.split(',') {
            match parse_into_u32_color(pairs) {
                (r, Color::Red) => {
                    red_count += r;
                    if red_count > MAX_RED_CUBES {
                        return None;
                    }
                }

                (g, Color::Green) => {
                    green_count += g;
                    if green_count > MAX_GREEN_CUBES {
                        return None;
                    }
                }

                (b, Color::Blue) => {
                    blue_count += b;
                    if blue_count > MAX_BLUE_CUBES {
                        return None;
                    }
                }
            }
        }
    }

    Some(())
}
