use bitvec::prelude::*;

const ROW_LEN: usize = 5;

// I considered doing BitArr!(for ROW_LEN * ROW_LEN, in u32)
// but the bitslice API kept asserting I was trying to assign to [()]
// for reasons I could not understand.
pub type Shape = [BitArr!(for ROW_LEN, in u8); 5];

pub const EMPTY_SHAPE: Shape = [bitarr![u8, Lsb0; 0; ROW_LEN]; 5];

pub const PIECE_SHAPES: [Shape; 21] = [
    // DOT - 0
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // LINE2 - 1
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // LINE3 - 2
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // L3 - 3
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // LINE4 - 4
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
    ],
    // L4 - 5
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // ZIG-ZAG - 6
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // SQUARE - 7
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // TEE - 8
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // LINE5 - 9
    [
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
    ],
    // L5 - 10
    [
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // EXTENDED ZIG - 11
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 1, 0],
    ],
    // EXTENDED TEE - 12
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
    ],
    // U - 13
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 1, 0, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // NOTCH SQUARE - 14
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // BIG TEE - 15
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 1, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // BIG L5 - 16
    [
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 1],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // STAIRS - 17
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // WIDE ZIG - 18
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // CHAIR - 19
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
    // PLUS - 20
    [
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 1, 1, 1, 0],
        bitarr![const u8, Lsb0; 0, 0, 1, 0, 0],
        bitarr![const u8, Lsb0; 0, 0, 0, 0, 0],
    ],
];

// Maybe a little overkill but it's explicit.
#[derive(Debug)]
pub enum RotateDir {
    Right,
    Left,
}

#[derive(Debug)]
pub enum FlipDir {
    Horizontal,
    Vertical,
}

pub fn rotate(shape: Shape, dir: RotateDir) -> Shape {
    match dir {
        // Rotate right TAU/4 := Transpose . Flip Vert
        RotateDir::Right => transpose(flip(shape, FlipDir::Vertical)),
        // Rotate left TAU/4 := Flip Vert . Transpose
        RotateDir::Left => flip(transpose(shape), FlipDir::Vertical),
    }
}

// I initially chose bit arrays because I thought it would offer me some kind of performance gain.
// All I can say it brought me was a marginally faster vertical flip.
// This probably wasn't worth it.
pub fn flip(shape: Shape, dir: FlipDir) -> Shape {
    let mut new_shape = EMPTY_SHAPE;
    match dir {
        FlipDir::Vertical => {
            for (i, row) in shape.into_iter().rev().enumerate() {
                new_shape[i] = row;
            }
        }
        FlipDir::Horizontal => {
            for row in 0..ROW_LEN {
                for col in 0..ROW_LEN {
                    *new_shape[row].get_mut(col).expect("In bounds.") =
                        shape[row][ROW_LEN - col - 1];
                }
            }
        }
    }

    new_shape
}

/// Returns adjusted coordinates if `shape` can be placed at them. Returns `None` otherwise.
pub fn check_bounds_and_recenter(shape: Shape, row: isize, col: isize) -> Option<(isize, isize)> {
    // top row, bottom row, left col, right col
    let mut shape_bounds = [0; 4];

    for (dr, r) in shape.iter().enumerate() {
        for dc in r.iter_ones() {
            let dr = dr as isize - 2;
            let dc = dc as isize - 2;
            // Only update if we have any 1s in this row. If we don't, do nothing.
            if dr < shape_bounds[0] {
                shape_bounds[0] = dr;
            } else if dr > shape_bounds[1] {
                shape_bounds[1] = dr;
            }

            if dc < shape_bounds[2] {
                shape_bounds[2] = dc;
            } else if dc > shape_bounds[3] {
                shape_bounds[3] = dc;
            }
        }
    }

    dbg!(&shape_bounds);

    if row + shape_bounds[0] >= 0
        && row + shape_bounds[1] < 20
        && col + shape_bounds[2] >= 0
        && col + shape_bounds[3] < 20
    {
        Some((row - 2, col - 2))
    } else {
        None
    }
}

// Sure, [[bool; 5]; 5] would have been easier to work with.
// Do I really see any performance wins with this after all? Who knows at this point lmfao
fn transpose(shape: Shape) -> Shape {
    let mut new_shape = EMPTY_SHAPE;
    for row in 0..ROW_LEN {
        for col in 0..ROW_LEN {
            let mut pt = new_shape[col]
                .get_mut(row)
                .expect("Should be in bounds, no?");
            *pt = shape[row][col];
        }
    }

    new_shape
}

mod tests {
    use super::*;

    #[test]
    fn tranpose_ok() {
        let chair = PIECE_SHAPES[19];
        let chair_t = [
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 1, 1, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 1, 1, 0],
            bitarr![u8, Lsb0; 0, 0, 1, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
        ];

        assert_eq!(transpose(chair), chair_t);

        let line5 = PIECE_SHAPES[9];
        let line5_t = [
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 1, 1, 1, 1, 1],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
        ];

        assert_eq!(transpose(line5), line5_t);
    }

    #[test]
    fn flip_ok() {
        let chair = PIECE_SHAPES[19];
        let chair_fv = [
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 1, 0, 0],
            bitarr![u8, Lsb0; 0, 1, 1, 1, 0],
            bitarr![u8, Lsb0; 0, 1, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
        ];
        let chair_fh = [
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 1, 0],
            bitarr![u8, Lsb0; 0, 1, 1, 1, 0],
            bitarr![u8, Lsb0; 0, 0, 1, 0, 0],
            bitarr![u8, Lsb0; 0, 0, 0, 0, 0],
        ];

        assert_eq!(flip(chair, FlipDir::Vertical), chair_fv);
        assert_eq!(flip(chair, FlipDir::Horizontal), chair_fh);
    }
}
