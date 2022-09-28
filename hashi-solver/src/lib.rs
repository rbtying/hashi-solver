use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NumEdges {
    None,
    One,
    Two,
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
    pos: (usize, usize),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Edge {
    V { x: usize, y_range: (usize, usize) },
    H { x_range: (usize, usize), y: usize },
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
            (
                Edge::V { x, y_range },
                Edge::V {
                    x: x2,
                    y_range: y_range2,
                },
            ) => x == x2 && Self::interval_intersects(y_range, y_range2),
            (
                Edge::H { y, x_range },
                Edge::H {
                    y: y2,
                    x_range: x_range2,
                },
            ) => y == y2 && Self::interval_intersects(x_range, x_range2),
            (Edge::H { y, x_range }, Edge::V { x, y_range })
            | (Edge::V { x, y_range }, Edge::H { y, x_range }) => {
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
            (Edge::H { .. }, NumEdges::None) | (Edge::V { .. }, NumEdges::None) => ' ',
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
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let mut nodes = vec![];
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if let Some(n) = c.to_digit(10) {
                    nodes.push(Node {
                        n: n as u8,
                        pos: (x, y),
                    });
                } else if c != ' ' {
                    return Err("unexpected character (only expected 1-8)");
                }
            }
        }
        Ok(Self::new(nodes))
    }

    pub fn new(mut nodes: Vec<Node>) -> Self {
        let mut edges = vec![];

        // compute horizontal lines
        nodes.sort_by_key(|n| n.pos.0);

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                if nodes[i].pos.1 == nodes[j].pos.1 && (nodes[j].pos.0 - nodes[i].pos.0) > 1 {
                    edges.push(Edge::H {
                        y: nodes[i].pos.1,
                        x_range: (nodes[i].pos.0, nodes[j].pos.0),
                    });
                    break;
                }
            }
        }

        // compute vertical lines
        nodes.sort_by_key(|n| n.pos.1);

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                if nodes[i].pos.0 == nodes[j].pos.0 && (nodes[j].pos.1 - nodes[i].pos.1) > 1 {
                    edges.push(Edge::V {
                        x: nodes[i].pos.0,
                        y_range: (nodes[i].pos.1, nodes[j].pos.1),
                    });
                    break;
                }
            }
        }

        let mut edge_intersections = HashMap::new();

        for (idx, edge) in edges.iter().enumerate() {
            for (idx2, edge2) in edges.iter().enumerate().skip(idx) {
                if edge.intersects(*edge2) {
                    edge_intersections
                        .entry(idx)
                        .or_insert_with(Vec::new)
                        .push(idx2);
                    edge_intersections
                        .entry(idx2)
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
            }
        }

        Self {
            nodes,
            edges,
            edge_intersections,
        }
    }

    pub fn serialize(
        &self,
        soln: impl IntoIterator<Item = usize>,
        io: &'_ mut impl std::io::Write,
    ) -> std::io::Result<()> {
        let mut aggregated = HashMap::new();
        for idx in soln {
            aggregated.entry(idx).or_insert(NumEdges::None).increment();
        }

        fmt_viz(
            &self.nodes,
            &self.edges,
            |idx| aggregated.get(&idx).copied().unwrap_or(NumEdges::None),
            io,
        )
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
    log: Vec<&'static str>,
    depth: usize,
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
            edges_adjacent_to_node
                .entry(nodes_by_position[&p1])
                .or_insert_with(Vec::new)
                .push(idx);
            edges_adjacent_to_node
                .entry(nodes_by_position[&p2])
                .or_insert_with(Vec::new)
                .push(idx);
        }

        Self {
            soln: vec![],
            log: vec![],
            edge_counts: vec![NumEdges::None; board.edges.len()],
            node_counts: vec![0; board.nodes.len()],
            visited: HashSet::new(),
            edges_adjacent_to_node,
            nodes_by_position,
            board,
            depth: 0,
        }
    }

    pub fn already_visited(&mut self, edge: usize) -> bool {
        self.edge_counts[edge].increment();
        let r = self.visited.contains(&self.edge_counts);
        self.edge_counts[edge].decrement();
        r
    }

    pub fn add_edge(&mut self, edge: usize, reason: &'static str) {
        self.soln.push(edge);
        self.log.push(reason);
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
        self.log.remove(idx);
        self.edge_counts[edge].decrement();

        let (p1, p2) = self.board.edges[edge].endpoints();
        let n1 = self.nodes_by_position[&p1];
        let n2 = self.nodes_by_position[&p2];
        self.node_counts[n1] -= 1;
        self.node_counts[n2] -= 1;
    }

    fn assigned_edges_for_node(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.edges_adjacent_to_node[&node]
            .iter()
            .filter(|edge_idx| self.edge_counts[**edge_idx] != NumEdges::None)
            .copied()
    }

    fn available_edges_for_node(&self, node: usize) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.edges_adjacent_to_node[&node]
            .iter()
            .flat_map(|edge_idx| {
                let (p1, p2) = self.board.edges[*edge_idx].endpoints();

                let unused_slots = match self.edge_counts[*edge_idx] {
                    NumEdges::Two => 0,
                    NumEdges::One => 1,
                    NumEdges::None => 2,
                };

                if unused_slots > 0 {
                    let mut is_viable = true;

                    let n1 = self.nodes_by_position[&p1];
                    let n2 = self.nodes_by_position[&p2];

                    let available = unused_slots.min(self.remaining(n1).min(self.remaining(n2)));

                    if available == 0 {
                        is_viable = false;
                    }
                    // Don't allow single-bonds from 1 to 1 or double-bounds from 2 to 2
                    if self.board.nodes[n1].n == self.board.nodes[n2].n {
                        if self.board.nodes[n1].n == 1
                            || (self.board.nodes[n2].n == 2
                                && self.edge_counts[*edge_idx] == NumEdges::One)
                        {
                            is_viable = false;
                        }
                    }

                    if is_viable {
                        if let Some(intersecting_edges) =
                            self.board.edge_intersections.get(edge_idx)
                        {
                            for intersecting_edge_idx in intersecting_edges {
                                if self.edge_counts[*intersecting_edge_idx] != NumEdges::None {
                                    is_viable = false;
                                }
                            }
                        }
                    }

                    if is_viable {
                        Some((*edge_idx, available))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    fn remaining(&self, idx: usize) -> u8 {
        self.board.nodes[idx].n - self.node_counts[idx]
    }

    fn find_next_edges(&self) -> Vec<usize> {
        let mut viable = vec![];
        let mut viable_set = HashSet::new();

        for idx in 0..self.board.nodes.len() {
            if self.remaining(idx) == 0 {
                continue;
            }
            for (edge_idx, _) in self.available_edges_for_node(idx) {
                if !viable_set.contains(&edge_idx) {
                    viable.push(edge_idx);
                    viable_set.insert(edge_idx);
                }
            }
        }

        viable
    }

    // Check if we have any fully-constrained nodes
    fn solvable(&self) -> Result<(), &'static str> {
        for idx in 0..self.board.nodes.len() {
            let is_complete = self.remaining(idx) == 0;
            let has_no_edges = self.available_edges_for_node(idx).next().is_none();
            if !is_complete && has_no_edges {
                return Err("node cannot be completed");
            }
        }

        let mut visited = vec![-1; self.board.nodes.len()];
        for idx in 0..self.board.nodes.len() {
            if visited[idx] >= 0 {
                continue;
            }

            let mut has_free_edges = false;

            let mut stk = vec![idx];
            while let Some(n) = stk.pop() {
                visited[n] = idx as isize;

                for edge in self.assigned_edges_for_node(n) {
                    let (p1, p2) = self.board.edges[edge].endpoints();
                    let n1 = self.nodes_by_position[&p1];
                    let n2 = self.nodes_by_position[&p2];

                    if n1 == n && visited[n2] < 0 {
                        stk.push(n2);
                    }
                    if n2 == n && visited[n1] < 0 {
                        stk.push(n1);
                    }
                }

                if self.available_edges_for_node(n).next().is_some() {
                    has_free_edges = true;
                }
            }

            if !has_free_edges && !visited.iter().all(|v| *v == 0) {
                return Err("isolated connected component exists");
            }
        }

        return Ok(());
    }

    fn solved(&self) -> bool {
        // Check completion
        for idx in 0..self.board.nodes.len() {
            if self.remaining(idx) != 0 {
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

    fn solve_fully_constrained(&self) -> Option<(usize, &'static str)> {
        // Attempt to find any fully-constrained nodes.
        for idx in 0..self.board.nodes.len() {
            let remaining = self.remaining(idx);
            if remaining == 0 {
                continue;
            }

            let one_slots = self
                .available_edges_for_node(idx)
                .filter(|v| v.1 == 1)
                .map(|(e, _)| e)
                .collect::<Vec<_>>();
            let two_slots = self
                .available_edges_for_node(idx)
                .filter(|v| v.1 == 2)
                .map(|(e, _)| e)
                .filter(|e| self.edge_counts[*e] == NumEdges::None)
                .collect::<Vec<_>>();

            let v = match (remaining, one_slots.len(), two_slots.len()) {
                _ if one_slots.len() + two_slots.len() > 4 => unreachable!(),
                (1, 1, 0) => Some((one_slots[0], "only viable edge")),
                (1, 0, 1) => Some((two_slots[0], "only viable edge")),
                (2, 0, 1) => Some((two_slots[0], "must include all remaining edges")),
                (2, 1, 1) => Some((two_slots[0], "must include at least one of the double-bond")),
                (2, 2, 0) => Some((one_slots[0], "must include all of the remaining edges")),
                (3, 0, 2) => Some((
                    two_slots[0],
                    "must include at least one of each double-bond",
                )),
                (3, 1, 1) => Some((two_slots[0], "must include all of the remaining edges")),
                (3, 2, 1) => Some((two_slots[0], "must include at least one of the double-bond")),
                (3, 3, 0) => Some((one_slots[0], "must include all of the remaining edges")),
                (4, 0, 2) => Some((two_slots[0], "must include all of the remaining edges")),
                (4, 1, 2) => Some((
                    two_slots[0],
                    "must include at least one of each double-bond",
                )),
                (4, 2, 1) => Some((two_slots[0], "must include all of the remaining edges")),
                (4, 3, 1) => Some((two_slots[0], "must include at least one of the double-bond")),
                (5, 0, 3) => Some((
                    two_slots[0],
                    "must include at least one of each double-bond",
                )),
                (5, 1, 2) => Some((two_slots[0], "must include all of the remaining edges")),
                (5, 2, 2) => Some((
                    two_slots[0],
                    "must include at least one of each double-bond",
                )),
                (5, 3, 1) => Some((two_slots[0], "must include all of the remaining edges")),
                (6, 0, 3) => Some((two_slots[0], "must include all of the remaining edges")),
                (6, 2, 2) => Some((two_slots[0], "must include all of the remaining edges")),
                (7, 0, 4) => Some((two_slots[0], "must include all but one of the double-bond")),
                (7, 1, 3) => Some((one_slots[0], "must include all of the remaining edges")),
                (8, 0, 4) => Some((two_slots[0], "must include all of the remaining edges")),
                _ => None,
            };
            if v.is_some() {
                return v;
            }
        }
        None
    }

    pub fn solve(
        &mut self,
        max_depth: usize,
        max_visited: usize,
    ) -> Result<(Vec<usize>, Vec<&'static str>), &'static str> {
        if self.solved() {
            return Ok((self.soln.clone(), self.log.clone()));
        }
        if self.depth > max_depth {
            return Err("max depth exceeded");
        }

        self.solvable()?;

        if let Some((idx, reason)) = self.solve_fully_constrained() {
            self.add_edge(idx, reason);
            let ret = self.solve(max_depth, max_visited);
            match ret {
                Ok(ret) => return Ok(ret),
                Err(_) => self.remove_edge(idx),
            }
        }

        self.visited.insert(self.edge_counts.clone());
        if self.visited.len() > max_visited {
            return Err("max visited state count exceeded");
        }

        for idx in self.find_next_edges() {
            if self.already_visited(idx) {
                continue;
            }

            self.add_edge(idx, "speculative");
            self.depth += 1;
            eprintln!(
                "adding speculative edge {} @ depth {}\n{}",
                idx,
                self.depth,
                self.board.serialize_to_string(self.soln.iter().copied()),
            );
            let ret = self.solve(max_depth, max_visited);
            match ret {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    self.remove_edge(idx);
                    eprintln!(
                        "removing edge {} because {}\n{}",
                        idx,
                        err,
                        self.board.serialize_to_string(self.soln.iter().copied())
                    );
                    self.depth -= 1;
                }
            }
        }

        Err("searched all options")
    }
}

fn fmt_viz(
    nodes: &[Node],
    edges: &[Edge],
    edge_counts: impl Fn(usize) -> NumEdges,
    io: &'_ mut impl std::io::Write,
) -> std::io::Result<()> {
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

    const HARD_25X25_2: &'static str = r#"
1  2          1 3    4 2 
                         
 2   1          5       3
                 2       
 4 6    2         2 4   5
                         
    4  2         4 3 3 2 
      1                  
                 2       
                         
      3 3        1       
    5      5    7  5     
                         
    1 2    4  1 1    1 1 
4  8               6    3
                     2 3 
               2 1       
                    1  4 
                         
   3         2           
                         
   1                     
5            5 5 4 4   4 
                         
3                   1 1 2
"#;

    #[test]
    fn test_easy_7x7() {
        let b = Board::parse(EASY_7X7).unwrap();
        SolveState::new(&b).solve(0, 0).unwrap();

        assert_eq!(b.serialize_to_string(soln.iter().copied()), EASY_7X7_SOLN);
    }

    #[test]
    fn test_hard_25x25() {
        let b = Board::parse(HARD_25X25).unwrap();
        SolveState::new(&b).solve(0, 0).unwrap();
        assert_eq!(b.serialize_to_string(soln.iter().copied()), HARD_25X25_SOLN);
    }

    #[test]
    fn test_edge_intersections() {
        // parallel intersections
        assert!(Edge::V {
            x: 0,
            y_range: (0, 3)
        }
        .intersects(Edge::V {
            x: 0,
            y_range: (2, 4)
        }));
        assert!(!Edge::V {
            x: 0,
            y_range: (0, 2)
        }
        .intersects(Edge::V {
            x: 0,
            y_range: (2, 4)
        }));
        assert!(!Edge::V {
            x: 0,
            y_range: (2, 5)
        }
        .intersects(Edge::V {
            x: 0,
            y_range: (2, 4)
        }));
        assert!(Edge::H {
            y: 0,
            x_range: (0, 3)
        }
        .intersects(Edge::H {
            y: 0,
            x_range: (2, 4)
        }));
        assert!(!Edge::H {
            y: 0,
            x_range: (0, 2)
        }
        .intersects(Edge::H {
            y: 0,
            x_range: (2, 4)
        }));
        assert!(!Edge::H {
            y: 0,
            x_range: (2, 5)
        }
        .intersects(Edge::H {
            y: 0,
            x_range: (2, 4)
        }));

        // perpendicular intersections
        assert!(Edge::V {
            x: 1,
            y_range: (0, 2)
        }
        .intersects(Edge::H {
            y: 1,
            x_range: (0, 2)
        }));
        assert!(!Edge::V {
            x: 2,
            y_range: (0, 2)
        }
        .intersects(Edge::H {
            y: 1,
            x_range: (0, 2)
        }));
        assert!(!Edge::V {
            x: 1,
            y_range: (0, 2)
        }
        .intersects(Edge::H {
            y: 2,
            x_range: (0, 2)
        }));
    }
}
