use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NumEdges {
    None,
    One,
    Two
}

impl NumEdges {
    fn increment(&mut self) {
        *self = match *self {
            NumEdges::None => NumEdges::One,
            NumEdges::One => NumEdges::Two,
            NumEdges::Two => unreachable!("incrementing past 2!"),
        };
    }

    fn decrement(&mut self) {
        *self = match *self {
            NumEdges::None => unreachable!("decrementing past 0!"),
            NumEdges::One => NumEdges::None,
            NumEdges::Two => NumEdges::One,
        };
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Node {
    n: u8,
    pos: (usize, usize)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Edge {
    V {
        x: usize,
        y_range: (usize, usize)
    },
    H {
        x_range: (usize, usize),
        y: usize,
    }
}

impl Edge {
    fn interval_intersects(a: (usize, usize), b: (usize, usize)) -> bool {
        Self::value_in_interval(a.0, b) || Self::value_in_interval(a.1, b)
    }

    fn value_in_interval(v: usize, interval: (usize, usize)) -> bool {
        assert!(interval.0 < interval.1);
        v > interval.0 && v < interval.1
    }

    fn intersects(self, other: Edge) -> bool {
        match (self, other) {
            (Edge::V { x, y_range }, Edge::V { x: x2, y_range: y_range2 }) => {
                x == x2 && Self::interval_intersects(y_range, y_range2)
            }
            (Edge::H { y, x_range }, Edge::H { y: y2, x_range: x_range2 }) => {
                y == y2 && Self::interval_intersects(x_range, x_range2)
            }
            (Edge::H { y, x_range }, Edge::V { x, y_range}) |
            (Edge::V { x, y_range }, Edge::H { y, x_range}) => {
                Self::value_in_interval(x, x_range) && Self::value_in_interval(y, y_range)
            }
        }
    }

    fn endpoints(self) -> ((usize, usize), (usize, usize)) {
        match self {
            Edge::H { y, x_range } => ((x_range.0, y), (x_range.1, y)),
            Edge::V { x, y_range } => ((x, y_range.0), (x, y_range.1)),
        }
    }

    fn points(self) -> Vec<(usize, usize)> {
        match self {
            Edge::H { y, x_range } => (x_range.0..=x_range.1).map(|x| (x, y)).collect(),
            Edge::V { x, y_range } => (y_range.0..=y_range.1).map(|y| (x, y)).collect(),
        }
    }

    fn as_char(self, num_edges: NumEdges) -> char {
        match (self, num_edges) {
            (Edge::H { .. }, NumEdges::None) |
            (Edge::V { .. }, NumEdges::None) => ' ',
            (Edge::H { .. }, NumEdges::One) => '-',
            (Edge::V { .. }, NumEdges::One) => '|',
            (Edge::H { .. }, NumEdges::Two) => '=',
            (Edge::V { .. }, NumEdges::Two) => '‖',

        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    edge_intersections: HashMap<usize, Vec<usize>>,
}

impl Board {
    pub fn parse(s: &str) -> Self {
        let mut nodes = vec![];
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if let Some(n) = c.to_digit(10) {
                    nodes.push(Node {
                        n: n as u8,
                        pos: (x, y)
                    });
                }
            }
        }
        Self::new(nodes)
    }

    pub fn new(mut nodes: Vec<Node>) -> Self {
        let mut edges = vec![];

        // compute horizontal lines
        nodes.sort_by_key(|n| n.pos.0);

        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                if nodes[i].pos.1 == nodes[j].pos.1 && (nodes[j].pos.0 - nodes[i].pos.0) > 1 {
                    edges.push(Edge::H {
                        y: nodes[i].pos.1,
                        x_range: (nodes[i].pos.0, nodes[j].pos.0)
                    });
                    break;
                }
            }
        }


        // compute vertical lines
        nodes.sort_by_key(|n| n.pos.1);

        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                if nodes[i].pos.0 == nodes[j].pos.0 && (nodes[j].pos.1 - nodes[i].pos.1) > 1 {
                    edges.push(Edge::V {
                        x: nodes[i].pos.0,
                        y_range: (nodes[i].pos.1, nodes[j].pos.1)
                    });
                    break;
                }
            }
        }

        let mut edge_intersections = HashMap::new();

