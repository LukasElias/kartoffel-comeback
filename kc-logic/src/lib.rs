use {
    serde::{Deserialize, Serialize},
    std::default::Default,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub current_player_number: PlayerNumber,
    pub players: Vec<Player>,
    pub board: [[Square; 20]; 20],
}

impl GameState {
    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_number as usize]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_number as usize]
    }

    pub fn build(mut self, building: Building) -> Result<Self, ()> {
        // Check that the player has enough potatos
        if self.current_player().potato_count < building.piece.cost() {
            return Err(());
        }

        // Check that the player has enough pieces
        if !self.current_player().piece_counts.can_build(building.piece) {
            return Err(());
        }

        if building.x >= self.board.len() || building.y >= self.board[0].len() {
            return Err(());
        };

        match building.piece {
            BuildablePiece::By => {
                let neighbor_squares = ALL_DIRECTION_RET.map(|direction| {
                    let vector = direction.into_vector();

                    let x = building.x as isize + vector.0;
                    let y = building.y as isize + vector.1;

                    if x < 0 || x >= self.board.len() as isize || y < 0 || y >= self.board[0].len() as isize {
                        return None; // Maybe make another type later, for outofbounds errors and
                                     // make it a result instead of option
                    }

                    Some(self.board[x as usize][y as usize])
                });

                let mut next_to_vej = false;

                for neighbor in neighbor_squares {
                    if let Some(neighbor) = neighbor {
                        let is_vej = neighbor.map_piece(|piece, player| {
                            if player == self.current_player_number {
                                match piece {
                                    Piece::Vej(_) => true,
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        });

                        if let Some(is_vej) = is_vej {
                            if is_vej {
                                next_to_vej = true;
                                break;
                            }
                        }
                    }
                }

                if !next_to_vej || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] = Square::from_player_number(&self.current_player_number, building.piece.into())
            },
            BuildablePiece::Vej => {
                let neighbor_squares = ALL_DIRECTION_RET.map(|direction| {
                    let vector = direction.into_vector();

                    let x = building.x as isize + vector.0;
                    let y = building.y as isize + vector.1;

                    if x < 0 || x >= self.board.len() as isize || y < 0 || y >= self.board[0].len() as isize {
                        return None; // Maybe make another type later, for outofbounds errors and
                                     // make it a result instead of option
                    }

                    Some(self.board[x as usize][y as usize])
                });

                let mut next_to_correct_piece = false;

                for neighbor in neighbor_squares {
                    if let Some(neighbor) = neighbor {
                        let is_correct_piece = neighbor.map_piece(|piece, player| {
                            if player == self.current_player_number {
                                match piece {
                                    Piece::Vej(_) => true,
                                    Piece::By(_) => true,
                                    Piece::Hovedby(_) => true,
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        });

                        if let Some(is_correct_piece) = is_correct_piece {
                            if is_correct_piece {
                                next_to_correct_piece = true;
                                break;
                            }
                        }
                    }
                }

                if !next_to_correct_piece || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] = Square::from_player_number(&self.current_player_number, building.piece.into())
            },
            BuildablePiece::Mur => {
                let neighbor_squares = ALL_DIRECTION_RET.map(|direction| {
                    let vector = direction.into_vector();

                    let x = building.x as isize + vector.0;
                    let y = building.y as isize + vector.1;

                    if x < 0 || x >= self.board.len() as isize || y < 0 || y >= self.board[0].len() as isize {
                        return None; // Maybe make another type later, for outofbounds errors and
                                     // make it a result instead of option
                    }

                    Some(self.board[x as usize][y as usize])
                });

                let mut next_to_correct_piece = false;

                for neighbor in neighbor_squares {
                    if let Some(neighbor) = neighbor {
                        let is_correct_piece = neighbor.map_piece(|piece, player| {
                            if player == self.current_player_number {
                                match piece {
                                    Piece::Vej(_) => true,
                                    Piece::By(_) => true,
                                    Piece::Hovedby(_) => true,
                                    Piece::Mur => true,
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        });

                        if let Some(is_correct_piece) = is_correct_piece {
                            if is_correct_piece {
                                next_to_correct_piece = true;
                                break;
                            }
                        }
                    }
                }

                if !next_to_correct_piece || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] = Square::from_player_number(&self.current_player_number, building.piece.into())
            },
            BuildablePiece::Kartopult(kartopult) => {
                let new_piece = self.board[building.x][building.y].map_piece(|piece, player| {
                    if player == self.current_player_number {
                        match piece {
                            Piece::By(None) => Some(Piece::By(Some(kartopult))),
                            Piece::Hovedby(None) => Some(Piece::Hovedby(Some(kartopult))),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }).unwrap_or(None);

                if let Some(piece) = new_piece {
                    // Build
                    self.board[building.x][building.y] = Square::from_player_number(&self.current_player_number, piece);
                } else {
                    // incompatible piece for kartopult building
                    return Err(());
                }
            },
        };

        // Remove the potatos from the player
        let current_player = self.current_player_mut();

        current_player.potato_count -= building.piece.cost();

        // Remove the pieces from the player
        // This function is unchecked since we already checked before
        current_player.piece_counts.build_piece_unchecked(building.piece);

        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub preferred_color: Color,
    pub potato_count: usize,
    pub piece_counts: PieceCounts,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerNumber {
    PlayerOne = 0,
    PlayerTwo = 1,
    PlayerThree = 2,
    PlayerFour = 3,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Square {
    Empty,
    PlayerOne(Piece),
    PlayerTwo(Piece),
    PlayerThree(Piece),
    PlayerFour(Piece),
}

impl Square {
    pub fn from_player_number(player: &PlayerNumber, piece: Piece) -> Self {
        match player {
            PlayerNumber::PlayerOne => Square::PlayerOne(piece),
            PlayerNumber::PlayerTwo => Square::PlayerTwo(piece),
            PlayerNumber::PlayerThree => Square::PlayerThree(piece),
            PlayerNumber::PlayerFour => Square::PlayerFour(piece),
        }
    }

    pub fn map_piece<F, R>(&self, function: F) -> Option<R>
    where
        F: Fn(&Piece, PlayerNumber) -> R
    {
        match self {
            Self::Empty => None,
            Self::PlayerOne(piece) => Some(function(piece, PlayerNumber::PlayerOne)),
            Self::PlayerTwo(piece) => Some(function(piece, PlayerNumber::PlayerTwo)),
            Self::PlayerThree(piece) => Some(function(piece, PlayerNumber::PlayerThree)),
            Self::PlayerFour(piece) => Some(function(piece, PlayerNumber::PlayerFour)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Piece {
    Hovedby(Option<Kartopult>),
    ØdelagtHovedby,
    By(Option<Kartopult>),
    Vej(Option<Kartopult>),
    Mur,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Kartopult {
    Ret,
    Diagonal,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct PieceCounts {
    pub by: usize,
    pub kartopult_ret: usize,
    pub kartopult_diagonal: usize,
    pub vej: usize,
    pub mur: usize,
}

impl PieceCounts {
    pub fn can_build(&self, building: BuildablePiece) -> bool {
        (match building {
            BuildablePiece::By => self.by,
            BuildablePiece::Vej => self.vej,
            BuildablePiece::Mur => self.mur,
            BuildablePiece::Kartopult(Kartopult::Ret) => self.kartopult_ret,
            BuildablePiece::Kartopult(Kartopult::Diagonal) => self.kartopult_diagonal,
        }) >= 1
    }

    pub fn build_piece_unchecked(&mut self, building: BuildablePiece) {
        match building {
            BuildablePiece::By => self.by -= 1,
            BuildablePiece::Vej => self.vej -= 1,
            BuildablePiece::Mur => self.mur -= 1,
            BuildablePiece::Kartopult(Kartopult::Ret) => self.kartopult_ret -= 1,
            BuildablePiece::Kartopult(Kartopult::Diagonal) => self.kartopult_diagonal -= 1,
        }
    }

    // TODO: Make a function that can't panic and is checking if you can build first. Also make an
    // error type for this
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub builds: Vec<Building>,
    pub kartopult_move: Vec<KartopultMove>,
    pub kartopult_shots: Vec<KartopultShot>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub x: usize,
    pub y: usize,
    pub piece: BuildablePiece,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BuildablePiece {
    By,
    Vej,
    Mur,
    Kartopult(Kartopult),
}

impl BuildablePiece {
    pub fn cost(&self) -> usize {
        match self {
            Self::By => 4,
            Self::Vej => 1,
            Self::Mur => 1,
            Self::Kartopult(_) => 3,
        }
    }
}

impl Into<Piece> for BuildablePiece {
    fn into(self) -> Piece {
        match self {
            Self::By => Piece::By(None),
            Self::Vej => Piece::Vej(None),
            Self::Mur => Piece::Mur,
            Self::Kartopult(_) => panic!("can't turn a buildable kartopult into a piece, because you need to build it on another piece"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct KartopultMove {
    pub x: usize,
    pub y: usize,
    pub moves: Vec<DirectionRet>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct KartopultShot {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub power: KartopultPower,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KartopultPower {
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Ret(DirectionDiagonal),
    Diagonal(DirectionRet),
}

pub const ALL_DIRECTION_RET: [DirectionRet; 4] = [
    DirectionRet::Up,
    DirectionRet::Right,
    DirectionRet::Down,
    DirectionRet::Left,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DirectionRet {
    Up,
    Right,
    Down,
    Left,
}

impl DirectionRet {
    pub fn into_vector(&self) -> (isize, isize) {
        match self {
            Self::Up => (0, 1),
            Self::Right => (1, 0),
            Self::Down => (0, -1),
            Self::Left => (-1, 0),
        }
    }
}

pub const ALL_DIRECTION_DIAGONAL: [DirectionDiagonal; 4] = [
    DirectionDiagonal::UpLeft,
    DirectionDiagonal::UpRight,
    DirectionDiagonal::DownRight,
    DirectionDiagonal::DownLeft,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DirectionDiagonal {
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}

impl DirectionDiagonal {
    pub fn into_vector(&self) -> (isize, isize) {
        match self {
            Self::UpLeft => (-1, 1),
            Self::UpRight => (1, 1),
            Self::DownRight => (-1, 1),
            Self::DownLeft => (-1, -1),
        }
    }
}
