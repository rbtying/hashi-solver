use std::io::Read;
use hashi_solver::{Board, SolveState};

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    println!("solving...");

    let b = Board::parse(&s);
    let soln = SolveState::new(&b).solve().unwrap();

    for i in 0..soln.len() {
        println!("{}", b.serialize_to_string(soln.iter().copied().take(i)));
        println!();
    }
}
