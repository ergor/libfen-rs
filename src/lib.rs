

//! Module for parsing Forsyth–Edwards Notation (FEN) in chess.

use regex::Regex;

static RANK_REGEX: &str = r"([prnbqkbnrPRNBQKBNR1-8]{1,8})/?";

const WHITE_KINGSIDE: i32 =  1 << 1;
const WHITE_QUEENSIDE: i32 = 1 << 2;
const BLACK_KINGSIDE: i32 =  1 << 3;
const BLACK_QUEENSIDE: i32 = 1 << 4;


macro_rules! prettyprint {
    ( $msg:expr, $endl:expr ) => {
        print!("fen-rs: {}{}", $msg, $endl);
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White,
    Black
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Kind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Position(usize, usize);

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    kind: Kind,
    color: Color,
    position: Position,
}

#[derive(Debug)]
pub struct GameState {
    /// organization: [y][x]
    pieces: [[Option<Piece>; 8]; 8],
    active_color: Color,
    castling_availability: i32,
    en_passant: Option<Position>,
    /// This is the number of halfmoves since the last capture or pawn advance.
    half_move_clock: i32,
    /// The number of the full move. It starts at 1, and is incremented after Black's move
    full_move_clock: i32,
}

pub fn parse(fen_str: &str) -> GameState {
    let mut split = fen_str.split_whitespace();
    let pieces = parse_ranks(split.next());

    let mut game_state = GameState {
        pieces: [[None; 8]; 8],
        active_color: Color::White,
        castling_availability: 0,
        en_passant: None,
        half_move_clock: 0,
        full_move_clock: 0
    };

    // organization: [y][x]
    for piece in pieces {
        game_state.pieces[piece.position.1][piece.position.0] = Some(piece);
    }

    return game_state;
}

fn parse_ranks(ranks: Option<&str>) -> Vec<Piece> {

    if let Some(ranks) = ranks {
        let pattern = format!("^{}$", RANK_REGEX.repeat(8));
        //println!("pattern: {}", pattern);
        let pattern = pattern.as_str();
        let re = Regex::new(pattern).unwrap();
        let cap = re.captures(ranks);
        //println!("captures: {:?}", cap);
        if let Some(cap) = cap {
            return cap.iter()
                .enumerate()
                .skip(1) // skip capture[0], because it is the whole match
                .flat_map(|(cap_idx, re_match)| parse_rank(7-(cap_idx-1), re_match.unwrap().as_str()))
                .collect();
        }
    }

    prettyprint!("pieces could not be parsed; returning empty set.", "\n");
    return Vec::new();
}


fn parse_rank(y: usize, rank: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();

    if y > 7 {
        return pieces;
    }

    let mut x = 0;
    for c in rank.chars() {
        if x >= 8 {
            prettyprint!(format!("rank {} should have been blank but there are more pieces. ignoring them.", y), "\n");
            break;
        }
        let position = Position(x, y);
        let color = if char::is_ascii_uppercase(&c) { Color::White } else { Color::Black };
        match c {
            'p' | 'P' => pieces.push(Piece { kind: Kind::Pawn, color, position }),
            'r' | 'R' => pieces.push(Piece { kind: Kind::Rook, color, position }),
            'n' | 'N' => pieces.push(Piece { kind: Kind::Knight, color, position }),
            'b' | 'B' => pieces.push(Piece { kind: Kind::Bishop, color, position }),
            'q' | 'Q' => pieces.push(Piece { kind: Kind::Queen, color, position }),
            'k' | 'K' => pieces.push(Piece { kind: Kind::King, color, position }),
            '1'..='8' => x += char::to_digit(c, 10).unwrap() as usize,
            _ => prettyprint!(format!("found unexpected token '{}' in rank {}", c, y), "\n")
        };
        if char::is_alphabetic(c) {
            x += 1;
        }
    }

    return pieces;
}


#[cfg(test)]
mod tests {
    use crate::{parse, Piece, Kind, Color, Position, GameState};

    macro_rules! test_piece {
        ( $game_state:expr, $kind:expr, $color:expr, $position:expr ) => {
            let p = $game_state.pieces[$position.1][$position.0].unwrap();
            assert!(p.kind == $kind && p.color == $color && p.position == $position);
        };
    }

    macro_rules! test_empty {
        ( $game_state:expr, $position:expr ) => {
            assert!($game_state.pieces[$position.1][$position.0].is_none());
        };
    }

    #[test]
    fn empty_board() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let game_state = parse(fen);
        for y in 0..=7 {
            for x in 0..=7 {
                test_empty!(game_state, Position(x, y));
            }
        }
    }

    #[test]
    fn starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let game_state = parse(fen);

        test_piece!(game_state, Kind::Rook, Color::White, Position(0, 0));
        test_piece!(game_state, Kind::Knight, Color::White, Position(1, 0));
        test_piece!(game_state, Kind::Bishop, Color::White, Position(2, 0));
        test_piece!(game_state, Kind::Queen, Color::White, Position(3, 0));
        test_piece!(game_state, Kind::King, Color::White, Position(4, 0));
        test_piece!(game_state, Kind::Bishop, Color::White, Position(5, 0));
        test_piece!(game_state, Kind::Knight, Color::White, Position(6, 0));
        test_piece!(game_state, Kind::Rook, Color::White, Position(7, 0));
        for x in 0..=7 {
            test_piece!(game_state, Kind::Pawn, Color::White, Position(x, 1));
        }
        for y in 2..=5 {
            for x in 0..=7 {
                test_empty!(game_state, Position(x, y));
            }
        }
        for x in 0..=7 {
            test_piece!(game_state, Kind::Pawn, Color::Black, Position(x, 6));
        }
        test_piece!(game_state, Kind::Rook, Color::Black, Position(0, 7));
        test_piece!(game_state, Kind::Knight, Color::Black, Position(1, 7));
        test_piece!(game_state, Kind::Bishop, Color::Black, Position(2, 7));
        test_piece!(game_state, Kind::Queen, Color::Black, Position(3, 7));
        test_piece!(game_state, Kind::King, Color::Black, Position(4, 7));
        test_piece!(game_state, Kind::Bishop, Color::Black, Position(5, 7));
        test_piece!(game_state, Kind::Knight, Color::Black, Position(6, 7));
        test_piece!(game_state, Kind::Rook, Color::Black, Position(7, 7));
    }

    #[test]
    fn e4_c5_nf3() {
        let fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let game_state = parse(fen);

        test_piece!(game_state, Kind::Rook, Color::White, Position(0, 0));
        test_piece!(game_state, Kind::Knight, Color::White, Position(1, 0));
        test_piece!(game_state, Kind::Bishop, Color::White, Position(2, 0));
        test_piece!(game_state, Kind::Queen, Color::White, Position(3, 0));
        test_piece!(game_state, Kind::King, Color::White, Position(4, 0));
        test_piece!(game_state, Kind::Bishop, Color::White, Position(5, 0));
        test_empty!(game_state, Position(6, 0));
        test_piece!(game_state, Kind::Rook, Color::White, Position(7, 0));
        for x in 0..=7 {
            let pos = Position(x, 1);
            if x == 4 {
                test_empty!(game_state, pos);
            } else {
                test_piece!(game_state, Kind::Pawn, Color::White, pos);
            }
        }
        for x in 0..=7 {
            let pos = Position(x, 2);
            if x == 5 {
                test_piece!(game_state, Kind::Knight, Color::White, pos);
            } else {
                test_empty!(game_state, pos);
            }
        }
        for x in 0..=7 {
            let pos = Position(x, 3);
            if x == 4 {
                test_piece!(game_state, Kind::Pawn, Color::White, pos);
            } else {
                test_empty!(game_state, pos);
            }
        }
        for x in 0..=7 {
            let pos = Position(x, 4);
            if x == 2 {
                test_piece!(game_state, Kind::Pawn, Color::Black, pos);
            } else {
                test_empty!(game_state, pos);
            }
        }
        for x in 0..=7 {
            let pos = Position(x, 6);
            if x == 2 {
                test_empty!(game_state, pos);
            } else {
                test_piece!(game_state, Kind::Pawn, Color::Black, pos);
            }
        }
        test_piece!(game_state, Kind::Rook, Color::Black, Position(0, 7));
        test_piece!(game_state, Kind::Knight, Color::Black, Position(1, 7));
        test_piece!(game_state, Kind::Bishop, Color::Black, Position(2, 7));
        test_piece!(game_state, Kind::Queen, Color::Black, Position(3, 7));
        test_piece!(game_state, Kind::King, Color::Black, Position(4, 7));
        test_piece!(game_state, Kind::Bishop, Color::Black, Position(5, 7));
        test_piece!(game_state, Kind::Knight, Color::Black, Position(6, 7));
        test_piece!(game_state, Kind::Rook, Color::Black, Position(7, 7));
    }
}
