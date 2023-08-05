use crate::logic::TileColor;

pub fn print_board(board: &[[TileColor; 22]; 22]) {
    for row in board {
        for col in row {
            print!("{}", col);
        }
        println!();
    }
}
