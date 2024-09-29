#![feature(iter_map_windows)]

use std::{error::Error, fs, num::ParseIntError, str::FromStr};

const INPUT: &str = "input";

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {:#?}", err),
    }
}

fn solve() -> Result<i64, Box<dyn Error>> {
    let input = fs::read_to_string(INPUT)?;
    Ok(input
        .lines()
        .filter_map(|line| {
            if line.trim().is_empty() {
                None
            } else {
                Some(
                    line.split_whitespace()
                        .map(i64::from_str)
                        .collect::<Result<Vec<_>, _>>(),
                )
            }
        })
        .map::<Result<_, ParseIntError>, _>(|vec| Ok(get_next_number_of_sequence(vec?)))
        .sum::<Result<i64, ParseIntError>>()?)
}

fn get_next_number_of_sequence(seq: Vec<i64>) -> i64 {
    let mut vec_stack = vec![seq];
    while vec_stack
        .last()
        .expect("Non-empty Vec doesn't have a last element")
        .iter()
        .any(|&val| val != 0)
    {
        vec_stack.push(
            vec_stack
                .last()
                .expect("Non-empty Vec doesn't have a last element")
                .iter()
                .map_windows(|&[a, b]| b - a)
                .collect(),
        );
    }

    vec_stack
        .into_iter()
        .map(|vec| *vec.last().unwrap_or(&0))
        .sum()
}
