use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use csv::Reader;

#[derive(Debug)]
struct FlightData {
    year: u32,
    month: u32,
    us_airport: String,
    foreign_airport: String,
    carrier: String,
    flight_type: String,
    total_flights: u32,
}

fn read_csv(file_path: &str) -> Result<Vec<FlightData>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;
    let mut flights = Vec::new();

    for result in rdr.records() {
        let record = result?;
        flights.push(FlightData {
            year: record[1].parse()?,
            month: record[2].parse()?,
            us_airport: record[4].to_string(),
            foreign_airport: record[7].to_string(),
            carrier: record[10].to_string(),
            flight_type: record[12].to_string(),
            total_flights: record[15].parse()?,
        });
    }

    Ok(flights)
}

#[derive(Debug)]
struct Graph {
    adjacency_list: HashMap<String, Vec<(String, u32)>>, // Node -> [(Neighbor, Weight)]
}

impl Graph {
    fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: &str, to: &str, weight: u32) {
        self.adjacency_list
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push((to.to_string(), weight));

        self.adjacency_list
            .entry(to.to_string())
            .or_insert_with(Vec::new)
            .push((from.to_string(), weight));
    }

    fn bfs_shortest_paths(&self, start: &str) -> HashMap<String, u32> {
        let mut distances: HashMap<String, u32> = HashMap::new();
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        queue.push_back((start.to_string(), 0));

        while let Some((current, distance)) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }

            distances.insert(current.clone(), distance);

            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for (neighbor, _) in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back((neighbor.clone(), distance + 1));
                    }
                }
            }
        }

        distances
    }

    fn connected_components(&self) -> Vec<HashSet<String>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node in self.adjacency_list.keys() {
            if !visited.contains(node) {
                let mut component = HashSet::new();
                let mut stack = vec![node.clone()];
                while let Some(current) = stack.pop() {
                    if visited.insert(current.clone()) {
                        component.insert(current.clone());
                        if let Some(neighbors) = self.adjacency_list.get(&current) {
                            for (neighbor, _) in neighbors {
                                if !visited.contains(neighbor) {
                                    stack.push(neighbor.clone());
                                }
                            }
                        }
                    }
                }
                components.push(component);
            }
        }

        components
    }

    fn largest_component(&self) -> HashSet<String> {
        self.connected_components()
            .into_iter()
            .max_by_key(|component| component.len())
            .unwrap_or_default()
    }

    fn harmonic_centrality(&self) -> Vec<(String, f64)> {
        let mut centrality_scores = Vec::new();

        for node in self.adjacency_list.keys() {
            let distances = self.bfs_shortest_paths(node);
            let harmonic_sum: f64 = distances
                .values()
                .map(|&d| if d > 0 { 1.0 / d as f64 } else { 0.0 })
                .sum();

            centrality_scores.push((node.clone(), harmonic_sum));
        }

        centrality_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        centrality_scores.into_iter().take(5).collect()
    }

      
    
}

fn build_graph(flights: &[FlightData]) -> Graph {
    let mut graph = Graph::new();

    for flight in flights {
        graph.add_edge(
            &flight.us_airport,
            &flight.foreign_airport,
            flight.total_flights,
        );
    }

    graph
}

fn top_busiest_airports(flights: &[FlightData]) -> Vec<(String, u32)> {
    let mut airport_totals: HashMap<String, u32> = HashMap::new();

    for flight in flights {
        *airport_totals
            .entry(flight.us_airport.clone())
            .or_insert(0) += flight.total_flights;
    }

    let mut totals: Vec<_> = airport_totals.into_iter().collect();
    totals.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by total flights in descending order

    totals.into_iter().take(5).collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "International_Report_Departures.csv";
    let flights = read_csv(file_path)?;

    println!("Loaded {} records.", flights.len());

    let busiest_airports = top_busiest_airports(&flights);
    println!("\nTop 5 Busiest Airports:");
    for (airport, total) in busiest_airports {
        println!("{}: {} flights", airport, total);
    }

    let graph = build_graph(&flights);

    let node_count = graph.adjacency_list.len();
    let edge_count: usize = graph
        .adjacency_list
        .values()
        .map(|neighbors| neighbors.len())
        .sum::<usize>() / 2; // Divide by 2 for undirected edges
    println!("\nGraph Statistics:");
    println!("Nodes: {}", node_count);
    println!("Edges: {}", edge_count);

    let components = graph.connected_components();
    println!("\nNumber of connected components: {}", components.len());
    for (i, component) in components.iter().enumerate() {
        println!("Component {}: {} nodes", i + 1, component.len());
    }

    let largest_component = graph.largest_component();
    println!("\nLargest component size: {}", largest_component.len());

    let harmonic_centralities = graph
        .harmonic_centrality()
        .into_iter()
        .filter(|(node, _)| largest_component.contains(node))
        .collect::<Vec<_>>();
    println!("\nTop 5 Airports by Harmonic Centrality (Largest Component):");
    for (airport, centrality) in harmonic_centralities {
        println!("{}: {:.4}", airport, centrality);
    }


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_add_edge() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 10);
        graph.add_edge("B", "C", 5);

        assert_eq!(graph.adjacency_list.len(), 3); // Nodes: A, B, C
        assert_eq!(graph.adjacency_list["A"].len(), 1); // A -> B
        assert_eq!(graph.adjacency_list["B"].len(), 2); // B -> A, B -> C
        assert_eq!(graph.adjacency_list["C"].len(), 1); // C -> B
    }

    #[test]
    fn test_graph_bfs_shortest_paths() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 1);
        graph.add_edge("B", "C", 1);
        graph.add_edge("A", "C", 2); // Edge weight ignored in BFS (unweighted graph)

        let distances = graph.bfs_shortest_paths("A");

        assert_eq!(distances["A"], 0); // Distance to self
        assert_eq!(distances["B"], 1); // Distance from A -> B
        assert_eq!(distances["C"], 1); // Distance from A -> C (direct edge)
    }

    #[test]
    fn test_connected_components() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 1);
        graph.add_edge("B", "C", 1);
        graph.add_edge("D", "E", 1); // Separate component

        let components = graph.connected_components();

        assert_eq!(components.len(), 2); // Two connected components
        assert!(components.iter().any(|c| c.contains("A") && c.contains("B") && c.contains("C"))); // Component 1
        assert!(components.iter().any(|c| c.contains("D") && c.contains("E"))); // Component 2
    }

    #[test]
    fn test_largest_component() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 1);
        graph.add_edge("B", "C", 1);
        graph.add_edge("D", "E", 1); // Separate smaller component

        let largest_component = graph.largest_component();

        assert_eq!(largest_component.len(), 3); // Largest component size
        assert!(largest_component.contains("A"));
        assert!(largest_component.contains("B"));
        assert!(largest_component.contains("C"));
    }

    #[test]
    fn test_harmonic_centrality() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 1);
        graph.add_edge("B", "C", 1);
        graph.add_edge("C", "D", 1);

        let harmonic_centralities = graph.harmonic_centrality();

        // Validate the top node by centrality
        assert_eq!(harmonic_centralities[0].0, "B"); // Node B is central
        assert!(harmonic_centralities[0].1 > 0.0); // Centrality score > 0
    }

    #[test]
    fn test_busiest_routes() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B", 100);
        graph.add_edge("B", "C", 200);
        graph.add_edge("C", "D", 50);

        let busiest_routes = graph.busiest_routes();

        // Check the top route
        assert_eq!(busiest_routes[0].0, ("B".to_string(), "C".to_string())); // Top route
        assert_eq!(busiest_routes[0].1, 200); // Flight count
    }
}
