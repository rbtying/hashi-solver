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
extern "C" {
    fn alert(s: &str);
}

fn _solve(s: &str, depth: usize, max_visited: usize) -> Result<String, &'static str> {
    let b = Board::parse(s)?;
    let (soln, log) = SolveState::new(&b).solve(depth, max_visited)?;
    let mut results = vec![];

    for i in 0..soln.len() {
        writeln!(&mut results).unwrap();
        writeln!(&mut results, "Step {}", i + 1).unwrap();
        writeln!(&mut results, "{}", log[i]).unwrap();
        writeln!(&mut results).unwrap();
        write!(
            &mut results,
            "{}",
            b.serialize_to_string(soln.iter().copied().take(i + 1))
        )
        .unwrap();
    }

    Ok(String::from_utf8_lossy(&results).to_string())
}

#[wasm_bindgen]
pub fn solve(s: &str, depth: usize) -> String {
    match _solve(s, depth, 10_000) {
        Ok(r) => r,
        Err(e) => e.to_string(),
    }
}
