use itertools::Itertools;
use std::{error::Error, fmt, fs, ops::{Deref, Index}};

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
    let data: Vec<Vec<CosmosCell>> = input
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
    println!();

    let coords = expand_cosmos(data);

    println!("\nExpanded Cosmos: (not printed cause too big anyways, also not enough memory to store everything)");
    //print_cosmos(&data);

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

const N: usize = 1_000_000;

fn expand_cosmos(cosmos: Vec<Vec<CosmosCell>>) -> Vec<Galaxy> {
    if cosmos.is_empty() {
        return vec![];
    }

    let mut empty_rows = vec![];
    for (row_index, row) in cosmos.iter().enumerate() {
        if row.iter().all(|&cell| cell == CosmosCell::Empty) {
            empty_rows.push(row_index);
        }
    }

    let mut empty_columns = vec![];
    let row = cosmos.index(0);
    for (col_index, _) in row.iter().enumerate() {
        if cosmos
            .iter()
            .map(|row| *row.get(col_index).unwrap_or(&CosmosCell::Empty))
            .all(|cell| cell == CosmosCell::Empty)
        {
            empty_columns.push(col_index);
        }
    }

    println!("Empty Rows: {:?}", empty_rows);
    println!("Empty Columns: {:?}", empty_columns);


    let mut galaxies = vec![];
    let mut current_row = 0;
    for (row_index, row) in cosmos.into_iter().enumerate() {
        let mut current_col = 0;
        for (col_index, cell) in row.into_iter().enumerate() {
            if cell == CosmosCell::Galaxy {
                galaxies.push(Galaxy(current_row, current_col));
            }

            current_col += if empty_columns.contains(&col_index) { N } else { 1 };
        }

        current_row += if empty_rows.contains(&row_index) { N } else { 1 };
    }

    galaxies
}

/* // quarantine
fn expand_cosmos(mut cosmos: Vec<Vec<CosmosCell>>) -> Vec<Vec<CosmosCell>> {
    let mut i = 0;
    while i < cosmos.len() {
        // expand rows
        if cosmos[i].iter().all(|&cell| cell == CosmosCell::Empty) {
            // just realized I don't have to clone the whole line each time...
            //let copy = cosmos[i].clone();
            cosmos
                .splice(i..i, iter::once(vec![]).cycle().take(N))
                .collect_vec();
            i += N;
        } else {
            i += 1;
        }
    }

    i = 0;
    while i < cosmos[0].len() {
        // expand cols
        if cosmos
            .iter()
            .map(|row| *row.get(i).unwrap_or(&CosmosCell::Empty))
            .all(|cell| cell == CosmosCell::Empty)
        {
            cosmos.iter_mut().for_each(|row| {
                row
                    .splice(i..i, iter::once(CosmosCell::Empty).cycle().take(N))
                    .collect_vec();
            });
            i += N;
        } else {
            i += 1;
        }
    }

    cosmos
}
*/