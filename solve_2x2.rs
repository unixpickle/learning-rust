// A 2x2 Rubik's cube solver adapted from an MIT course:
// https://courses.csail.mit.edu/6.006/fall07/source/rubik.py
// https://courses.csail.mit.edu/6.006/fall07/source/test-rubik.py

use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Cube([u8; 24]);

impl Cube {
    fn new() -> Cube {
        let mut res = Cube([0; 24]);
        for i in 0..24 {
            res.0[i] = i as u8;
        }
        res
    }

    fn permute(&self, perm: &Cube) -> Cube {
        let mut res = Cube::new();
        for (i, c) in perm.0.into_iter().enumerate() {
            res.0[i as usize] = self.0[*c as usize];
        }
        res
    }

    fn permute_inv(&self, perm: &Cube) -> Cube {
        let mut res: Cube = Cube([0; 24]);
        for (i, c) in perm.0.into_iter().enumerate() {
            res.0[*c as usize] = self.0[i as usize];
        }
        res
    }
}

#[derive(Debug, Clone, Copy)]
enum Move {
    F,
    Fi,
    L,
    Li,
    U,
    Ui
}

impl Move {
    fn apply(&self, cube: &Cube) -> Cube {
        let f_move = Cube([6, 7, 8, 0, 1, 2, 9, 10, 11, 3, 4, 5, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23]);
        let l_move = Cube([13, 14, 12, 3, 4, 5, 2, 0, 1, 9, 10, 11, 20, 18,
            19, 15, 16, 17, 7, 8, 6, 21, 22, 23]);
        let u_move = Cube([5, 3, 4, 16, 17, 15, 6, 7, 8, 9, 10, 11, 1, 2, 0,
            14, 12, 13, 18, 19, 20, 21, 22, 23]);
        // TODO: why does rustc give a warning without Move::?
        match *self {
            Move::F => cube.permute(&f_move),
            Move::Fi => cube.permute_inv(&f_move),
            Move::L => cube.permute(&l_move),
            Move::Li => cube.permute_inv(&l_move),
            Move::U => cube.permute(&u_move),
            Move::Ui => cube.permute_inv(&u_move)
        }
    }

    fn inverse(&self) -> Move {
        match *self {
            Move::F => Move::Fi,
            Move::Fi => Move::F,
            Move::L => Move::Li,
            Move::Li => Move::L,
            Move::U => Move::Ui,
            Move::Ui => Move::U
        }
    }
}

#[derive(Clone)]
struct PartialSolution(Cube, Vec<Move>);

struct Searcher {
    found: HashMap<Cube, Vec<Move>>,
    to_expand: VecDeque<PartialSolution>
}

impl Searcher {
    fn new(start: &Cube) -> Searcher {
        let mut res = Searcher{
            found: HashMap::new(),
            to_expand: VecDeque::new()
        };
        res.found.insert(*start, Vec::new());
        res.to_expand.push_back(PartialSolution(*start, Vec::new()));
        res
    }

    fn expand_depth(&mut self) {
        let num_search = self.to_expand.len();
        for _ in 0..num_search {
            match self.to_expand.pop_front() {
                Some(partial) => {
                    for m in &[Move::F, Move::Fi, Move::L, Move::Li, Move::U, Move::Ui] {
                        let next_cube = m.apply(&partial.0);
                        if self.found.contains_key(&next_cube) {
                            continue
                        }
                        let mut next_sequence = partial.1.clone();
                        next_sequence.push(*m);
                        self.found.insert(next_cube, next_sequence.clone());
                        self.to_expand.push_back(PartialSolution(next_cube, next_sequence));
                    }
                },
                None => panic!("should not be empty")
            }
        }
    }
}

fn fwd_bwd_intersection(fwd: &Searcher, bwd: &Searcher) -> Option<Vec<Move>> {
    // TODO: why is fwd.to_expand not already a reference here?
    for partial in &fwd.to_expand {
        // TODO: why is partial.0 not already a reference here?
        if bwd.found.contains_key(&partial.0) {
            // TODO: how come we don't need &partial.1 here to
            // call a method on it? Is it because self is a
            // reference already?
            let mut part1 = partial.1.clone();
            let mut part2 = bwd.found[&partial.0].clone();
            part2.reverse();
            part1.extend(part2.into_iter().map(|x| x.inverse()));
            return Some(part1);
        }
    }
    None
}

macro_rules! check_solution {
    ($fwd:expr, $bwd:expr) => {
        match fwd_bwd_intersection($fwd, $bwd) {
            Some(solution) => return Some(solution),
            None => ()
        }
    }
}

fn solve(c: &Cube) -> Option<Vec<Move>> {
    let mut fwd = Searcher::new(c);
    let mut bwd = Searcher::new(&Cube::new());
    check_solution!(&fwd, &bwd);
    for i in 0..7 {
        fwd.expand_depth();
        check_solution!(&fwd, &bwd);
        bwd.expand_depth();
        check_solution!(&fwd, &bwd);
    }
    None
}

fn main() {
    // Demonstrate solving a 2-move scramble.
    let mut easy = Cube::new();
    easy = Move::F.apply(&easy);
    easy = Move::L.apply(&easy);
    easy = Move::U.apply(&easy);
    match solve(&easy) {
        Some(solution) => println!("simple cube solution: {:?}", solution),
        None => println!("simple cube has no solution!")
    }

    // Demonstrate solving a 14-move scramble.
    let hard = Cube([6, 7, 8, 20, 18, 19, 3, 4, 5, 16, 17, 15, 0, 1, 2, 14,
        12, 13, 10, 11, 9, 21, 22, 23]);
    match solve(&hard) {
        Some(solution) => println!("hard cube solution: {:?}", solution),
        None => println!("hard cube has no solution!")
    }
}
