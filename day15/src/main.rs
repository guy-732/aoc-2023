use core::fmt;
use std::{
    error::Error,
    fs,
    ops::{Index, IndexMut},
};

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref LABEL_REGEX: Regex = RegexBuilder::new(r#"^([a-zA-Z]+)([-=])(\d+)?$"#)
        .build()
        .expect("Regex failed to build");
}

#[inline]
pub(crate) fn hash_str(s: &str) -> u8 {
    let mut res: u32 = 0;
    for c in s.chars() {
        res += c as u32;
        res *= 17;
        res &= 0xFF;
    }

    res as u8
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MapEntry<'s> {
    label: &'s str,
    focal: u64,
}

impl<'s> MapEntry<'s> {
    pub(crate) fn new(label: &'s str, focal: u64) -> Self {
        Self {
            label,
            focal,
        }
    }

    pub(crate) fn calculate_power(&self, in_box: u64, slot: u64) -> u64 {
        (in_box + 1) * (slot + 1) * self.focal
    }
}

impl fmt::Debug for MapEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.label, self.focal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Map<'s> {
    boxes: [Vec<MapEntry<'s>>; 256],
}

impl Default for Map<'_> {
    fn default() -> Self {
        let mut vecs = Vec::with_capacity(256);
        for _ in 0..256 {
            vecs.push(vec![]);
        }

        Self {
            boxes: vecs.try_into().expect("Vec was not 256 elements long"),
        }
    }
}

impl<'s> Index<u8> for Map<'s> {
    type Output = [MapEntry<'s>];

    fn index(&self, index: u8) -> &Self::Output {
        &self.boxes[index as usize]
    }
}

impl<'s> IndexMut<u8> for Map<'s> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.boxes[index as usize]
    }
}

impl<'s> Map<'s> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, string: &'s str) {
        let m = match LABEL_REGEX.captures(string) {
            Some(ok) => ok,
            None => panic!("The string {:?} did not match the regex /{}/", string, LABEL_REGEX.as_str()),
        };

        let label = m.get(1).expect("Capture group 1 did not exist").as_str();
        let sign = m[2].chars().next().expect("Capture group 2 captured nothing");
        if sign == '=' {
            let number = m[3].parse::<u64>().expect(r#"Could not parse a \d+ match"#);
            self.insert_equals_impl(label, number);
        } else { // sign == '-'
            self.insert_dash_impl(label);
        }
    }

    fn insert_equals_impl(&mut self, label: &'s str, number: u64) {
        let hash = hash_str(label);
        if let Some(s) = self[hash].iter_mut().find(|entry| entry.label == label) {
            s.focal = number;
        } else {
            self.boxes[hash as usize].push(MapEntry::new(label, number));
        }
    }

    fn insert_dash_impl(&mut self, label: &str) {
        self.boxes[hash_str(label) as usize].retain(|entry| entry.label != label);
    }

    pub(crate) fn print_box(&self, box_to_print: u8) {
        println!("Box {}: {:?}", box_to_print, &self[box_to_print]);
    }

    pub(crate) fn print_non_empty_boxes(&self) {
        for (i, b) in self.boxes.iter().enumerate() {
            if !b.is_empty() {
                self.print_box(i as u8);
            }
        }
    }

    pub(crate) fn calculate_power(&self) -> u64 {
        self.boxes.iter().enumerate()
            .flat_map(|(box_index, b)| {
                b.iter().enumerate()
                    .map(move |(lens_slot, lens)| {
                        let res = lens.calculate_power(box_index as u64, lens_slot as u64);
                        // println!("Box {}: Slot {}: Power of {:?}: {}", box_index, lens_slot, lens, res);
                        res
                    })
            })
            .sum()
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 2 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    let input = fs::read_to_string(input)?;

    let part_1: u64 = input
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                None
            } else {
                Some(hash_str(part) as u64)
            }
        })
        .sum();
    println!("Part 1 answer: {}", part_1);

    let mut hash_map = Map::new();
    for s in input.split(',').filter_map(|part| {
        let part = part.trim();
        if part.is_empty() {
            None
        } else {
            Some(part)
        }
    }) {
        hash_map.insert(s);
        // println!("Inserted {s:?}");
        // hash_map.print_non_empty_boxes();
    }

    // hash_map.print_non_empty_boxes();
    Ok(hash_map.calculate_power())
}
