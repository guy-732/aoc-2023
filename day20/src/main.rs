use fnv::FnvHashMap;
use itertools::Itertools;
use std::{collections::VecDeque, error::Error, fs, time::Instant};

const BROADCAST: &str = "broadcaster";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Pulse {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModuleKind<'s> {
    Broadcast,
    FlipFlop(&'s str, bool),
    Conjunction(&'s str, FnvHashMap<&'s str, Pulse>),
}

impl<'s> From<&'s str> for ModuleKind<'s> {
    fn from(value: &'s str) -> Self {
        if value == BROADCAST {
            Self::Broadcast
        } else if let Some(label) = value.strip_prefix('%') {
            Self::FlipFlop(label, false)
        } else if let Some(label) = value.strip_prefix('&') {
            Self::Conjunction(label, FnvHashMap::default())
        } else {
            panic!("module name was neither {BROADCAST:?}, start with '&' or '%' ({value:?})");
        }
    }
}

impl<'s> ModuleKind<'s> {
    #[inline]
    pub(crate) const fn get_module_name(&self) -> &'s str {
        match self {
            Self::Broadcast => BROADCAST,
            Self::FlipFlop(name, _) => name,
            Self::Conjunction(name, _) => name,
        }
    }

    #[inline]
    pub(crate) fn pulse_to_send(&mut self, pulse: Pulse, from: &str) -> Option<Pulse> {
        match self {
            Self::Broadcast => Some(pulse),
            Self::FlipFlop(_, ref mut state) => {
                if matches!(pulse, Pulse::Low) {
                    *state = !(*state);
                    Some(if *state { Pulse::High } else { Pulse::Low })
                } else {
                    None
                }
            }
            Self::Conjunction(_, ref mut map) => {
                *map.get_mut(from)
                    .expect(r#"Unregistered "from" on Conjunction"#) = pulse;
                Some(
                    if map.iter().all(|(_, pulse)| matches!(pulse, Pulse::High)) {
                        Pulse::Low
                    } else {
                        Pulse::High
                    },
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Module<'s> {
    kind: ModuleKind<'s>,
    destinations: Box<[&'s str]>,
}

impl<'s> Module<'s> {
    #[inline]
    pub(crate) const fn get_module_name(&self) -> &'s str {
        self.kind.get_module_name()
    }

    #[inline]
    pub(crate) fn pulse_to_send(&mut self, pulse: Pulse, from: &str) -> Option<Pulse> {
        self.kind.pulse_to_send(pulse, from)
    }
}

impl<'s> From<&'s str> for Module<'s> {
    fn from(value: &'s str) -> Self {
        let (label, destinations) = value
            .split_once(" -> ")
            .expect(r#"Could not split at " -> ""#);

        Self {
            kind: label.into(),
            destinations: destinations.split(',').map(|label| label.trim()).collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct System<'s>(FnvHashMap<&'s str, Module<'s>>);

impl System<'_> {
    #[inline]
    /// First u64 is low pulse count, Second is high pulse count
    /// Third is wether "rx" received a low pulse
    pub(crate) fn push_button(&mut self) -> (u64, u64) {
        let mut low_count = 0;
        let mut high_count = 0;
        let mut pulse_backlog = VecDeque::new();
        pulse_backlog.push_back((BROADCAST, Pulse::Low, "button"));

        while let Some((label, pulse, from)) = pulse_backlog.pop_front() {
            match pulse {
                Pulse::Low => low_count += 1,
                Pulse::High => high_count += 1,
            }

            let Some(module) = self.0.get_mut(label) else {
                continue;
            };

            // eprintln!("{} -{:?}-> {}", from, pulse, label);

            if let Some(pulse) = module.pulse_to_send(pulse, from) {
                for &destination in module.destinations.iter() {
                    pulse_backlog.push_back((destination, pulse, label));
                }
            }
        }

        (low_count, high_count)
    }

    #[inline]
    pub(crate) fn count_until_rx_low(mut self) -> u64 {
        /// Hard coded but I don't care
        ///
        /// Those are all the modules leading to Conjunction "jz"... which leads to "rx"
        const FOUR_PRANKSTERS: [&str; 4] = ["mk", "vf", "rn", "dh"];

        let mut cycles = 0;
        let mut pulse_backlog = VecDeque::new();

        let mut pranksters_map = FnvHashMap::default();

        'bigassloop: loop {
            cycles += 1;
            pulse_backlog.push_back((BROADCAST, Pulse::Low, "button"));

            while let Some((label, pulse, from)) = pulse_backlog.pop_front() {
                let Some(module) = self.0.get_mut(label) else {
                    continue;
                };

                if FOUR_PRANKSTERS.contains(&module.get_module_name())
                    && matches!(pulse, Pulse::Low)
                {
                    if !pranksters_map.contains_key(label) {
                        pranksters_map.insert(label, cycles);
                        if pranksters_map.len() == FOUR_PRANKSTERS.len() {
                            // how does that even work? I don't know.
                            break 'bigassloop lcm(pranksters_map.into_values());
                        }
                    }
                }

                if let Some(pulse) = module.pulse_to_send(pulse, from) {
                    for &destination in module.destinations.iter() {
                        pulse_backlog.push_back((destination, pulse, label));
                    }
                }
            }
        }
    }
}

impl<'s> FromIterator<Module<'s>> for System<'s> {
    fn from_iter<T: IntoIterator<Item = Module<'s>>>(iter: T) -> Self {
        Self(FnvHashMap::from_iter(
            iter.into_iter()
                .map(|module| (module.get_module_name(), module)),
        ))
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
    let modules_vec = input.lines().map_into::<Module<'_>>().collect_vec();

    let mut modules: System<'_> = modules_vec
        .iter()
        .map(|module| {
            if let ModuleKind::Conjunction(label, _) = module.kind {
                Module {
                    destinations: module.destinations.clone(),
                    kind: ModuleKind::Conjunction(
                        label,
                        modules_vec
                            .iter()
                            .filter_map(|module| {
                                if module.destinations.contains(&label) {
                                    Some((module.get_module_name(), Pulse::Low))
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                }
            } else {
                module.clone()
            }
        })
        .collect();

    drop(modules_vec);

    let clone = modules.clone();
    let start = Instant::now();

    let mut current_low = 0;
    let mut current_high = 0;
    for _i in 0..1000 {
        let (low, high) = modules.push_button();
        current_low += low;
        current_high += high;
    }

    let part1_answ = current_low * current_high;

    let part1_time = start.elapsed();

    let part2_answ = clone.count_until_rx_low();

    let part2_time = start.elapsed();

    println!("Time to part 1: {:?}", part1_time);
    println!("Time to part 2: {:?}", part2_time);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}

fn lcm<T: Iterator<Item = u64>>(iter: T) -> u64 {
    iter.fold(1, |acc, v| acc * (v / gcd(acc, v)))
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else if a > b {
        gcd(b, a % b)
    } else {
        gcd(a, b % a)
    }
}
