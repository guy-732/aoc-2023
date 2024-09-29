use std::{error::Error, fs};

use regex::{Match, Regex, RegexBuilder};

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

#[derive(Debug)]
struct DigitIterator<'a> {
    string: &'a str,
    offset: usize,
    re: Regex,
}

impl Iterator for DigitIterator<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.re.find_at(self.string, self.offset).map(|m| {
            self.offset = m.start() + 1; // NOT from end
            Self::match_to_digit(&m)
        })
    }
}

impl<'a> DigitIterator<'a> {
    const REGEX_COMPONENTS: [&'static str; 10] = [
        "[1-9]", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    fn build_regex() -> regex::Regex {
        RegexBuilder::new(&Self::REGEX_COMPONENTS.join("|"))
            .build()
            .unwrap()
    }

    fn from(string: &'a str) -> Self {
        Self {
            string,
            offset: 0,
            re: Self::build_regex(),
        }
    }

    fn match_to_digit(m: &Match<'_>) -> u32 {
        match m.as_str() {
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            digit => digit.parse().unwrap_or_else(|err| {
                eprintln!("match fell though ({digit:?}) was not a digit (err: {err})");
                panic!()
            }),
        }
    }
}

fn get_number_from_line(line: &str) -> u32 {
    let mut iter = DigitIterator::from(line);
    let first = iter.next().expect("Not a single digit in line");
    let second = iter.last().unwrap_or(first);
    (first * 10) + second
}
