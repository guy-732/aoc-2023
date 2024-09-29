use itertools::Itertools;
use std::{error::Error, fs, ops, str::FromStr, vec};

#[derive(Debug, Clone, Copy)]
pub(crate) struct MapEntry {
    destination_start: u64,
    source_start: u64,
    range_length: u64,
}

impl MapEntry {
    #[inline]
    pub(crate) const fn destination_start(&self) -> u64 {
        self.destination_start
    }

    #[inline]
    pub(crate) const fn source_start(&self) -> u64 {
        self.source_start
    }

    #[inline]
    pub(crate) const fn range_length(&self) -> u64 {
        self.range_length
    }

    #[inline]
    pub(crate) const fn source_one_after_last(&self) -> u64 {
        self.source_start() + self.range_length()
    }

    #[inline]
    pub(crate) const fn source_range(&self) -> ops::Range<u64> {
        self.source_start()..self.source_one_after_last()
    }

    #[inline]
    pub(crate) const fn sort_key(&self) -> u64 {
        self.source_start()
    }

    #[inline]
    pub(crate) fn contains(&self, value: u64) -> bool {
        self.source_range().contains(&value)
    }

    #[inline]
    pub(crate) fn map(&self, value: u64) -> Option<u64> {
        if self.contains(value) {
            Some(self.map_impl(value))
        } else {
            None
        }
    }

    #[inline]
    fn map_impl(&self, value: u64) -> u64 {
        self.destination_start() + value - self.source_start()
    }

    /// The 3 ranges returned corresponds to the following:
    /// - 1st range are values contained before the map entry
    /// - 2nd range are values this map entry supports
    /// - 3rd range are values beyond this map entry
    ///
    /// Empty ranges means that none of the values meets the condition above for that range
    #[inline]
    pub(crate) fn map_range(
        &self,
        range: ops::Range<u64>,
    ) -> (ops::Range<u64>, ops::Range<u64>, ops::Range<u64>) {
        let before = if range.start < self.source_start() {
            range.start..range.end.min(self.source_start())
        } else {
            0..0
        };

        let matching = if range.end <= self.source_start() {
            // a.k.a. if the range ends BEFORE us then this is empty
            0..0
        } else {
            let mut result = (range.start.max(self.source_start()))
                ..(range.end.min(self.source_one_after_last()));

            result.start = self.map_impl(result.start);

            result.end = self.map_impl(result.end);

            result
        };

        let after = if range.end <= self.source_one_after_last() {
            0..0
        } else {
            (range.start.max(self.source_one_after_last()))..range.end
        };

        let res = (before, matching, after);
        // eprintln!("{self:?}: {range:?} => {res:?}");
        res
    }
}

impl FromStr for MapEntry {
    type Err = Box<dyn Error>;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((first_num, rest)) = s.split_once(' ') else {
            return Err(format!("Could not split {:?} into 3 number fields", s).into());
        };

        let Some((second_num, third_num)) = rest.split_once(' ') else {
            return Err(format!("Could not split {:?} into 3 number fields", s).into());
        };

