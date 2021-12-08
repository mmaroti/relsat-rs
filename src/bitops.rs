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

#[inline(always)]
pub const fn operation_222(oper: u32, a: u32, b: u32) -> u32 {
    debug_assert!(a <= 3 && b <= 3);
    (oper >> ((a << 3) | (b << 1))) & 3
}

const fn define_222(cases: &[(u32, u32, u32)]) -> u32 {
    debug_assert!(cases.len() == 16);
    let mut set: u32 = 0;
    let mut val: u32 = 0;
    let mut idx = 0;
    while idx < cases.len() {
        let (a, b, c) = cases[idx];
        debug_assert!(a <= 3 && b <= 3 && c <= 3);
        let pos = (a << 3) | (b << 1);
        val |= c << pos;
        set |= 3 << pos;
        idx += 1;
    }
    debug_assert!(set == 0xffffffff);
    val
}

pub const BOOL_FALSE: u32 = 0;
pub const BOOL_UNDEF: u32 = 1;
pub const BOOL_TRUE: u32 = 2;
pub const BOOL_MISSING: u32 = 3;

pub const BOOL_FORMAT: [char; 4] = ['0', '?', '1', 'x'];

pub const BOOL_OR: u32 = define_222(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_FALSE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_FALSE, BOOL_MISSING, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_FALSE, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_TRUE, BOOL_TRUE),
    (BOOL_UNDEF, BOOL_MISSING, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_FALSE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF, BOOL_TRUE),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_MISSING, BOOL_TRUE),
    (BOOL_MISSING, BOOL_FALSE, BOOL_FALSE),
    (BOOL_MISSING, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_MISSING, BOOL_TRUE, BOOL_TRUE),
    (BOOL_MISSING, BOOL_MISSING, BOOL_MISSING),
]);

pub const BOOL_AND: u32 = define_222(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF, BOOL_FALSE),
    (BOOL_FALSE, BOOL_TRUE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_MISSING, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_FALSE, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_TRUE, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_MISSING, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_TRUE, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_MISSING, BOOL_TRUE),
    (BOOL_MISSING, BOOL_FALSE, BOOL_FALSE),
    (BOOL_MISSING, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_MISSING, BOOL_TRUE, BOOL_TRUE),
    (BOOL_MISSING, BOOL_MISSING, BOOL_MISSING),
]);

pub const EVAL_FALSE: u32 = 0;
pub const EVAL_UNIT: u32 = 1;
pub const EVAL_UNDEF: u32 = 2;
pub const EVAL_TRUE: u32 = 3;

pub const EVAL_FORMAT: [char; 4] = ['0', '!', '?', '1'];

pub const FOLD_POS: u32 = define_222(&[
    (EVAL_FALSE, BOOL_FALSE, EVAL_FALSE),
    (EVAL_FALSE, BOOL_UNDEF, EVAL_UNIT),
    (EVAL_FALSE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_FALSE, BOOL_MISSING, EVAL_FALSE),
    (EVAL_UNIT, BOOL_FALSE, EVAL_UNIT),
    (EVAL_UNIT, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNIT, BOOL_TRUE, EVAL_TRUE),
    (EVAL_UNIT, BOOL_MISSING, EVAL_UNIT),
    (EVAL_UNDEF, BOOL_FALSE, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_TRUE, EVAL_TRUE),
    (EVAL_UNDEF, BOOL_MISSING, EVAL_UNDEF),
    (EVAL_TRUE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_UNDEF, EVAL_TRUE),
    (EVAL_TRUE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_MISSING, EVAL_TRUE),
]);

pub const FOLD_NEG: u32 = define_222(&[
    (EVAL_FALSE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_FALSE, BOOL_UNDEF, EVAL_UNIT),
    (EVAL_FALSE, BOOL_TRUE, EVAL_FALSE),
    (EVAL_FALSE, BOOL_MISSING, EVAL_FALSE),
    (EVAL_UNIT, BOOL_FALSE, EVAL_TRUE),
    (EVAL_UNIT, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNIT, BOOL_TRUE, EVAL_UNIT),
    (EVAL_UNIT, BOOL_MISSING, EVAL_UNIT),
    (EVAL_UNDEF, BOOL_FALSE, EVAL_TRUE),
    (EVAL_UNDEF, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_TRUE, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_MISSING, EVAL_UNDEF),
    (EVAL_TRUE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_UNDEF, EVAL_TRUE),
    (EVAL_TRUE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_MISSING, EVAL_TRUE),
]);

pub const EVAL_AND: u32 = define_222(&[
    (EVAL_FALSE, EVAL_FALSE, EVAL_FALSE),
    (EVAL_FALSE, EVAL_UNIT, EVAL_FALSE),
    (EVAL_FALSE, EVAL_UNDEF, EVAL_FALSE),
    (EVAL_FALSE, EVAL_TRUE, EVAL_FALSE),
    (EVAL_UNIT, EVAL_FALSE, EVAL_FALSE),
    (EVAL_UNIT, EVAL_UNIT, EVAL_UNIT),
    (EVAL_UNIT, EVAL_UNDEF, EVAL_UNIT),
    (EVAL_UNIT, EVAL_TRUE, EVAL_UNIT),
    (EVAL_UNDEF, EVAL_FALSE, EVAL_FALSE),
    (EVAL_UNDEF, EVAL_UNIT, EVAL_UNIT),
    (EVAL_UNDEF, EVAL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, EVAL_TRUE, EVAL_UNDEF),
    (EVAL_TRUE, EVAL_FALSE, EVAL_FALSE),
    (EVAL_TRUE, EVAL_UNIT, EVAL_UNIT),
    (EVAL_TRUE, EVAL_UNDEF, EVAL_UNDEF),
    (EVAL_TRUE, EVAL_TRUE, EVAL_TRUE),
]);
