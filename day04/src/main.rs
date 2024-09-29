use std::{error::Error, fs, num::ParseIntError, str::FromStr};

const INPUT: &str = "input";

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {:#?}", err),
    }
}

fn solve() -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(INPUT)?;
    input
        .lines()
        .filter(|&line| !line.trim().is_empty())
        .map(|line| {
            line.split_once(':')
                .and_then(|(_, data)| data.split_once('|'))
                .map(|(winnings, nums)| {
                    Ok::<_, Box<dyn Error>>((parse_nums_list(winnings)?, parse_nums_list(nums)?))
                })
                .map(|result| result.map(card_winnings))
                .unwrap_or_else(|| {
                    Err(format!(
                        "Line ({line:?}) could not be parsed by spliting with ':' then '|'"
                    )
                    .into())
                })
        })
        .sum()
}

fn parse_nums_list(nums: &str) -> Result<Box<[u64]>, ParseIntError> {
    nums.split_whitespace().map(u64::from_str).collect()
}

fn card_winnings(data: (Box<[u64]>, Box<[u64]>)) -> u64 {
    let (winning_nums, nums) = data;
    let mut winnings = -1;

    for el in nums.into_iter() {
        if winning_nums.contains(el) {
            winnings += 1;
        }
    }

    dbg!(if winnings < 0 { 0 } else { 1 << winnings })
}