        for (idx, edge) in edges.iter().enumerate() {
            for (idx2, edge2) in edges.iter().enumerate().skip(idx) {
                if edge.intersects(*edge2) {
                    edge_intersections.entry(idx).or_insert_with(Vec::new).push(idx2);
                    edge_intersections.entry(idx2).or_insert_with(Vec::new).push(idx);
                }
            }
        }

        Self {
            nodes,
            edges,
            edge_intersections
        }
    }


    pub fn serialize(&self, soln: impl IntoIterator<Item = usize>, io: &'_ mut impl std::io::Write) -> std::io::Result<()> {
        let mut aggregated = HashMap::new();
        for idx in soln {
            aggregated.entry(idx).or_insert(NumEdges::None).increment();
        }

        fmt_viz(&self.nodes, &self.edges, |idx| aggregated.get(&idx).copied().unwrap_or(NumEdges::None), io)
    }

    pub fn serialize_to_string(&self, soln: impl IntoIterator<Item = usize>) -> String {
        let mut s = vec![];
        self.serialize(soln, &mut s).unwrap();
        String::from_utf8(s).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct SolveState<'b> {
    soln: Vec<usize>,
    edge_counts: Vec<NumEdges>,
    node_counts: Vec<u8>,
    nodes_by_position: HashMap<(usize, usize), usize>,
    edges_adjacent_to_node: HashMap<usize, Vec<usize>>,

    // Note: this could be made a lot more efficient, but it works fine for now.
    visited: HashSet<Vec<NumEdges>>,
    board: &'b Board,
}

