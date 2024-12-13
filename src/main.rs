use std::collections::{HashMap, VecDeque};
use csv::Reader;

// Graph Representation
type Vertex = usize;
type Distance = usize;
type Edge = (Vertex, Vertex, Distance);

#[derive(Debug, Copy, Clone)]
struct Outedge {
    vertex: Vertex,
    length: Distance,
}

type AdjacencyList = Vec<Outedge>;

#[derive(Debug)]
struct Graph {
    n: usize,
    outedges: Vec<AdjacencyList>,
}

impl Graph {
    fn create_directed(n: usize, edges: &[Edge]) -> Graph {
        let mut outedges = vec![vec![]; n];
        for (u, v, length) in edges {
            outedges[*u].push(Outedge {
                vertex: *v,
                length: *length,
            });
        }
        Graph { n, outedges }
    }

    fn degree_centrality(&self) -> HashMap<Vertex, usize> {
        self.outedges
            .iter()
            .enumerate()
            .map(|(u, edges)| (u, edges.len()))
            .collect()
    }

    fn closeness_centrality(&self) -> HashMap<Vertex, f64> {
        let mut centrality = HashMap::new();
    
        for node in 0..self.n {
            let shortest_paths = self.shortest_paths(node);
    
            let reachable_paths: Vec<_> = shortest_paths
                .values()
                .filter(|&&d| d < u64::MAX) // Ignore unreachable nodes
                .cloned()
                .collect();
    
            let total_distance: u64 = reachable_paths.iter().sum();
            let reachable_count = reachable_paths.len();
    
            if total_distance > 0 && reachable_count > 1 {
                let normalization_factor = (reachable_count - 1) as f64; // Exclude the starting node
                let closeness = normalization_factor / total_distance as f64;
                centrality.insert(node, closeness);
            } else {
                centrality.insert(node, 0.0); // Isolated or unreachable nodes
            }
        }
    
        centrality
    }
    
    fn shortest_paths(&self, start: Vertex) -> HashMap<Vertex, u64> {
        let mut distances: HashMap<Vertex, u64> = (0..self.n).map(|v| (v, u64::MAX)).collect();
        let mut queue = VecDeque::new();
    
        distances.insert(start, 0);
        queue.push_back(start);
    
        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
    
            for edge in &self.outedges[current] {
                if distances[&edge.vertex] == u64::MAX {
                    distances.insert(edge.vertex, current_distance + edge.length as u64);
                    queue.push_back(edge.vertex);
                }
            }
        }
    
        distances
    }    
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "International_Report_Departures.csv";
    let mut rdr = Reader::from_path(file_path)?;

    // Map airports to vertices
    let mut airport_map: HashMap<String, Vertex> = HashMap::new();
    let mut reverse_airport_map: HashMap<Vertex, String> = HashMap::new(); // For reverse lookup
    let mut edges: Vec<Edge> = vec![];
    let mut vertex_count = 0;

    // Print the first five rows of the CSV
    println!("=== First 5 Rows of the CSV ===");
    for (i, result) in rdr.records().enumerate() {
        let record = result?;
        println!("{:?}", record);

        // Break after printing the first 5 rows
        if i == 4 {
            break;
        }
    }

    // Reinitialize the reader to parse the data again (CSV reader does not allow rewinding)
    let mut rdr = Reader::from_path(file_path)?;

    for result in rdr.records() {
        let record = result?;

        // Extract fields using the correct column names
        let origin = record.get(4).unwrap().to_string(); // 'usg_apt'
        let destination = record.get(7).unwrap().to_string(); // 'fg_apt'
        let flights: usize = record.get(15).unwrap().parse().unwrap(); // 'Total'

        // Map airports to vertex indices
        let origin_index = *airport_map.entry(origin.clone()).or_insert_with(|| {
            let idx = vertex_count;
            reverse_airport_map.insert(idx, origin.clone());
            vertex_count += 1;
            idx
        });

        let destination_index = *airport_map.entry(destination.clone()).or_insert_with(|| {
            let idx = vertex_count;
            reverse_airport_map.insert(idx, destination.clone());
            vertex_count += 1;
            idx
        });

        // Add edge with the correct flight count
        edges.push((origin_index, destination_index, flights));
    }

    // Create the directed graph
    let graph = Graph::create_directed(vertex_count, &edges);

    // Compute centralities
    let degree_centrality = graph.degree_centrality();
    let closeness_centrality = graph.closeness_centrality();

    // Display results
    println!("\n=== Centrality Metrics for Airports ===\n");

    // Top 10 busiest airports by degree centrality
    let mut degree_sorted: Vec<_> = degree_centrality.iter().collect();
    degree_sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!("Top 10 Busiest Airports (by Degree):");
    for (i, (vertex, degree)) in degree_sorted.iter().take(10).enumerate() {
        let airport = reverse_airport_map.get(vertex).unwrap();
        println!("{}. {} - {} connections", i + 1, airport, degree);
    }

    // Top 10 airports by closeness centrality
    let mut closeness_sorted: Vec<_> = closeness_centrality.iter().collect();
    closeness_sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    println!("\nTop 10 Airports by Closeness Centrality:");
    for (i, (vertex, closeness)) in closeness_sorted.iter().take(10).enumerate() {
        let airport = reverse_airport_map.get(vertex).unwrap();
        println!("{}. {} - Closeness Centrality: {:.5}", i + 1, airport, closeness);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_directed() {
        let edges = vec![
            (0, 1, 10),
            (1, 2, 20),
            (2, 0, 30),
            (2, 3, 40),
        ];
        let graph = Graph::create_directed(4, &edges);

        assert_eq!(graph.n, 4);
        assert_eq!(graph.outedges[0].len(), 1);
        assert_eq!(graph.outedges[1].len(), 1);
        assert_eq!(graph.outedges[2].len(), 2);
        assert_eq!(graph.outedges[3].len(), 0);
    }

    #[test]
    fn test_degree_centrality() {
        let edges = vec![
            (0, 1, 1),
            (1, 2, 1),
            (2, 0, 1),
            (2, 3, 1),
        ];
        let graph = Graph::create_directed(4, &edges);

        let degree = graph.degree_centrality();
        assert_eq!(degree[&0], 1);
        assert_eq!(degree[&1], 1);
        assert_eq!(degree[&2], 2);
        assert_eq!(degree[&3], 0);
    }

    #[test]
    fn test_closeness_centrality() {
        let edges = vec![
            (0, 1, 1),
            (1, 2, 1),
            (2, 0, 1),
        ];
        let graph = Graph::create_directed(3, &edges);

        let closeness = graph.closeness_centrality();
        assert!((closeness[&0] - 0.6667).abs() < 1e-4); // Node 0
        assert!((closeness[&1] - 0.6667).abs() < 1e-4); // Node 1
        assert_eq!(closeness[&2], 0.0); // Node 2
    }

    #[test]
    fn test_shortest_paths() {
        let edges = vec![
            (0, 1, 1),
            (1, 2, 1),
            (0, 2, 2),
        ];
        let graph = Graph::create_directed(3, &edges);

        let paths = graph.shortest_paths(0);
        assert_eq!(paths[&0], 0); 
        assert_eq!(paths[&1], 1); 
        assert_eq!(paths[&2], 2); 
    }
}

