//! Hamiltonian paths on an arbitrary graph: given as an adjacency list rather than a grid, find
//! every path starting at a fixed vertex that visits every other vertex exactly once.
//!
//! Unlike `knights_journey` (whose moves are generated implicitly from a chessboard position),
//! this `Problem` reads its moves straight out of an adjacency list, showing that `Solutions`
//! doesn't care whether the underlying search space is a grid or a general graph.
//!
//! The bundled graph is the Petersen graph: a well-known example that has Hamiltonian paths but
//! no Hamiltonian cycle (it's "traceable" but not "Hamiltonian").
use generic_backtracking::{Problem, Solutions};

fn main() {
    let adjacency = petersen_graph();
    let start = 0;

    let problem = HamiltonianPath::new(adjacency.clone(), start);
    let paths: Vec<_> = Solutions::new(problem).collect();
    println!(
        "found {} Hamiltonian path(s) starting at vertex {start} in the Petersen graph",
        paths.len()
    );
    if let Some(first) = paths.first() {
        let rendered: Vec<String> = first.iter().map(usize::to_string).collect();
        println!("example: {}", rendered.join(" -> "));
    }

    // Fun fact: the Petersen graph is famously non-Hamiltonian, so none of the paths we find can
    // be closed back up into a cycle.
    let cycles = paths
        .iter()
        .filter(|path| adjacency[*path.last().unwrap()].contains(&start))
        .count();
    println!("{cycles} of those paths close into a Hamiltonian cycle (expected: 0)");
}

/// The Petersen graph: an outer 5-cycle (vertices `0..5`), an inner 5-pointed star (vertices
/// `5..10`), and a spoke connecting each outer vertex to its corresponding inner one.
fn petersen_graph() -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); 10];
    let mut connect = |a: usize, b: usize| {
        adjacency[a].push(b);
        adjacency[b].push(a);
    };

    for i in 0..5 {
        connect(i, (i + 1) % 5); // outer cycle
        connect(5 + i, 5 + (i + 2) % 5); // inner pentagram
        connect(i, 5 + i); // spoke
    }
    adjacency
}

#[derive(Clone)]
struct HamiltonianPath {
    adjacency: Vec<Vec<usize>>,
    visited: Vec<bool>,
    start: usize,
}

impl HamiltonianPath {
    fn new(adjacency: Vec<Vec<usize>>, start: usize) -> Self {
        let visited = vec![false; adjacency.len()];
        Self {
            adjacency,
            visited,
            start,
        }
    }
}

impl Problem for HamiltonianPath {
    type Possibility = usize;
    type Solution = Vec<usize>;

    fn extend_possibilities(&self, possibilities: &mut Vec<usize>, history: &[usize]) {
        if history.len() == self.adjacency.len() {
            return;
        }
        match history.last() {
            None => possibilities.push(self.start),
            Some(&last) => possibilities.extend(
                self.adjacency[last]
                    .iter()
                    .copied()
                    .filter(|&vertex| !self.visited[vertex]),
            ),
        }
    }

    fn what_if(&mut self, decision: usize) {
        self.visited[decision] = true;
    }

    fn undo(&mut self, last: &usize, _history: &[usize]) {
        self.visited[*last] = false;
    }