impl<'b> SolveState<'b> {
    pub fn new(board: &'b Board) -> SolveState<'b> {
        let mut nodes_by_position = HashMap::new();
        let mut edges_adjacent_to_node = HashMap::new();

        for (idx, n) in board.nodes.iter().enumerate() {
            nodes_by_position.insert(n.pos, idx);
        }

        for (idx, edge) in board.edges.iter().enumerate() {
            let (p1, p2) = edge.endpoints();
            edges_adjacent_to_node.entry(nodes_by_position[&p1]).or_insert_with(Vec::new).push(idx);
            edges_adjacent_to_node.entry(nodes_by_position[&p2]).or_insert_with(Vec::new).push(idx);
        }

        Self {
            soln: vec![],
            edge_counts: vec![NumEdges::None; board.edges.len()],
            node_counts: vec![0; board.nodes.len()],
            visited: HashSet::new(),
            edges_adjacent_to_node,
            nodes_by_position,
            board,
        }
    }

    pub fn add_edge(&mut self, edge: usize) {
        self.soln.push(edge);
        self.edge_counts[edge].increment();

        let (p1, p2) = self.board.edges[edge].endpoints();
        let n1 = self.nodes_by_position[&p1];
        let n2 = self.nodes_by_position[&p2];
        self.node_counts[n1] += 1;
        self.node_counts[n2] += 1;
    }

    fn remove_edge(&mut self, edge: usize) {
        let idx = self.soln.iter().rposition(|v| *v == edge).unwrap();
        self.soln.remove(idx);
        self.edge_counts[edge].decrement();


        let (p1, p2) = self.board.edges[edge].endpoints();
        let n1 = self.nodes_by_position[&p1];
        let n2 = self.nodes_by_position[&p2];
        self.node_counts[n1] -= 1;
        self.node_counts[n2] -= 1;
    }

    fn available_edges_for_node(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.edges_adjacent_to_node[&node].iter().filter(|edge_idx| {
            let (p1, p2) = self.board.edges[**edge_idx].endpoints();

            if self.edge_counts[**edge_idx] != NumEdges::Two {
                let mut is_viable = true;
                if let Some(intersecting_edges) = self.board.edge_intersections.get(*edge_idx) {
                    for intersecting_edge_idx in intersecting_edges {
                        if self.edge_counts[*intersecting_edge_idx] != NumEdges::None {
                            is_viable = false;
                        }
                    }
                }

                let n1 = self.nodes_by_position[&p1];
                let n2 = self.nodes_by_position[&p2];
                if self.node_counts[n1] > self.board.nodes[n1].n - 1 {
                    is_viable = false;
                }
                if self.node_counts[n2] > self.board.nodes[n2].n - 1 {
                    is_viable = false;
                }
                is_viable
            } else {
                false
            }
        }).copied()
    }

    fn find_next_edges(&self) -> Vec<usize> {
        let mut viable = vec![];
        let mut viable_set = HashSet::new();

        for idx in 0..self.board.nodes.len() {
            if self.node_counts[idx] == self.board.nodes[idx].n {
                continue
            }
            for edge_idx in self.available_edges_for_node(idx) {
                if !viable_set.contains(&edge_idx) {
                    viable.push(edge_idx);
                    viable_set.insert(edge_idx);
                }
            }
        }

        viable
    }

    fn solved(&self) -> bool {
        // Check completion
        for idx in 0..self.board.nodes.len() {
            if self.node_counts[idx] != self.board.nodes[idx].n {
                return false;
            }
        }


        // Check connectivity via disjoint-set algorithm
        let mut node_disjoint_set = (0..self.board.nodes.len()).collect::<Vec<_>>();

        for (edge, edge_count) in self.edge_counts.iter().enumerate() {
            if *edge_count == NumEdges::None {
                continue;
            }

            let (p1, p2) = self.board.edges[edge].endpoints();
            let n1 = self.nodes_by_position[&p1];
            let n2 = self.nodes_by_position[&p2];

            // Set both node's disjoint-set pointer the the lower of the two, now that they are
            // connected.
            let djs1 = node_disjoint_set[n1];
            let djs2 = node_disjoint_set[n2];

            let min = djs1.min(djs2);
            let max = djs1.max(djs2);
            if min != max {
                for v in &mut node_disjoint_set {
                    if *v == max {
                        *v = min
                    }
                }
            }
        }

        node_disjoint_set.iter().all(|v| *v == 0)
    }

    fn solve_fully_constrained(&self) -> Option<usize> {
        // Attempt to find any fully-constrained nodes.
        for idx in 0..self.board.nodes.len() {
            if self.node_counts[idx] == self.board.nodes[idx].n {
                continue
            }

            let count = self.available_edges_for_node(idx).count();

            let v = match self.board.nodes[idx].n - self.node_counts[idx] {
                1 | 2 if count == 1 => true,
                3 | 4 if count == 2 => true,
                5 | 6 if count == 3 => true,
                7 | 8 => true,
                _ => false,
            };

            if v {
                for edge in self.available_edges_for_node(idx) {
                    return Some(edge);
                }
            }
        }
        None
    }

    pub fn solve(&mut self) -> Result<Vec<usize>, ()> {
        if self.solved() {
            return Ok(self.soln.clone());
        }

        let v = self.edge_counts.clone();
        if self.visited.contains(&v) {
            return Err(());
        }
        self.visited.insert(v);

        if let Some(idx) = self.solve_fully_constrained() {
            self.add_edge(idx);
            let ret = self.solve();
            self.remove_edge(idx);
            if ret.is_ok() {
                return ret;
            }
        }

        for idx in self.find_next_edges() {
            self.add_edge(idx);
            let ret = self.solve();
            self.remove_edge(idx);
            if ret.is_ok() {
                return ret;
            }
        }

        Err(())
    }
}

fn fmt_viz(nodes: &[Node], edges: &[Edge], edge_counts: impl Fn(usize) -> NumEdges, io: &'_ mut impl std::io::Write) -> std::io::Result<()> {
    // compute the bounds
    let max_x = nodes.iter().map(|n| n.pos.0).max().unwrap_or(0) + 1;
    let max_y = nodes.iter().map(|n| n.pos.1).max().unwrap_or(0) + 1;

    let mut arr = vec![vec![' '; max_y]; max_x];

    for (idx, edge) in edges.iter().enumerate() {
        for (x, y) in edge.points() {
            let ct = edge_counts(idx);
            if ct != NumEdges::None {
                let c = edge.as_char(ct);
                if arr[x][y] == ' ' || arr[x][y] == c {
                    arr[x][y] = c;
                } else {
                    arr[x][y] = '+';
                }
            }
        }
    }

    for node in nodes {
        arr[node.pos.0][node.pos.1] = node.n.to_string().chars().next().unwrap();
    }

    for y in 0..max_y {
        if !(0..max_x).all(|x| arr[x][y] == ' ') {
            for x in 0..max_x {
                write!(io, "{}", arr[x][y])?;
            }
        }
        writeln!(io)?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    const EASY_7X7: &'static str = r#"
 2    4
3  4 3 
        
 1 2  3
4    3
       
3  3  3
"#;
    const EASY_7X7_SOLN: &'static str = r#"
 2====4
3==4-3‖
|  | ‖‖
|1-2 ‖3
4----3|
‖     |
3--3==3
"#;

    const HARD_25X25: &'static str = r#"
3 4             5 2 1  1 
    3       2           1
     2 3        6   4  4 
                  3   3 3
2  1  3        2 2 1     
                  1      
                 5 4 1   
1                   2 4  
                         
                       4 
3                        
                   2 1   
                 6    5  
                  2  2   
3                        
                  5  5 4 
    2 4         5        
                 3       
   2            3    1 2 
                 1      
5 5               6   7 6
   2       4             
4      4  1              
                         
2 1 1  5   5      4   2 2
"#;
    
    const HARD_25X25_SOLN: &'static str = r#"
3-4-------------5=2 1  1 
‖ ‖ 3=======2   ‖   |  |1
‖ ‖ |2=3--------6===4--4|
‖ ‖ |           | 3===3‖3
2 ‖1| 3========2|2|1  |‖‖
  ‖|| |         |‖1|  |‖‖
  ‖|| |         |5-4-1|‖‖
1 ‖|| |         |‖ |2=4‖‖
| ‖|| |         |‖ |  |‖‖
| ‖|| |         |‖ |  |4‖
3 ‖|| |         |‖ |  |‖‖
‖ ‖|| |         |‖ 2-1|‖‖
‖ ‖|| |         |6====5‖‖
‖ ‖|| |         |‖2  2‖‖‖
3 ‖|| |         |‖‖  ‖‖‖‖
| ‖|| |         |‖5==5‖4‖
| ‖|2-4=========5‖|  |‖‖‖
| ‖|            ‖3|  |‖‖‖
| ‖2------------3||  1‖2‖
| ‖              1|   ‖ ‖
5=5---------------6===7=6
‖  2=======4      ‖   | ‖
4------4--1‖      ‖   | ‖
|      ‖   ‖      ‖   | ‖
2-1 1--5===5------4---2 2
"#;

    #[test]
    fn test_easy_7x7() {
        let b = Board::parse(EASY_7X7);
        let soln = SolveState::new(&b).solve().unwrap();

        for i in 0..soln.len() {
            eprintln!("{}", b.serialize_to_string(soln.iter().copied().take(i)));
            eprintln!();
        }

        assert_eq!(b.serialize_to_string(soln.iter().copied()), EASY_7X7_SOLN);
    }

    #[test]
    fn test_hard_25x25() {
        let b = Board::parse(HARD_25X25);
        let soln = SolveState::new(&b).solve().unwrap();

        for i in 0..=soln.len() {
            eprintln!("{}", b.serialize_to_string(soln.iter().copied().take(i)));
            eprintln!();
        }

        assert_eq!(b.serialize_to_string(soln.iter().copied()), HARD_25X25_SOLN);
    }

    #[test]
    fn test_edge_intersections() {
        // parallel intersections
        assert!(Edge::V { x: 0, y_range: (0, 3) }.intersects(Edge::V { x: 0, y_range: (2, 4) }));
        assert!(!Edge::V { x: 0, y_range: (0, 2) }.intersects(Edge::V { x: 0, y_range: (2, 4) }));
        assert!(!Edge::V { x: 0, y_range: (2, 5) }.intersects(Edge::V { x: 0, y_range: (2, 4) }));
        assert!(Edge::H { y: 0, x_range: (0, 3) }.intersects(Edge::H { y: 0, x_range: (2, 4) }));
        assert!(!Edge::H { y: 0, x_range: (0, 2) }.intersects(Edge::H { y: 0, x_range: (2, 4) }));
        assert!(!Edge::H { y: 0, x_range: (2, 5) }.intersects(Edge::H { y: 0, x_range: (2, 4) }));

        // perpendicular intersections
        assert!(Edge::V { x: 1, y_range: (0, 2) }.intersects(Edge::H { y: 1, x_range: (0, 2) }));
        assert!(!Edge::V { x: 2, y_range: (0, 2) }.intersects(Edge::H { y: 1, x_range: (0, 2) }));
        assert!(!Edge::V { x: 1, y_range: (0, 2) }.intersects(Edge::H { y: 2, x_range: (0, 2) }));
    }
}

