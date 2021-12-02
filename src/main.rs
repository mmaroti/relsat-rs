/*
* Copyright (C) 2019-2021, Miklos Maroti
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

#![allow(dead_code)]

mod buffer;
mod shape;
mod solver;
mod theory;

use solver::*;
use theory::*;

fn main() {
    let mut theory: Theory = Default::default();
    let equ = theory.add_variable("equ", 2);
    let mul = theory.add_variable("mul", 3);
    let inv = theory.add_variable("inv", 2);
    let one = theory.add_variable("one", 1);

    theory.add_clause(
        6,
        vec![
            (false, &mul, vec![0, 1, 3]),
            (false, &mul, vec![3, 2, 4]),
            (false, &mul, vec![1, 2, 5]),
            (true, &mul, vec![0, 5, 4]),
        ],
    );

    theory.add_clause(
        6,
        vec![
            (false, &mul, vec![1, 2, 3]),
            (false, &mul, vec![0, 3, 4]),
            (false, &mul, vec![0, 1, 5]),
            (true, &mul, vec![5, 2, 4]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (false, &inv, vec![0, 1]),
            (false, &mul, vec![1, 0, 2]),
            (true, &one, vec![2]),
        ],
    );

    theory.add_clause(2, vec![(false, &one, vec![0]), (true, &mul, vec![0, 1, 1])]);

    theory.add_clause(
        4,
        vec![
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 1, 3]),
            (true, &equ, vec![2, 3]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (false, &inv, vec![0, 1]),
            (false, &inv, vec![0, 2]),
            (true, &equ, vec![1, 2]),
        ],
    );

    theory.add_clause(
        2,
        vec![
            (false, &one, vec![0]),
            (false, &one, vec![1]),
            (true, &equ, vec![0, 1]),
        ],
    );

    theory.print();

    let mut solver = Solver::new(theory, 2);
    solver.get_relation(&equ).unwrap().set_equ();
    solver.print();
}
