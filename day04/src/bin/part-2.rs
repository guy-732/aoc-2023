use std::{error::Error, fs, iter::Sum, num::ParseIntError, str::FromStr};

const INPUT: &str = "input";

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {:#?}", err),
    }
}

fn solve() -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(INPUT)?;
    let mut cards = input
        .lines()
        .filter(|&line| !line.trim().is_empty())
        .map(|line| {
            line.split_once(':')
                .and_then(|(_, data)| data.split_once('|'))
                .map(|(winnings, nums)| {
                    Ok::<_, Box<dyn Error>>((parse_nums_list(winnings)?, parse_nums_list(nums)?))
                })
                .map(|result| result.map(ScratchCard::new))
                .unwrap_or_else(|| {
                    Err(format!(
                        "Line ({line:?}) could not be parsed by spliting with ':' then '|'"
                    )
                    .into())
                })
        })
        .collect::<Result<Box<[_]>, _>>()?;

    process_cards(&mut cards);
    dbg!(&cards);
    Ok(cards.into_iter().sum())
}

fn process_cards(cards: &mut [ScratchCard]) {
    for i in 0..cards.len() {
        for j in (i + 1)..=(i + (cards[i].matches as usize)) {
            cards[j].card_count += cards[i].card_count;
        }
    }
}

fn parse_nums_list(nums: &str) -> Result<Box<[u64]>, ParseIntError> {
    nums.split_whitespace().map(u64::from_str).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ScratchCard {
    card_count: u64,
    matches: u64,
}

impl<'a> Sum<&'a ScratchCard> for u64 {
    fn sum<I: Iterator<Item = &'a ScratchCard>>(iter: I) -> Self {
        iter.map(|v| dbg!(v.card_count)).sum()
    }
}

impl ScratchCard {
    fn new(data: (Box<[u64]>, Box<[u64]>)) -> Self {
        let (winning_nums, nums) = data;
        let mut matches = 0;

        for el in nums.into_iter() {
            if winning_nums.contains(el) {
                matches += 1;
            }
        }

        Self {
            card_count: 1,
            matches,
        }
    }
}
