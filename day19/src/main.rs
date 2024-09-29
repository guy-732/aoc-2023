use itertools::Itertools;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    ops::{Index, Range},
    str::FromStr,
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    ExtremelyCoolLooking,
    Musical,
    Aerodynamic,
    Shiny,
}

impl From<char> for Category {
    fn from(value: char) -> Self {
        match value {
            'x' => Self::ExtremelyCoolLooking,
            'm' => Self::Musical,
            'a' => Self::Aerodynamic,
            's' => Self::Shiny,
            other => panic!("Category was not any of ['x', 'm', 'a', 's'] ({:?})", other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WorkflowConditionDetails {
    category: Category,
    compare_value: u32,
}

impl WorkflowConditionDetails {
    #[inline]
    pub(crate) fn map_greater(
        &self,
        part: PartRatingsRange,
    ) -> (PartRatingsRange, PartRatingsRange) {
        match self.category {
            Category::ExtremelyCoolLooking => (
                PartRatingsRange {
                    x: (part.x.start.max(self.compare_value + 1))..(part.x.end),
                    ..part.clone()
                },
                PartRatingsRange {
                    x: (part.x.start)..(part.x.end.min(self.compare_value + 1)),
                    ..part
                },
            ),
            Category::Musical => (
                PartRatingsRange {
                    m: (part.m.start.max(self.compare_value + 1))..(part.m.end),
                    ..part.clone()
                },
                PartRatingsRange {
                    m: (part.m.start)..(part.m.end.min(self.compare_value + 1)),
                    ..part
                },
            ),
            Category::Aerodynamic => (
                PartRatingsRange {
                    a: (part.a.start.max(self.compare_value + 1))..(part.a.end),
                    ..part.clone()
                },
                PartRatingsRange {
                    a: (part.a.start)..(part.a.end.min(self.compare_value + 1)),
                    ..part
                },
            ),
            Category::Shiny => (
                PartRatingsRange {
                    s: (part.s.start.max(self.compare_value + 1))..(part.s.end),
                    ..part.clone()
                },
                PartRatingsRange {
                    s: (part.s.start)..(part.s.end.min(self.compare_value + 1)),
                    ..part
                },
            ),
        }
    }

    #[inline]
    pub(crate) fn map_lesser(
        &self,
        part: PartRatingsRange,
    ) -> (PartRatingsRange, PartRatingsRange) {
        match self.category {
            Category::ExtremelyCoolLooking => (
                PartRatingsRange {
                    x: (part.x.start)..(part.x.end.min(self.compare_value)),
                    ..part.clone()
                },
                PartRatingsRange {
                    x: (part.x.start.max(self.compare_value))..(part.x.end),
                    ..part
                },
            ),
            Category::Musical => (
                PartRatingsRange {
                    m: (part.m.start)..(part.m.end.min(self.compare_value)),
                    ..part.clone()
                },
                PartRatingsRange {
                    m: (part.m.start.max(self.compare_value))..(part.m.end),
                    ..part
                },
            ),
            Category::Aerodynamic => (
                PartRatingsRange {
                    a: (part.a.start)..(part.a.end.min(self.compare_value)),
                    ..part.clone()
                },
                PartRatingsRange {
                    a: (part.a.start.max(self.compare_value))..(part.a.end),
                    ..part
                },
            ),
            Category::Shiny => (
                PartRatingsRange {
                    s: (part.s.start)..(part.s.end.min(self.compare_value)),
                    ..part.clone()
                },
                PartRatingsRange {
                    s: (part.s.start.max(self.compare_value))..(part.s.end),
                    ..part
                },
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WorkflowCondition {
    Greater(WorkflowConditionDetails),
    Lesser(WorkflowConditionDetails),
    AlwaysTrue,
}

impl FromStr for WorkflowCondition {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::AlwaysTrue)
        } else if let Some((category, compare_value)) = s.split_once('<') {
            if category.len() != 1 {
                Err(format!("Category should be 1 character, was {:?}", category).into())
            } else {
                Ok(Self::Lesser(WorkflowConditionDetails {
                    category: category.chars().next().unwrap().into(),
                    compare_value: compare_value.parse()?,
                }))
            }
        } else if let Some((category, compare_value)) = s.split_once('>') {
            if category.len() != 1 {
                Err(format!("Category should be 1 character, was {:?}", category).into())
            } else {
                Ok(Self::Greater(WorkflowConditionDetails {
                    category: category.chars().next().unwrap().into(),
                    compare_value: compare_value.parse()?,
                }))
            }
        } else {
            Err(format!("{:?} could not be parsed into a WorkflowCondition", s).into())
        }
    }
}

impl WorkflowCondition {
    #[inline]
    pub(crate) fn is_condition_true(&self, part: &PartRatings) -> bool {
        match self {
            Self::Greater(details) => part[details.category] > details.compare_value,
            Self::Lesser(details) => part[details.category] < details.compare_value,
            Self::AlwaysTrue => true,
        }
    }

    #[inline]
    pub(crate) fn map_range(&self, part: PartRatingsRange) -> (PartRatingsRange, PartRatingsRange) {
        // let source = part.clone();
        let result = match self {
            Self::AlwaysTrue => (
                part,
                PartRatingsRange {
                    x: 0..0,
                    m: 0..0,
                    a: 0..0,
                    s: 0..0,
                },
            ),
            Self::Greater(details) => details.map_greater(part),
            Self::Lesser(details) => details.map_lesser(part),
        };

        // eprintln!("{:?}: Source: {:?} ==> {:?}", self, source, result);

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct WorkflowPart<'s> {
    condition: WorkflowCondition,
    if_true: &'s str,
}

impl<'s> WorkflowPart<'s> {
    #[inline]
    pub(crate) fn is_condition_true(&self, part: &PartRatings) -> bool {
        self.condition.is_condition_true(part)
    }

    #[inline]
    pub(crate) const fn get_target_flow(&self) -> &'s str {
        self.if_true
    }

    #[inline]
    /// The first value is mapped to this workflow part, the second is not
    pub(crate) fn map_range(&self, part: PartRatingsRange) -> (PartRatingsRange, PartRatingsRange) {
        self.condition.map_range(part)
    }
}

impl<'s> TryFrom<&'s str> for WorkflowPart<'s> {
    type Error = Box<dyn Error>;

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        if let Some((condition, if_true)) = s.split_once(':') {
            Ok(Self {
                condition: condition.parse()?,
                if_true,
            })
        } else {
            Ok(Self {
                condition: "".parse()?,
                if_true: s,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Workflow<'s> {
    workflow_name: &'s str,
    conditions: Box<[WorkflowPart<'s>]>,
}

impl<'s> Workflow<'s> {
    #[inline]
    pub(crate) fn execute_workflow(&self, part: &PartRatings) -> &'s str {
        for flow in self.conditions.iter() {
            if flow.is_condition_true(part) {
                return flow.get_target_flow();
            }
        }

        panic!("Workflow::execute_workflow(): Unreachable");
    }

    #[inline]
    pub(crate) fn execute_on_range(
        &self,
        part: PartRatingsRange,
    ) -> Vec<(&'s str, PartRatingsRange)> {
        let mut result = vec![];
        let mut current = part;
        for flow in self.conditions.iter() {
            let (mapped, non_mapped) = flow.map_range(current);
            if !mapped.is_empty() {
                result.push((flow.get_target_flow(), mapped));
            }

            current = non_mapped;
            if current.is_empty() {
                break;
            }
        }

        if !current.is_empty() {
            panic!("Unreachable");
        }

        result
    }
}

impl<'s> TryFrom<&'s str> for Workflow<'s> {
    type Error = Box<dyn Error>;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        let value = value
            .strip_suffix('}')
            .ok_or("Workflow::try_from(): value did not end with '}'")?;
        let (workflow_name, conditions) = value.split_once('{').ok_or_else(|| {
            format!(
                "Workflow::try_from(): {:?} could nto be splut at '{{'",
                value
            )
        })?;

        Ok(Self {
            workflow_name,
            conditions: conditions
                .split(',')
                .map(|part| part.try_into())
                .try_collect()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PartRatings {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}

impl PartRatings {
    #[inline]
    pub(crate) const fn sum(&self) -> u32 {
        self.x + self.m + self.a + self.s
    }

    #[inline]
    pub(crate) fn is_accepted(&self, workflows: &HashMap<&str, Workflow<'_>>) -> bool {
        // dbg!(self);
        let mut current_flow = "in";
        loop {
            // dbg!(current_flow);
            if current_flow == "A" {
                break true;
            }

            if current_flow == "R" {
                break false;
            }

            let workflow = workflows
                .get(current_flow)
                .ok_or_else(|| format!("The workflow {:?} does not exist", current_flow))
                .unwrap();

            current_flow = workflow.execute_workflow(self);
        }
    }
}

impl FromStr for PartRatings {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('{').trim_end_matches('}');
        let mut splits = s.split(',');
        let x = splits
            .next()
            .ok_or("Expected 3 ',', found -1???")?
            .strip_prefix("x=")
            .ok_or(r#"Expected first value to start with "x=""#)?
            .parse()?;
        let m = splits
            .next()
            .ok_or("Expected 3 ',', found none")?
            .strip_prefix("m=")
            .ok_or(r#"Expected second value to start with "m=""#)?
            .parse()?;
        let a = splits
            .next()
            .ok_or("Expected 3 ',', found 1")?
            .strip_prefix("a=")
            .ok_or(r#"Expected third value to start with "a=""#)?
            .parse()?;
        let s = splits
            .next()
            .ok_or("Expected 3 ',', found 2")?
            .strip_prefix("s=")
            .ok_or(r#"Expected fourth value to start with "s=""#)?
            .parse()?;
        Ok(Self { x, m, a, s })
    }
}

impl Index<Category> for PartRatings {
    type Output = u32;

    fn index(&self, index: Category) -> &Self::Output {
        match index {
            Category::ExtremelyCoolLooking => &self.x,
            Category::Musical => &self.m,
            Category::Aerodynamic => &self.a,
            Category::Shiny => &self.s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PartRatingsRange {
    x: Range<u32>,
    m: Range<u32>,
    a: Range<u32>,
    s: Range<u32>,
}

impl Default for PartRatingsRange {
    #[inline]
    fn default() -> Self {
        Self {
            x: 1..4001,
            m: 1..4001,
            a: 1..4001,
            s: 1..4001,
        }
    }
}

impl PartRatingsRange {
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.x.is_empty() || self.m.is_empty() || self.a.is_empty() || self.s.is_empty()
    }

    #[inline]
    pub(crate) fn count_values(&self) -> u64 {
        (self.x.clone().count() as u64)
            * (self.m.clone().count() as u64)
            * (self.a.clone().count() as u64)
            * (self.s.clone().count() as u64)
    }

    #[inline]
    pub(crate) fn pass_through_workflow(
        self,
        workflows: &HashMap<&str, Workflow<'_>>,
    ) -> Vec<PartRatingsRange> {
        let mut result = vec![];
        let mut stack = vec![("in", self)];
        while let Some((workflow, range)) = stack.pop() {
            if workflow == "A" {
                result.push(range);
                continue;
            }

            if workflow == "R" {
                continue;
            }

            let workflow = workflows
                .get(workflow)
                .ok_or_else(|| format!("The workflow {:?} does not exist", workflow))
                .unwrap();

            stack.extend(workflow.execute_on_range(range));
        }

        result
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
    let workflows: Vec<Workflow<'_>> = lines
        .by_ref()
        .take_while(|&line| !line.trim().is_empty())
        .map(|line| Workflow::try_from(line.trim()))
        .try_collect()?;

    // println!("{:#?}", workflows);
    let workflows: HashMap<&'_ str, Workflow<'_>> = HashMap::from_iter(
        workflows
            .into_iter()
            .map(|workflow| (workflow.workflow_name, workflow)),
    );

    let parts: Vec<PartRatings> = lines
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.parse())
            }
        })
        .try_collect()?;

    // println!("{:#?}", parts);

    let start = Instant::now();

    let part1_answ: u64 = parts
        .iter()
        .filter_map(|&part| {
            if part.is_accepted(&workflows) {
                Some(part.sum() as u64)
            } else {
                None
            }
        })
        .sum();

    let part1_time = start.elapsed();

    let ranges = PartRatingsRange::default().pass_through_workflow(&workflows);
    let part2_answ = ranges.into_iter().map(|range| range.count_values()).sum();

    let part2_time = start.elapsed();

    println!("Time to part 1: {:?}", part1_time);
    println!("Time to part 2: {:?}", part2_time);
    println!("Part 1 answer: {}", part1_answ);
    Ok(part2_answ)
}
