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
mod solver2;
mod theory;
mod tokenizer;

use solver2::*;

fn main() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 3);
    let equ = sol.add_variable("equ", vec![&set, &set]);
    let mul = sol.add_variable("mul", vec![&set, &set, &set]);
    let inv = sol.add_variable("inv", vec![&set, &set]);
    let one = sol.add_variable("one", vec![&set]);

    sol.add_clause(vec![
        (false, &mul, vec![3, 0, 1]),
        (false, &mul, vec![4, 3, 2]),
        (false, &mul, vec![5, 1, 2]),
        (true, &mul, vec![4, 0, 5]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![3, 1, 2]),
        (false, &mul, vec![4, 0, 3]),
        (false, &mul, vec![5, 0, 1]),
        (true, &mul, vec![4, 5, 2]),
    ]);

    sol.add_clause(vec![
        (false, &inv, vec![1, 0]),
        (false, &mul, vec![2, 1, 0]),
        (true, &one, vec![2]),
    ]);

    sol.add_clause(vec![(false, &one, vec![0]), (true, &mul, vec![1, 0, 1])]);

    sol.add_clause(vec![
        (false, &mul, vec![2, 0, 1]),
        (false, &mul, vec![3, 0, 1]),
        (true, &equ, vec![2, 3]),
    ]);

    sol.add_clause(vec![
        (false, &inv, vec![1, 0]),
        (false, &inv, vec![2, 0]),
        (true, &equ, vec![1, 2]),
    ]);

    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &one, vec![1]),
        (true, &equ, vec![0, 1]),
    ]);

    sol.print();
}
