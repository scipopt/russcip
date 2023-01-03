use russcip::solve;

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Expected filename of lp file as an argument");
    solve(filename)
}
