use std::collections::{HashMap, HashSet, VecDeque};
use csv::Reader;

#[derive(Debug)]
struct Graph {
    adjacency_list: HashMap<String, HashSet<(String, u32)>>,
}

impl Graph {
    fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, origin: &str, destination: &str, flights: u32) {
        self.adjacency_list
            .entry(origin.to_string())
            .or_insert_with(HashSet::new)
            .insert((destination.to_string(), flights));

        self.adjacency_list
            .entry(destination.to_string())
            .or_insert_with(HashSet::new)
            .insert((origin.to_string(), flights));
    }

    fn shortest_paths(&self, start: &str) -> HashMap<String, u64> {
        let mut distances: HashMap<String, u64> = self
            .adjacency_list
            .keys()
            .map(|node| (node.clone(), u64::MAX))
            .collect();
        let mut queue = VecDeque::new();
    
        if let Some(start_distance) = distances.get_mut(start) {
            *start_distance = 0;
        }
        queue.push_back(start.to_string());
    
        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for (neighbor, weight) in neighbors {
                    let new_distance = current_distance.saturating_add(*weight as u64);
                    if new_distance < distances[neighbor] {
                        distances.insert(neighbor.clone(), new_distance);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    
        distances
    }
    fn closeness_centrality(&self) -> HashMap<String, f64> {
        let mut centrality = HashMap::new();
    
        for node in self.adjacency_list.keys() {
            let shortest_paths = self.shortest_paths(node);
            let total_distance: u64 = shortest_paths
                .values()
                .filter(|&&d| d < u64::MAX) // Exclude unreachable nodes
                .sum();
    
            if total_distance > 0 {
                centrality.insert(node.clone(), 1.0 / total_distance as f64);
            } else {
                centrality.insert(node.clone(), 0.0);
            }
        }
    
        centrality
    }      

    // Calculate betweenness centrality
    fn betweenness_centrality(&self) -> HashMap<String, f64> {
        let mut centrality = HashMap::new();
        for node in self.adjacency_list.keys() {
            centrality.insert(node.clone(), 0.0);
        }

        for start in self.adjacency_list.keys() {
            let (shortest_paths, path_counts) = self.all_pairs_shortest_paths(start);

            for (target, paths_through_target) in path_counts.iter() {
                for (node, count) in paths_through_target.iter() {
                    if node != start && node != target {
                        *centrality.get_mut(node).unwrap() += *count as f64 / shortest_paths[target] as f64;
                    }
                }
            }
        }

        centrality
    }

    fn all_pairs_shortest_paths(
        &self,
        start: &str,
    ) -> (HashMap<String, u32>, HashMap<String, HashMap<String, u32>>) {
        let mut distances: HashMap<String, u32> = self
            .adjacency_list
            .keys()
            .map(|node| (node.clone(), u32::MAX))
            .collect();
        let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut path_counts: HashMap<String, u32> = HashMap::new();

        for node in self.adjacency_list.keys() {
            path_counts.insert(node.clone(), 0);
        }
        if let Some(start_distance) = distances.get_mut(start) {
            *start_distance = 0;
        }
        path_counts.insert(start.to_string(), 1);
        queue.push_back(start.to_string());

        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for (neighbor, _) in neighbors {
                    if distances[neighbor] == u32::MAX {
                        distances.insert(neighbor.clone(), current_distance + 1);
                        queue.push_back(neighbor.clone());
                    }
                    if distances[neighbor] == current_distance + 1 {
                        path_counts.insert(
                            neighbor.clone(),
                            path_counts[neighbor] + path_counts[&current],
                        );
                        predecessors
                            .entry(neighbor.clone())
                            .or_insert_with(Vec::new)
                            .push(current.clone());
                    }
                }
            }
        }

        (distances, build_path_counts(&predecessors, start))
    }
}

fn build_path_counts(
    predecessors: &HashMap<String, Vec<String>>,
    start: &str,
) -> HashMap<String, HashMap<String, u32>> {
    let mut counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
    let mut stack: Vec<String> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    stack.push(start.to_string());

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());

        let mut current_counts: HashMap<String, u32> = HashMap::new();
        for pred in predecessors.get(&node).unwrap_or(&Vec::new()) {
            *current_counts.entry(pred.clone()).or_insert(0) += 1;
            stack.push(pred.clone());
        }
        counts.insert(node.clone(), current_counts);
    }

    counts
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "International_Report_Departures.csv"; 
    let mut graph = Graph::new();

    // Read the CSV file using the csv crate
    let mut rdr = Reader::from_path(file_path)?;
    for result in rdr.records() {
        let record = result?;
        let origin = record.get(4).ok_or("Missing usg_apt (origin)")?;
        let destination = record.get(7).ok_or("Missing fg_apt (destination)")?;
        let flights: u32 = record
            .get(14)
            .ok_or("Missing Total (flights)")?
            .parse()
            .unwrap_or(1);

        graph.add_edge(origin, destination, flights);
    }

    // Perform analysis
    let closeness = graph.closeness_centrality();
    println!("Closeness centrality: {:?}", closeness);

    let betweenness = graph.betweenness_centrality();
    println!("Betweenness centrality: {:?}", betweenness);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 5);
        graph.add_edge("B", "C", 10);

        assert_eq!(graph.adjacency_list.len(), 3);
        assert!(graph.adjacency_list.contains_key("A"));
        assert!(graph.adjacency_list.contains_key("B"));
        assert!(graph.adjacency_list.contains_key("C"));
    }

    #[test]
    fn test_degree_centrality() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 5);
        graph.add_edge("B", "C", 10);
        graph.add_edge("C", "A", 15);

        let centrality = graph.closeness_centrality();

        assert!(centrality["A"] > 0.0);
        assert!(centrality["B"] > 0.0);
        assert!(centrality["C"] > 0.0);
    }

    #[test]
    fn test_shortest_paths() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 5);
        graph.add_edge("B", "C", 10);
        graph.add_edge("A", "C", 20);

        let shortest_paths = graph.shortest_paths("A");

        assert_eq!(shortest_paths["A"], 0);
        assert_eq!(shortest_paths["B"], 1); // Assuming unit weights
        assert_eq!(shortest_paths["C"], 1);
    }
}
