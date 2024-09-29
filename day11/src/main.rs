use itertools::Itertools;
use std::{error::Error, fmt, fs, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CosmosCell {
    Empty,
    Galaxy,
}

impl TryFrom<char> for CosmosCell {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '.' => Self::Empty,
            '#' => Self::Galaxy,
            other => Err(format!("Character was neither '.' nor '#' ({:?})", other))?,
        })
    }
}

impl fmt::Display for CosmosCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Empty => '.',
                Self::Galaxy => '#',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Galaxy(usize, usize);

impl Galaxy {
    fn distance_from(&self, other: &Self) -> usize {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;
    let mut data: Vec<Vec<CosmosCell>> = input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.chars().map(CosmosCell::try_from).try_collect())
            }
        })
        .try_collect()?;

    println!("Original Cosmos:");
    print_cosmos(&data);

    expand_cosmos(&mut data);

    println!("\nExpanded Cosmos:");
    print_cosmos(&data);

    let coords = data
        .into_iter()
        .enumerate()
        .flat_map(|(row_index, row)| {
            row.into_iter()
                .enumerate()
                .filter_map(move |(col_index, cosmos)| {
                    if cosmos == CosmosCell::Galaxy {
                        Some(Galaxy(row_index, col_index))
                    } else {
                        None
                    }
                })
        })
        .collect_vec();

    //println!("Coords: {:#?}", &coords);

    Ok(coords
        .into_iter()
        .combinations(2)
        .map(|pair| {
            let first = pair.first().unwrap();
            let second = pair.last().unwrap();
            //eprintln!("({:?}, {:?})", first, second);
            first.distance_from(second) as u64
        })
        .sum())
}

fn print_cosmos<I, I2, C>(iter: I)
where
    I: IntoIterator<Item = I2>,
    I2: IntoIterator<Item = C>,
    C: Deref<Target = CosmosCell>,
{
    for row in iter.into_iter() {
        for cell in row.into_iter() {
            print!("{}", *cell);
        }

        println!();
    }
}

// works
fn expand_cosmos(cosmos: &mut Vec<Vec<CosmosCell>>) {
    let mut i = 0;
    while i < cosmos.len() {
        // expand rows
        if cosmos[i].iter().all(|&cell| cell == CosmosCell::Empty) {
            let copy = cosmos[i].clone();
            cosmos.insert(i + 1, copy);
            i += 1;
        }

        i += 1;
    }

    i = 0;
    while i < cosmos[0].len() {
        // expand cols
        if cosmos
            .iter()
            .map(|row| row[i])
            .all(|cell| cell == CosmosCell::Empty)
        {
            cosmos
                .iter_mut()
                .for_each(|row| row.insert(i + 1, CosmosCell::Empty));
            i += 1;
        }

        i += 1;
    }
}
