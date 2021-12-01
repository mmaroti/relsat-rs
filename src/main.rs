/*
* Copyright (C) 2019-2020, Miklos Maroti
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
mod theory;

// use shape::*;
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
            (mul, false, vec![0, 1, 3]),
            (mul, false, vec![3, 2, 4]),
            (mul, false, vec![1, 2, 5]),
            (mul, true, vec![0, 5, 4]),
        ],
    );

    theory.add_clause(
        6,
        vec![
            (mul, false, vec![1, 2, 3]),
            (mul, false, vec![0, 3, 4]),
            (mul, false, vec![0, 1, 5]),
            (mul, true, vec![5, 2, 4]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (inv, false, vec![0, 1]),
            (mul, false, vec![1, 0, 2]),
            (one, true, vec![2]),
        ],
    );

    theory.add_clause(2, vec![(one, false, vec![0]), (mul, true, vec![0, 1, 1])]);

    theory.add_clause(
        4,
        vec![
            (mul, false, vec![0, 1, 2]),
            (mul, false, vec![0, 1, 3]),
            (equ, true, vec![2, 3]),
        ],
    );

    theory.add_clause(
        3,
        vec![
            (inv, false, vec![0, 1]),
            (inv, false, vec![0, 2]),
            (equ, true, vec![1, 2]),
        ],
    );

    theory.add_clause(
        2,
        vec![
            (one, false, vec![0]),
            (one, false, vec![1]),
            (equ, true, vec![0, 1]),
        ],
    );

    theory.print()
}
