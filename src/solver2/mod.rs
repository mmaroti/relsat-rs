/*
* Copyright (C) 2019-2022, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

mod domain;
mod eval1;
mod formula;
mod predicate;
mod solver;

use domain::{get_coords, get_offset, Coord, Domain};
use eval1::{EvalStep, Evaluator};
use formula::{Clause, ClauseIdx, UniversalFormula};
use predicate::{Literal, LiteralIdx, Predicate};
use solver::State;

pub use solver::Solver;

pub fn main() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set".into(), 3);

    let equ = sol.add_predicate("equ".into(), vec![set, set]);
    let one = sol.add_predicate("one".into(), vec![set]);
    let inv = sol.add_predicate("inv".into(), vec![set, set]);
    let mul = sol.add_predicate("mul".into(), vec![set, set, set]);

    sol.add_formula(vec![(false, equ, vec![0, 0])]);

    sol.add_formula(vec![(true, equ, vec![0, 1]), (false, equ, vec![1, 0])]);

    sol.add_formula(vec![
        (true, equ, vec![0, 1]),
        (true, equ, vec![1, 2]),
        (false, equ, vec![0, 2]),
    ]);

    sol.add_formula(vec![
        (true, mul, vec![0, 1, 3]),
        (true, mul, vec![3, 2, 4]),
        (true, mul, vec![1, 2, 5]),
        (false, mul, vec![0, 5, 4]),
    ]);

    sol.add_formula(vec![
        (true, mul, vec![1, 2, 3]),
        (true, mul, vec![0, 3, 4]),
        (true, mul, vec![0, 1, 5]),
        (false, mul, vec![5, 2, 4]),
    ]);

    sol.add_formula(vec![
        (true, inv, vec![0, 1]),
        (true, mul, vec![1, 0, 2]),
        (false, one, vec![2]),
    ]);

    sol.add_formula(vec![(true, one, vec![0]), (false, mul, vec![0, 1, 1])]);

    sol.print();
    sol.test();
}
