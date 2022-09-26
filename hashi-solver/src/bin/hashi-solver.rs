use hashi_solver::{Board, SolveState};
use std::io::Read;

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    println!("solving...");

    let b = Board::parse(&s);
    let (soln, log) = SolveState::new(&b).solve().unwrap();

    for i in 0..soln.len() {
        println!("{}", log[i]);
        println!("{}", b.serialize_to_string(soln.iter().copied().take(i)));
        println!();
    }
}
