use std::default::Default;

pub struct GameState {
    pub players: Vec<Player>,
    pub board: [[Square; 20]; 20],
}

pub struct Player {
    pub preferred_color: Color,
    pub potato_count: usize,
    pub piece_counts: PieceCounts,
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub enum Square {
    Empty,
    PlayerOne(Piece),
    PlayerTwo(Piece),
    PlayerThree(Piece),
    PlayerFour(Piece),
}

pub enum Piece {
    Hovedby(Option<Kartopult>),
    ØdelagtHovedby,
    By(Option<Kartopult>),
    Vej(Option<Kartopult>),
    Mur,
}

pub enum Kartopult {
    Ret,
    Diagonal,
}

pub struct PieceCounts {
    pub by: usize,
    pub kartopult_ret: usize,
    pub kartopult_diagonal: usize,
    pub vej: usize,
    pub mur: usize,
}

impl Default for PieceCounts {
    fn default() -> Self {
        Self {
            by: 5,
            kartopult_ret: 5,
            kartopult_diagonal: 5,
            vej: 40,
            mur: 10,
        }
    }
}

pub struct Round {
    pub builds: Vec<Building>,
    pub kartopult_move: Vec<KartopultMove>,
    pub kartopult_shots: Vec<KartopultShot>,
}

pub struct Building {
    pub cost: usize,
    pub x: usize,
    pub y: usize,
    pub piece: BuildablePiece,
}

pub enum BuildablePiece {
    By,
    Vej,
    Mur,
    Kartopult(Kartopult),
}

pub struct KartopultMove {
    pub x: usize,
    pub y: usize,
    pub moves: Vec<DirectionRet>,
}

pub struct KartopultShot {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub power: KartopultPower,
}

pub enum KartopultPower {
    One = 1,
    Two = 2,
    Three = 3,
}

pub enum Direction {
    Ret(DirectionDiagonal),
    Diagonal(DirectionRet),
}

pub enum DirectionRet {
    Up,
    Right,
    Down,
    Left,
}

pub enum DirectionDiagonal {
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}
