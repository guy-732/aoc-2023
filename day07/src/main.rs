use std::{error::Error, fs};

const INPUT: &str = "input";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum HandType {
    FiveOfAKind = 6,
    FourOfAKind = 5,
    FullHouse = 4,
    ThreeOfAKind = 3,
    TwoPair = 2,
    OnePair = 1,
    HighCard = 0,
}

impl From<&[Card; 5]> for HandType {
    fn from(value: &[Card; 5]) -> Self {
        <Self as From<[Card; 5]>>::from(*value)
    }
}

impl From<[Card; 5]> for HandType {
    // ... honestly, the jokers are gonna be scary with THAT implementation
    fn from(mut value: [Card; 5]) -> Self {
        value.sort_unstable();
        let mut other_occurrences = 0;
        let mut occurrences = 0;
        let mut last_card = Card::Ace; // doesn't matter which it is
        for card in value {
            if card == last_card {
                occurrences += 1;
                continue;
            }

            if occurrences > 1 {
                if other_occurrences != 0 {
                    break; // It's two pairs, the last card (current iteration) is the unique one
                }

                other_occurrences = occurrences;
            }

            occurrences = 1;
            last_card = card;
        }

        match (other_occurrences, occurrences) {
            (5, _) | (_, 5) => Self::FiveOfAKind,
            (4, _) | (_, 4) => Self::FourOfAKind,
            (3, 2) | (2, 3) => Self::FullHouse,
            (3, _) | (_, 3) => Self::ThreeOfAKind,
            (2, 2) => Self::TwoPair,
            (2, _) | (_, 2) => Self::OnePair,
            _ => Self::HighCard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '2' => Self::Two,
            '3' => Self::Three,
            '4' => Self::Four,
            '5' => Self::Five,
            '6' => Self::Six,
            '7' => Self::Seven,
            '8' => Self::Eight,
            '9' => Self::Nine,
            'T' => Self::Ten,
            'J' => Self::Jack,
            'Q' => Self::Queen,
            'K' => Self::King,
            'A' | '1' => Self::Ace,
            other => Err(format!("Invalid card {other:?}"))?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    hand_type: HandType,
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand_type
            .cmp(&other.hand_type)
            .then_with(|| self.cards.cmp(&other.cards))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<[char; 5]> for Hand {
    type Error = String;

    fn try_from(value: [char; 5]) -> Result<Self, Self::Error> {
        let cards: [Card; 5] = value
            .iter()
            .map(|&v| Card::try_from(v))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .expect("Array of 5 elements didn't map to 5 elements");

        Ok(Self {
            cards,
            hand_type: HandType::from(cards),
        })
    }
}

impl TryFrom<&str> for Hand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let vec = value.chars().collect::<Vec<_>>();
        let sized_arr: [char; 5] = match vec.try_into() {
            Ok(ok) => ok,
            Err(_) => Err(format!(
                "string {value:?} did not have 5 characters (whitespace excluded)"
            ))?,
        };

        <Self as TryFrom<[char; 5]>>::try_from(sized_arr)
    }
}

#[derive(Debug, Clone, Copy, Eq)]
struct HandWithBid {
    bid: u64,
    hand: Hand,
}

impl PartialEq for HandWithBid {
    fn eq(&self, other: &Self) -> bool {
        self.hand.eq(&other.hand)
    }
}

impl Ord for HandWithBid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand.cmp(&other.hand)
    }
}

impl PartialOrd for HandWithBid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<&str> for HandWithBid {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (hand, bid) = value
            .trim()
            .split_once(' ')
            .ok_or_else(|| format!("value ({value:?}) could not be split at a whitespace"))?;

        Ok(Self {
            bid: bid.parse()?,
            hand: Hand::try_from(hand)?,
        })
    }
}

fn main() {
    match solve() {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {:#?}", err),
    }
}

fn solve() -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(INPUT)?;
    let input = input.lines().filter(|&s| !s.trim().is_empty());
    let mut hands = input
        .map(HandWithBid::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    //println!("{:#?}", hands);
    hands.sort();
    Ok(hands
        .into_iter()
        .zip(1..)
        .fold(0, |acc, (hand, rank)| acc + (hand.bid * rank)))
}
