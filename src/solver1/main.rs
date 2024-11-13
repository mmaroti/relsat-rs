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

use super::solver::*;

pub fn main1() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 7);
    let one = sol.add_variable("one", vec![set]);
    let inv = sol.add_variable("inv", vec![set, set]);
    let mul = sol.add_variable("mul", vec![set, set, set]);
    let equ = sol.add_variable("equ", vec![set, set]);

    // equivalence relation
    if true {
        sol.add_clause(vec![(true, &equ, vec![0, 0])]);

        sol.add_clause(vec![(false, &equ, vec![0, 1]), (true, &equ, vec![1, 0])]);

        sol.add_clause(vec![
            (false, &equ, vec![0, 1]),
            (false, &equ, vec![1, 2]),
            (true, &equ, vec![0, 2]),
        ]);
    }

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (true, &mul, vec![0, 5, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 0]),
        (false, &mul, vec![1, 1, 2]),
        (true, &mul, vec![0, 2, 0]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![1, 0, 2]),
        (true, &mul, vec![0, 1, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![1, 2, 3]),
        (true, &mul, vec![0, 3, 3]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 3]),
        (false, &mul, vec![0, 3, 4]),
        (false, &mul, vec![0, 1, 5]),
        (true, &mul, vec![5, 2, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![0, 0, 2]),
        (true, &mul, vec![2, 1, 1]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![0, 1, 2]),
        (true, &mul, vec![1, 0, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 1]),
        (false, &mul, vec![0, 1, 3]),
        (true, &mul, vec![3, 2, 3]),
    ]);

    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &mul, vec![1, 0, 2]),
        (true, &one, vec![2]),
    ]);

    sol.add_clause(vec![(false, &one, vec![0]), (true, &mul, vec![0, 1, 1])]);

    // mul is an operation
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 2]),
        (false, &mul, vec![0, 1, 3]),
        (true, &equ, vec![2, 3]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, &mul, vec![0, 1, 2]),
            (false, &equ, vec![0, 3]),
            (true, &mul, vec![3, 1, 2]),
        ]);

        sol.add_clause(vec![
            (false, &mul, vec![0, 1, 2]),
            (false, &equ, vec![1, 3]),
            (true, &mul, vec![0, 3, 2]),
        ]);

        sol.add_clause(vec![
            (false, &mul, vec![0, 1, 2]),
            (false, &equ, vec![2, 3]),
            (true, &mul, vec![0, 1, 3]),
        ]);
    }

    sol.add_exist(&mul);

    // inv is an operation
    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &inv, vec![0, 2]),
        (true, &equ, vec![1, 2]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, &inv, vec![0, 1]),
            (false, &equ, vec![1, 2]),
            (true, &inv, vec![0, 2]),
        ]);
        sol.add_clause(vec![
            (false, &inv, vec![0, 1]),
            (false, &equ, vec![0, 2]),
            (true, &inv, vec![2, 1]),
        ]);
    }

    sol.add_exist(&inv);

    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &one, vec![1]),
        (true, &equ, vec![0, 1]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, &one, vec![0]),
            (false, &equ, vec![0, 1]),
            (true, &one, vec![1]),
        ]);
    }

    sol.add_exist(&one);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &inv, vec![1, 1]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &inv, vec![1, 2]),
        (false, &mul, vec![1, 0, 2]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &mul, vec![1, 1, 2]),
        (false, &mul, vec![1, 2, 0]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &inv, vec![1, 2]),
        (false, &inv, vec![0, 0]),
        (false, &mul, vec![1, 1, 2]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &mul, vec![1, 0, 2]),
        (false, &inv, vec![2, 1]),
        (false, &mul, vec![1, 1, 3]),
        (false, &inv, vec![3, 1]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &inv, vec![1, 2]),
        (false, &mul, vec![1, 1, 3]),
        (false, &inv, vec![3, 2]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &mul, vec![1, 0, 2]),
        (false, &inv, vec![2, 1]),
        (false, &mul, vec![1, 1, 3]),
        (false, &inv, vec![3, 2]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    sol.set_value(true, &one, &[0]);
    sol.set_value(false, &mul, &[1, 0, 1]);
    sol.propagate_clauses();
    // sol.set_value(true, &inv, &[1, 2]);
    sol.propagate_clauses();
    sol.set_value(true, &mul, &[1, 0, 3]);
    // sol.set_value(true, &inv, &[0, 0]);
    // sol.set_value(true, &inv, &[2, 1]);
    // sol.set_value(true, &inv, &[3, 1]);
    sol.propagate_clauses();
    sol.set_value(true, &mul, &[1, 1, 4]);
    sol.propagate_clauses();
    sol.set_value(true, &inv, &[4, 5]);
    sol.propagate_clauses();
    sol.set_value(true, &mul, &[1, 3, 6]);

    sol.search_all();
}

pub fn main2() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 3);

    let equ = sol.add_variable("equ", vec![set, set]);
    sol.set_equality(&equ);

    let mul = sol.add_variable("mul", vec![set, set, set]);

    sol.add_exist(&mul);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 2]),
        (false, &mul, vec![0, 1, 3]),
        (true, &equ, vec![2, 3]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (true, &mul, vec![0, 5, 4]),
    ]);

    // trivial consequences but not unit propagated

    sol.add_clause(vec![
        (false, &mul, vec![2, 1, 2]),
        (false, &mul, vec![1, 1, 0]),
        (true, &mul, vec![2, 0, 2]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 1, 0]),
        (false, &mul, vec![0, 1, 0]),
        (true, &mul, vec![1, 0, 0]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 0]),
        (false, &mul, vec![0, 2, 1]),
        (true, &mul, vec![0, 1, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 2, 2]),
        (false, &mul, vec![2, 1, 1]),
        (true, &mul, vec![0, 1, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 2, 2]),
        (false, &mul, vec![2, 0, 1]),
        (true, &mul, vec![0, 1, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 2]),
        (false, &mul, vec![2, 2, 0]),
        (true, &mul, vec![1, 0, 0]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![2, 2, 0]),
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![2, 1, 2]),
        (true, &equ, vec![0, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![2, 1, 0]),
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![1, 1, 1]),
        (true, &equ, vec![0, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 1, 0]),
        (false, &mul, vec![0, 1, 1]),
        (true, &mul, vec![1, 0, 1]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 1, 0]),
        (false, &mul, vec![0, 1, 2]),
        (true, &mul, vec![1, 0, 2]),
    ]);

    sol.search_all();
}

pub fn main3() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 3);

    let equ = sol.add_variable("equ", vec![set, set]);
    sol.set_equality(&equ);

    /*
    let ord = sol.add_variable("ord", vec![&set, &set]);
    sol.add_clause(vec![(true, &ord, vec![0, 0])]);
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![1, 0]),
        (true, &equ, vec![0, 1]),
    ]);
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![1, 2]),
        (true, &ord, vec![0, 2]),
    ]);
    */

    let mul = sol.add_variable("mul", vec![set, set, set]);
    sol.add_exist(&mul);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 2]),
        (false, &mul, vec![0, 1, 3]),
        (true, &equ, vec![2, 3]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (false, &mul, vec![0, 5, 6]),
        (true, &equ, vec![4, 6]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 1, 0]),
        (false, &mul, vec![2, 0, 0]),
        (false, &mul, vec![2, 1, 2]),
        (true, &equ, vec![0, 2]),
    ]);

    /*
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![2, 3]),
        (false, &mul, vec![0, 2, 4]),
        (false, &mul, vec![1, 3, 5]),
        (true, &ord, vec![4, 5]),
    ]);
    */

    sol.search_all();
}
