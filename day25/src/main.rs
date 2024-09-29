use fnv::{FnvHashMap, FnvHashSet};
use std::{error::Error, fs, io, iter, collections::VecDeque};

#[derive(Debug, Clone, Default)]
struct Graph<'s> {
    adjacency_list: FnvHashMap<&'s str, FnvHashSet<&'s str>>,
}

impl<'s> Graph<'s> {
    #[inline]
    fn remove_undirected_edge(&mut self, src_vertex: &str, dst_vertex: &str) {
        if let Some(src_adj) = self.adjacency_list.get_mut(src_vertex) {
            src_adj.remove(dst_vertex);
        }

        if let Some(dst_adj) = self.adjacency_list.get_mut(dst_vertex) {
            dst_adj.remove(src_vertex);
        }
    }

    #[inline]
    fn add_undirected_edge(&mut self, src_vertex: &'s str, dst_vertex: &'s str) {
        if let Some(src_adj) = self.adjacency_list.get_mut(src_vertex) {
            src_adj.insert(dst_vertex);
        } else {
            self.adjacency_list
                .insert(src_vertex, iter::once(dst_vertex).collect());
        }

        if let Some(dst_adj) = self.adjacency_list.get_mut(dst_vertex) {
            dst_adj.insert(src_vertex);
        } else {
            self.adjacency_list
                .insert(dst_vertex, iter::once(src_vertex).collect());
        }
    }

    #[inline]
    fn write_as_gv<W: io::Write>(&self, writer: &mut W, layout: &str) -> io::Result<()> {
        writeln!(writer, "graph {{\n    layout={:?}\n", layout)?;

        for &vertex in self.adjacency_list.keys() {
            writeln!(writer, "    {} [label={:?}]", vertex, vertex)?;
        }

        writeln!(writer)?;

        for (&src, dests) in &self.adjacency_list {
            for &dst in dests {
                if src < dst {
                    writeln!(writer, "    {} -- {}", src, dst)?;
                }
            }
        }

        writeln!(writer, "}}")
    }

    #[inline]
    fn count_connected(&self, start: &str) -> u64 {
        let mut queue = VecDeque::from([start]);
        let mut visited = FnvHashSet::from_iter([start]);
        while let Some(vertex) = queue.pop_front() {
            for &dest in &self.adjacency_list[vertex] {
                if visited.insert(dest) {
                    queue.push_back(dest);
                }
            }
        }

        visited.len() as u64
    }
}

impl<'s> FromIterator<&'s str> for Graph<'s> {
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        let mut graph = Self::default();
        for line in iter {
            let Some((src_label, dests)) = line.split_once(':') else {
                panic!("{:?} could not be split on a ':'", line);
            };

            for dest in dests.split_whitespace() {
                graph.add_undirected_edge(src_label, dest);
            }
        }

        graph
    }
}

fn main() {
    match solve("input") {
        Ok(answer) => println!("Part 1 answer: {}", answer),
        Err(err) => eprintln!("Error occurred: {}\nDebug: {:#?}", err, err),
    }
}

fn solve(input: &str) -> Result<u64, Box<dyn Error>> {
    // hardcoded from graphviz's output (input.ex1)
    // const TO_CUT: [(&'static str, &'static str); 3] =
    //     [("hfx", "pzl"), ("bvb", "cmg"), ("jqt", "nvd")];

    // hardcoded from graphviz's output (input)
    const TO_CUT: [(&'static str, &'static str); 3] = [
        ("txm", "fdb"),
        ("mnl", "nmz"),
        ("jpn", "vgf"),
    ];

    let input = fs::read_to_string(input)?;
    let mut graph = input.lines().collect::<Graph>();

    let mut out_file = fs::File::create("input.gv")?;
    graph.write_as_gv(&mut out_file, "neato")?;
    drop(out_file);

    TO_CUT
        .iter()
        .for_each(|&(src, dst)| graph.remove_undirected_edge(src, dst));

    let mut out_file = fs::File::create("input.cut.gv")?;
    graph.write_as_gv(&mut out_file, "neato")?;
    drop(out_file);

    let (section1, section2) = TO_CUT[0];
    let section1_size = graph.count_connected(section1);
    let section2_size = graph.count_connected(section2);
    println!("graph.count_connected({:?}) = {}", section1, section1_size);
    println!("graph.count_connected({:?}) = {}", section2, section2_size);

    Ok(section1_size * section2_size)
}
