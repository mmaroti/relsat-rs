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
            (false, &mul, vec![3, 0, 1]),
            (false, &mul, vec![4, 3, 2]),
            (false, &mul, vec![5, 1, 2]),
            (true, &mul, vec![4, 0, 5]),
        ],
    );

    theory.add_clause(
        6,
        vec![
            (false, &mul, vec![3, 1, 2]),
            (false, &mul, vec![4, 0, 3]),
            (false, &mul, vec![5, 0, 1]),
            (true, &mul, vec![4, 5, 2]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (false, &inv, vec![1, 0]),
            (false, &mul, vec![2, 1, 0]),
            (true, &one, vec![2]),
        ],
    );

    theory.add_clause(2, vec![(false, &one, vec![0]), (true, &mul, vec![1, 0, 1])]);

    theory.add_clause(
        4,
        vec![
            (false, &mul, vec![2, 0, 1]),
            (false, &mul, vec![3, 0, 1]),
            (true, &equ, vec![2, 3]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (false, &inv, vec![1, 0]),
            (false, &inv, vec![2, 0]),
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

    let solver = Solver::new(theory, 2);
    solver.get_relation(&equ).unwrap().set_equ();
    solver.print();
}
