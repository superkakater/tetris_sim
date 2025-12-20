use crate::grid::Grid;

const GAP: usize = 5;

fn in_blind(r: usize, c: usize) -> bool {
    let blind_row_start = 9usize;
    let blind_row_end = 18usize;
    let blind_col_start = 2usize;
    let blind_col_end = 8usize;
    r >= blind_row_start && r <= blind_row_end && c >= blind_col_start && c <= blind_col_end
}

fn print_two_row(m1: &Vec<Vec<char>>, m2: &Vec<Vec<char>>, row: usize, blind1: bool, blind2: bool) {
    for c in 0..11 {
        let mut ch = m1[row][c];
        if blind1 && in_blind(row, c) { ch = '?'; }
        print!("{}", ch);
    }
    for _ in 0..GAP { print!(" "); }
    for c in 0..11 {
        let mut ch = m2[row][c];
        if blind2 && in_blind(row, c) { ch = '?'; }
        print!("{}", ch);
    }
    println!();
}

pub fn print_two_boards(g1: &Grid, g2: &Grid, blind1: bool, blind2: bool, hi_score: i32) {
    let m1 = g1.matrix();
    let m2 = g2.matrix();

    println!("\nHi Score: {}\n", hi_score);

    // mimic the C++: print rows 1..=3, then 4..=21, then 22..=25 (skip row0)
    print_two_row(m1, m2, 1, blind1, blind2);
    print_two_row(m1, m2, 2, blind1, blind2);
    print_two_row(m1, m2, 3, blind1, blind2);

    for r in 4..=21 {
        print_two_row(m1, m2, r, blind1, blind2);
    }

    print_two_row(m1, m2, 22, blind1, blind2);
    print_two_row(m1, m2, 23, blind1, blind2);
    print_two_row(m1, m2, 24, blind1, blind2);
    print_two_row(m1, m2, 25, blind1, blind2);

    println!();
}
