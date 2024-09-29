use std::{error::Error, fs, num::ParseIntError, ops::Mul, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RecordData {
    time: u64,
    distance: u64,
}

impl RecordData {
    fn new(time: u64, distance: u64) -> Self {
        Self { time, distance }
    }

    fn count_ways_to_beat(&self) -> u64 {
        (1..self.time)
            .filter(|&time_held| (self.time - time_held) * time_held > self.distance)
            .count() as u64
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {:#?}", err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;
    let mut lines = input.lines();
    let times = lines
        .next()
        .and_then(|line| line.strip_prefix("Time:"))
        .ok_or(r#"The first line did not start with "Time:""#)?
        .split_whitespace()
        .map(u64::from_str);
    let distances = lines
        .next()
        .and_then(|line| line.strip_prefix("Distance:"))
        .ok_or(r#"The second line did not start with "Distance:""#)?
        .split_whitespace()
        .map(u64::from_str);
    let records = times
        .zip(distances)
        .map(|(time, distance)| Ok::<_, ParseIntError>(RecordData::new(time?, distance?)))
        .collect::<Result<Vec<_>, _>>()?;

    println!("Data: {:#?}", &records);
    Ok(records
        .into_iter()
        .map(|record| record.count_ways_to_beat())
        .fold(1, u64::mul))
}
