use std::{error::Error, fs};

const INPUT_FILE: &str = "input";

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {answer}"),
        Err(err) => eprintln!("Error occurred: {err}"),
    }
}

fn solve() -> Result<u32, Box<dyn Error>> {
    Ok(fs::read_to_string(INPUT_FILE)?
        .lines()
        .inspect(|line| eprint!("{:?} => ", line))
        .map(|line| get_number_from_line(line))
        .inspect(|res| eprintln!("{:?}", res))
        .sum())
}

fn get_number_from_line(line: &str) -> u32 {
    let chars = line.chars();
    let val_1 = chars.clone().find_map(|c| c.to_digit(10)).unwrap_or(0) * 10;
    let val_2 = chars.rev().find_map(|c| c.to_digit(10)).unwrap_or(0);
    val_1 + val_2
}