    fn is_solution(&self, history: &[usize]) -> Option<Vec<usize>> {
        (history.len() == self.adjacency.len()).then(|| history.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn petersen_graph_is_10_vertices_3_regular() {
        let adjacency = petersen_graph();

        assert_eq!(10, adjacency.len());
        assert!(adjacency.iter().all(|neighbors| neighbors.len() == 3));
    }

    /// Pins `petersen_graph` to the actual Petersen graph rather than merely "some 3-regular graph
    /// on 10 vertices". Petersen is the unique strongly regular graph srg(10, 3, 0, 1): adjacent
    /// vertices share no common neighbour, non-adjacent vertices share exactly one. Worth checking
    /// explicitly, because wiring the inner ring as `(i + 1) % 5` instead of `(i + 2) % 5` is an
    /// easy typo that builds the pentagonal prism — also 10 vertices and 3-regular, but Hamiltonian,
    /// so it would invalidate what this example claims to demonstrate.
    #[test]
    fn petersen_graph_is_the_unique_strongly_regular_graph_srg_10_3_0_1() {
        let adjacency = petersen_graph();

        assert_eq!(10, adjacency.len());
        assert!(adjacency.iter().all(|neighbors| neighbors.len() == 3));
        for u in 0..adjacency.len() {
            for v in (u + 1)..adjacency.len() {
                let common = (0..adjacency.len())
                    .filter(|w| adjacency[u].contains(w) && adjacency[v].contains(w))
                    .count();
                let expected = usize::from(!adjacency[u].contains(&v));
                assert_eq!(expected, common, "common neighbours of {u} and {v}");
            }
        }
    }

    #[test]
    fn petersen_graph_adjacency_is_symmetric() {
        let adjacency = petersen_graph();

        for (vertex, neighbors) in adjacency.iter().enumerate() {
            for &neighbor in neighbors {
                assert!(
                    adjacency[neighbor].contains(&vertex),
                    "{neighbor} should list {vertex} back as a neighbor"
                );
            }
        }
    }

    #[test]
    fn extend_possibilities_starts_with_only_the_start_vertex() {
        let problem = HamiltonianPath::new(petersen_graph(), 3);

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &[]);

        assert_eq!(vec![3], possibilities);
    }

    #[test]
    fn extend_possibilities_excludes_visited_neighbors() {
        let mut problem = HamiltonianPath::new(petersen_graph(), 0);
        problem.what_if(0);
        problem.what_if(1);

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &[0, 1]);

        // Vertex 1's neighbors are 0, 2, 6; 0 is already visited.
        possibilities.sort();
        assert_eq!(vec![2, 6], possibilities);
    }

    /// `Solutions` relies on `what_if`/`undo` being exact inverses when it rewinds between sibling
    /// branches, so check that directly rather than only through a full search.
    #[test]
    fn undo_reverses_what_if() {
        let mut problem = HamiltonianPath::new(petersen_graph(), 0);
        let unvisited = problem.visited.clone();

        problem.what_if(0);
        problem.what_if(5);
        assert!(problem.visited[0] && problem.visited[5]);

        problem.undo(&5, &[0]);
        problem.undo(&0, &[]);

        assert_eq!(unvisited, problem.visited);
    }

    #[test]
    fn extend_possibilities_empty_once_every_vertex_is_visited() {
        let problem = HamiltonianPath::new(petersen_graph(), 0);
        let history: Vec<usize> = (0..10).collect();

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &history);

        assert!(possibilities.is_empty());
    }

    #[test]
    fn is_solution_only_once_every_vertex_is_visited() {
        let problem = HamiltonianPath::new(petersen_graph(), 0);

        assert_eq!(None, problem.is_solution(&[0, 1, 2]));
        let full: Vec<usize> = (0..10).collect();
        assert_eq!(Some(full.clone()), problem.is_solution(&full));
    }

    #[test]
    fn petersen_graph_has_hamiltonian_paths_but_no_hamiltonian_cycle() {
        let adjacency = petersen_graph();
        let problem = HamiltonianPath::new(adjacency.clone(), 0);

        let paths: Vec<_> = Solutions::new(problem).collect();

        // The Petersen graph has 120 Hamiltonian paths; each is found from both of its endpoints,
        // and by vertex transitivity those 240 endpoints spread evenly over the 10 vertices.
        assert_eq!(24, paths.len());
        // Every path is a genuine permutation of the vertices walking real edges.
        for path in &paths {
            let mut visited = path.clone();
            visited.sort();
            assert_eq!((0..10).collect::<Vec<usize>>(), visited);
            assert!(path.windows(2).all(|w| adjacency[w[0]].contains(&w[1])));
        }
        // Non-Hamiltonicity: no path's far end links back to the start. Since the graph is vertex
        // transitive, any Hamiltonian cycle would have to pass through vertex 0, so this rules out
        // a Hamiltonian cycle anywhere in the graph, not just one through 0.
        assert!(paths
            .iter()
            .all(|path| !adjacency[*path.last().unwrap()].contains(&0)));
    }
}
