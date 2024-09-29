use itertools::Itertools;
use std::{
    collections::HashMap,
    error::Error,
    fmt, fs,
    iter::{Product, Sum},
    ops::Deref,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EngineCell {
    Digit(u8),
    Symbol(char),
    Nothing,
    Gear,
}

impl From<char> for EngineCell {
    fn from(value: char) -> Self {
        match value {
            '.' => Self::Nothing,
            c @ '0'..='9' => Self::Digit(c.to_digit(10).unwrap() as u8),
            '*' => Self::Gear,
            symbol => Self::Symbol(symbol),
        }
    }
}

impl fmt::Display for EngineCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => write!(f, "."),
            Self::Digit(d) => write!(f, "{}", d),
            Self::Symbol(c) => write!(f, "{}", c),
            Self::Gear => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PartNumber {
    number: u64,
    line_no: usize,
    column_no: usize,
    length: usize,
}

impl PartNumber {
    fn from_vec(vec: &[Vec<EngineCell>]) -> Vec<Self> {
        let mut result = vec![];
        for (line_no, inner) in vec.iter().enumerate() {
            let mut current_number = 0;
            let mut starting_column = 0;
            let mut was_last_digit = false;
            let mut last_col = 0;
            for (column_no, &cell) in inner.iter().enumerate() {
                last_col = column_no;
                match cell {
                    EngineCell::Digit(digit) => {
                        if was_last_digit {
                            current_number *= 10;
                            current_number += digit as u64;
                        } else {
                            was_last_digit = true;
                            starting_column = column_no;
                            current_number = digit as u64;
                        }
                    }

                    _ => {
                        if was_last_digit {
                            was_last_digit = false;
                            result.push(PartNumber {
                                number: current_number,
                                line_no,
                                column_no: starting_column,
                                length: column_no - starting_column,
                            });
                        }
                    }
                }
            }

            if was_last_digit {
                result.push(PartNumber {
                    number: current_number,
                    line_no,
                    column_no: starting_column,
                    length: last_col - starting_column + 1,
                });
            }
        }

        result
    }

    fn adjacent_gear(&self, vec: &[Vec<EngineCell>]) -> Option<(usize, usize)> {
        let start_line = self.line_no.checked_sub(1).unwrap_or(self.line_no);
        let end_line = vec.len().min(self.line_no + 2);
        //let line_range = start_line..end_line;

        let start_col = self.column_no.checked_sub(1).unwrap_or(self.column_no);
        let end_col = vec[0].len().min(self.column_no + self.length + 1);
        //let column_range = start_col..end_col;

        for row in start_line..end_line {
            for col in start_col..end_col {
                match vec[row][col] {
                    EngineCell::Gear => {
                        return Some((row, col));
                    }
                    _ => (),
                }
            }
        }

        //eprintln!("Not counting {:?}", self);
        None
    }

    fn is_adjacent_to_symbol(&self, vec: &[Vec<EngineCell>]) -> bool {
        let start_line = self.line_no.checked_sub(1).unwrap_or(self.line_no);
        let end_line = vec.len().min(self.line_no + 2);
        //let line_range = start_line..end_line;

        let start_col = self.column_no.checked_sub(1).unwrap_or(self.column_no);
        let end_col = vec[0].len().min(self.column_no + self.length + 1);
        //let column_range = start_col..end_col;

        for row in start_line..end_line {
            for col in start_col..end_col {
                match vec[row][col] {
                    EngineCell::Symbol(_) | EngineCell::Gear => {
                        return true;
                    }
                    _ => (),
                }
            }
        }

        //eprintln!("Not counting {:?}", self);
        false
    }
}

impl Sum<PartNumber> for u64 {
    fn sum<I: Iterator<Item = PartNumber>>(iter: I) -> Self {
        iter.map(|part| part.number).sum()
    }
}

impl Product<PartNumber> for u64 {
    fn product<I: Iterator<Item = PartNumber>>(iter: I) -> Self {
        iter.map(|part| part.number).product()
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
    let engine = input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.chars().map_into::<EngineCell>().collect_vec())
            }
        })
        .collect_vec();

    print_engine(&engine);
    let parts = PartNumber::from_vec(&engine);
    //println!("Parts: {:#?}", parts);
    let mut map: HashMap<(usize, usize), Vec<PartNumber>> = HashMap::new();
    for part in parts.into_iter() {
        if let Some(pos) = part.adjacent_gear(&engine) {
            if let Some(vec) = map.get_mut(&pos) {
                vec.push(part);
            } else {
                map.insert(pos, vec![part]);
            }
        }
    }

    Ok(map
        .into_values()
        .filter_map(|parts| {
            if parts.len() < 2 {
                None
            } else {
                Some(parts.into_iter().product::<u64>())
            }
        })
        //.inspect(|val| {
        //    dbg!(val);
        //})
        .sum())
}

fn print_engine<I, I2, C>(iter: I)
where
    I: IntoIterator<Item = I2>,
    I2: IntoIterator<Item = C>,
    C: Deref<Target = EngineCell>,
{
    for row in iter.into_iter() {
        for cell in row.into_iter() {
            print!("{}", *cell);
        }

        println!();
    }
}
