use std::io::Write;

use hashi_solver::{Board, SolveState};

mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn solve(s: &str) -> String {
    let b = Board::parse(s);
    let soln = SolveState::new(&b).solve().unwrap();
    let mut results = vec![];

    for i in 0..soln.len() {
        writeln!(&mut results, "Step {}", i+1).unwrap();
        writeln!(&mut results).unwrap();
        write!(&mut results, "{}", b.serialize_to_string(soln.iter().copied().take(i))).unwrap();
    }

    String::from_utf8_lossy(&results).to_string()
}