        Ok(Self {
            destination_start: first_num.parse()?,
            source_start: second_num.parse()?,
            range_length: third_num.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Map {
    entries: Box<[MapEntry]>,
}

impl Map {
    #[inline]
    pub(crate) fn map(&self, value: u64) -> u64 {
        let res = self
            .entries
            .iter()
            .find_map(|map| map.map(value))
            .unwrap_or(value);
        // eprintln!("{} --> {}", value, res);
        res
    }

    #[inline]
    pub(crate) fn map_range(&self, mut range: ops::Range<u64>) -> Vec<ops::Range<u64>> {
        let mut res = vec![];
        for map in self.entries.iter() {
            if range.is_empty() {
                break;
            }

            let (before, mapped, after) = map.map_range(range);

            // before doesn't have any mapping
            if !before.is_empty() {
                res.push(before);
            }

            // mapped just got mapped
            if !mapped.is_empty() {
                res.push(mapped);
            }

            // after are values beyond this mapping, so check the next one (they are in sorted order)
            range = after;
        }

        if !range.is_empty() {
            res.push(range);
        }

        res
    }

    #[inline]
    pub(crate) fn map_ranges(&self, ranges: Vec<ops::Range<u64>>) -> Vec<ops::Range<u64>> {
        ranges
            .into_iter()
            .flat_map(|range| self.map_range(range))
            .collect_vec()
        // let mut result = vec![];
        // for range in ranges {
        //     result.extend(self.map_range(range));
        // }

        // result
    }
}

impl<'s> FromIterator<&'s str> for Map {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        let mut entries: Box<[MapEntry]> = match iter
            .into_iter()
            .map(|line| line.trim().parse())
            .try_collect()
        {
            Ok(entries) => entries,
            Err(err) => panic!("Error occurred: {}\nDebug: {:#?}", err, err),
        };

        entries.sort_unstable_by_key(MapEntry::sort_key);

        Self { entries }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AllMaps {
    maps: [Map; 7],
}

impl AllMaps {
    #[inline]
    pub(crate) fn map(&self, mut value: u64) -> u64 {
        for map in self.maps.iter() {
            value = map.map(value);
        }

        value
    }

    #[inline]
    pub(crate) fn map_range(&self, range: SeedRange) -> Vec<ops::Range<u64>> {
        let mut result = vec![range.seed_range()];
        for map in self.maps.iter() {
            result = map.map_ranges(result);
        }

        result
    }
}

impl<'s> FromIterator<&'s str> for AllMaps {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        let mut lines = iter.into_iter();
        let mut maps = vec![];
        for _ in 0..7 {
            maps.push(
                lines
                    .by_ref()
                    .skip_while(|&line| line.trim().is_empty())
                    .skip(1)
                    .take_while(|&line| !line.trim().is_empty())
                    .collect(),
            );
        }

        Self {
            maps: maps.try_into().expect("Vec did not have 7 elements"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SeedRange {
    seed_start: u64,
    seed_length: u64,
}

impl SeedRange {
    #[inline]
    pub(crate) const fn seed_start(&self) -> u64 {
        self.seed_start
    }

    #[inline]
    pub(crate) const fn seed_length(&self) -> u64 {
        self.seed_length
    }

    #[inline]
    pub(crate) const fn seed_range(&self) -> ops::Range<u64> {
        self.seed_start()..(self.seed_start() + self.seed_length())
    }

    #[inline]
    pub(crate) const fn new(seed_start: u64, seed_length: u64) -> Self {
        Self {
            seed_start,
            seed_length,
        }
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
    let mut lines = input.lines();
    let seeds: Box<[u64]> = lines
        .next()
        .expect("Empty input")
        .strip_prefix("seeds:")
        .expect(r#"First line did not start with "seeds:""#)
        .split_whitespace()
        .map(|seed| seed.parse())
        .try_collect()?;

    let maps: AllMaps = lines.collect();

    // println!("{:?}", seeds);
    // println!("{:#?}", maps);

    println!(
        "Part 1 answer: {}",
        seeds
            .iter()
            .map(|&seed| {
                let res = maps.map(seed);
                // eprintln!("{} => {}", seed, res);
                res
            })
            .min()
            .expect("No seeds")
    );

    Ok(part_2(seeds, maps))
}

#[inline]
fn part_2(seeds: Box<[u64]>, maps: AllMaps) -> u64 {
    let seeds = seeds
        .chunks_exact(2)
        .map(|data| SeedRange::new(data[0], data[1]))
        .collect_vec();

    seeds
        .into_iter()
        .map(|range| maps.map_range(range))
        .flatten()
        .map(|range| range.start) // range start is smallest value obviously
        .min()
        .expect("No seeds")
}
