use {
    serde::{Deserialize, Serialize},
    std::{collections::VecDeque, default::Default},
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub current_player_number: PlayerNumber,
    pub players: [Option<Player>; 4],
    pub board: [[Square; 20]; 20],
}

impl GameState {
    pub fn current_player(&self) -> &Player {
        let current_player = &self.players[self.current_player_number as usize];

        current_player
            .as_ref()
            .expect("the current player is dead and didn't get switched to a new player")
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        let current_player = &mut self.players[self.current_player_number as usize];

        current_player
            .as_mut()
            .expect("the current player is dead and didn't get switched to a new player")
    }

    pub fn next_player(&self) -> PlayerNumber {
        let mut n = (self.current_player_number as usize + 1) % 4;

        // Find the next time an alive player is in the game and return that PlayerNumber
        // corresponding with the index.
        while let None = self.players[n] {
            n += 1;
        }

        PlayerNumber::from(n)
    }

    pub fn inbound_usize(&self, x: usize, y: usize) -> bool {
        x < self.board.len() && y < self.board[0].len()
    }

    pub fn inbound_isize(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.board.len() as isize && y >= 0 && y < self.board[0].len() as isize
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> [Option<(Square, usize, usize)>; 4] {
        ALL_DIRECTION_RET.map(|direction| {
            let vector = direction.into_vector();

            let x = x as isize + vector.0;
            let y = y as isize + vector.1;

            if self.inbound_isize(x, y) {
                return None; // Maybe make another type later, for outofbounds errors and
                // make it a result instead of option
            }

            let (x, y) = (x as usize, y as usize);

            Some((self.board[x][y], x, y))
        })
    }

    pub fn has_neighbor_of_kind<F>(&self, x: usize, y: usize, is_correct_piece: F) -> bool
    where
        F: Fn(&Piece, PlayerNumber) -> bool,
    {
        let neighbor_squares = self.get_neighbors(x, y);

        let mut next_to_correct_piece = false;

        for neighbor in neighbor_squares {
            if let Some(neighbor) = neighbor {
                let is_correct_piece = neighbor.0.map_piece(&is_correct_piece);

                if let Some(is_correct_piece) = is_correct_piece {
                    if is_correct_piece {
                        next_to_correct_piece = true;
                        break;
                    }
                }
            }
        }

        next_to_correct_piece
    }

    pub fn get_hovedbyer(&self) -> [[Option<(usize, usize)>; 4]; 4] {
        let mut hovedby_squares = [[None; 4]; 4];
        let mut n = [0; 4];

        for x in 0..self.board.len() {
            for y in 0..self.board[0].len() {
                let hovedby: Option<(usize, usize, PlayerNumber)> = self.board[x][y]
                    .map_piece(|piece, player| {
                        let is_hovedby = match piece {
                            Piece::Hovedby(_) => true,
                            _ => false,
                        };

                        if !is_hovedby {
                            return None;
                        }

                        Some((x, y, player))
                    })
                    .unwrap_or(None);

                if let Some(hovedby) = hovedby {
                    let player = hovedby.2 as usize;

                    hovedby_squares[player][n[player]] = Some((hovedby.0, hovedby.1));
                    n[player] += 1;
                }
            }
        }

        hovedby_squares
    }

    pub fn potatos_produced(&self) -> usize {
        let mut potatos_produced = 0;

        // Find the hovedby for the current player
        let hovedby_squares = self.get_hovedbyer()[self.current_player_number as usize];

        // Put the squares of the hovedby into a FIFO queue
        let mut queue = VecDeque::<(usize, usize)>::new();

        let mut visited = [[false; 20]; 20]; // 400 BYTES not bits, crazy

        for hovedby in hovedby_squares {
            if let Some(hovedby) = hovedby {
                queue.push_front(hovedby);
            } else {
                break;
            }
        }

        while !queue.is_empty() {
            let square_position = queue.pop_back().unwrap();

            let square = self.board[square_position.0][square_position.1];

            // We can safely assume that this piece is either a Hovedby, By or Vej of the current
            // player, since we only put those pieces into the queue.
            let piece = square.map_piece(|piece, _| piece.clone()).unwrap();

            match piece {
                Piece::Hovedby(_) | Piece::By(_) => {
                    let neighbors = self.get_neighbors(square_position.0, square_position.1);

                    for neighbor in neighbors {
                        // Validate and push to queue
                        if let None = neighbor {
                            continue;
                        }

                        let neighbor = neighbor.unwrap();

                        let is_valid = neighbor
                            .0
                            .map_piece(|piece, player| {
                                if player == self.current_player_number {
                                    match piece {
                                        Piece::Vej(_) => true,
                                        _ => false,
                                    }
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);

                        let is_visited = visited[neighbor.1][neighbor.2];

                        if is_valid && !is_visited {
                            queue.push_front((neighbor.1, neighbor.2));
                        }
                    }

                    potatos_produced += 1;
                }
                Piece::Vej(_) => {
                    let neighbors = self.get_neighbors(square_position.0, square_position.1);

                    for neighbor in neighbors {
                        // Validate and push to queue
                        if let None = neighbor {
                            continue;
                        }

                        let neighbor = neighbor.unwrap();

                        let is_valid = neighbor
                            .0
                            .map_piece(|piece, player| {
                                if player == self.current_player_number {
                                    match piece {
                                        Piece::By(_) | Piece::Vej(_) => true,
                                        _ => false,
                                    }
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);

                        let is_visited = visited[neighbor.1][neighbor.2];

                        if is_valid && !is_visited {
                            queue.push_front((neighbor.1, neighbor.2));
                        }
                    }
                }
                piece => panic!("A wrong piece got put into the BFS queue: {:?}", piece),
            }

            // Mark as visited
            visited[square_position.0][square_position.1] = true;
        }

        potatos_produced
    }

    pub fn build(&mut self, building: Building) -> Result<(), ()> {
        // Check that the player has enough potatos
        if self.current_player().potato_count < building.piece.cost() {
            return Err(());
        }

        // Check that the player has enough pieces
        if !self.current_player().piece_counts.can_build(building.piece) {
            return Err(());
        }

        // Check that the building position is inside the boards bounds
        if self.inbound_usize(building.x, building.y) {
            return Err(());
        };

        match building.piece {
            BuildablePiece::By => {
                let next_to_correct_piece =
                    self.has_neighbor_of_kind(building.x, building.y, |piece, player| {
                        if player == self.current_player_number {
                            match piece {
                                Piece::Vej(_) => true,
                                _ => false,
                            }
                        } else {
                            false
                        }
                    });

                if !next_to_correct_piece || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] =
                    Square::from_player_number(self.current_player_number, building.piece.into())
            }
            BuildablePiece::Vej => {
                let next_to_correct_piece =
                    self.has_neighbor_of_kind(building.x, building.y, |piece, player| {
                        if player == self.current_player_number {
                            match piece {
                                Piece::Vej(_) | Piece::By(_) | Piece::Hovedby(_) => true,
                                _ => false,
                            }
                        } else {
                            false
                        }
                    });

                if !next_to_correct_piece || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] =
                    Square::from_player_number(self.current_player_number, building.piece.into())
            }
            BuildablePiece::Mur => {
                let next_to_correct_piece =
                    self.has_neighbor_of_kind(building.x, building.y, |piece, player| {
                        if player == self.current_player_number {
                            match piece {
                                Piece::Vej(_) | Piece::By(_) | Piece::Hovedby(_) | Piece::Mur => {
                                    true
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    });

                if !next_to_correct_piece || self.board[building.x][building.y] != Square::Empty {
                    return Err(());
                }

                // Build
                self.board[building.x][building.y] =
                    Square::from_player_number(self.current_player_number, building.piece.into())
            }
            BuildablePiece::Kartopult(kartopult) => {
                let new_piece = self.board[building.x][building.y]
                    .map_piece(|piece, player| {
                        if player == self.current_player_number {
                            match piece {
                                Piece::By(None) => Some(Piece::By(Some(kartopult))),
                                Piece::Hovedby(None) => Some(Piece::Hovedby(Some(kartopult))),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap_or(None);

                if let Some(piece) = new_piece {
                    // Build
                    self.board[building.x][building.y] =
                        Square::from_player_number(self.current_player_number, piece);
                } else {
                    // incompatible piece for kartopult building
                    return Err(());
                }
            }
        };

        // Remove the potatos from the player
        let current_player = self.current_player_mut();

        current_player.potato_count -= building.piece.cost();

        // Remove the pieces from the player
        // This function is unchecked since we already checked before
        current_player
            .piece_counts
            .build_piece_unchecked(building.piece);

        Ok(())
    }

    pub fn move_kartopult(&mut self, kartopult_move: KartopultMove) -> Result<(), ()> {
        let cost = kartopult_move.moves.len();

        // Check if we have enough potatos
        if self.current_player().potato_count < cost {
            return Err(());
        }

        if !self.inbound_usize(kartopult_move.x, kartopult_move.y) {
            return Err(());
        }

        // If no kartopult is at the start square, return error
        let start_piece = self.board[kartopult_move.x][kartopult_move.y]
            .has_kartopult(self.current_player_number);
        if start_piece.is_none() {
            return Err(());
        }

        let kartopult = match start_piece.unwrap() {
            Piece::Hovedby(Some(kartopult))
            | Piece::By(Some(kartopult))
            | Piece::Vej(Some(kartopult)) => kartopult,
            _ => panic!("Should be impossible to get here"),
        };

        let mut x = kartopult_move.x as isize;
        let mut y = kartopult_move.y as isize;

        for direction in kartopult_move.moves {
            // Move to new square
            let vector = direction.into_vector();

            x += vector.0;
            y += vector.1;

            if !self.inbound_isize(x, y) {
                return Err(());
            }

            // Check square
            let square = self.board[x as usize][y as usize];

            if square
                .is_kartopult_friendly(self.current_player_number)
                .is_none()
            {
                return Err(());
            }
        }

        // Move the kartopult to the new place

        // Place kartopult at end square
        let (x, y) = (x as usize, y as usize);
        self.board[x][y] = Square::from_player_number(
            self.current_player_number,
            self.board[x][y]
                .is_kartopult_friendly(self.current_player_number)
                .unwrap()
                .place_kartopult(kartopult)
                .unwrap(),
        );

        // Remove kartopult at start square
        self.board[kartopult_move.x][kartopult_move.y] = Square::from_player_number(
            self.current_player_number,
            self.board[kartopult_move.x][kartopult_move.y]
                .has_kartopult(self.current_player_number)
                .unwrap()
                .remove_kartopult()
                .unwrap(),
        );

        // Remove the potatos from the player
        self.current_player_mut().potato_count -= cost;

        Ok(())
    }

    pub fn shoot_kartopult(&mut self, kartopult_shot: KartopultShot) -> Result<(), ()> {
        // Check if player has enough potatos
        let cost = kartopult_shot.power as usize;

        if self.current_player().potato_count < cost {
            return Err(());
        }

        // Check that the position has a kartopult
        let kartopult = self.board[kartopult_shot.x][kartopult_shot.y]
            .has_kartopult(self.current_player_number);
        if kartopult.is_none() {
            return Err(());
        }

        let kartopult = match kartopult.unwrap() {
            Piece::Hovedby(Some(kartopult))
            | Piece::By(Some(kartopult))
            | Piece::Vej(Some(kartopult)) => kartopult,
            _ => panic!("Should be impossible to get here"),
        };

        // Check that the kartopult and kartopult shot is the same direction
        let is_same_direction = match kartopult_shot.direction {
            Direction::Ret(_) => kartopult == Kartopult::Ret,
            Direction::Diagonal(_) => kartopult == Kartopult::Diagonal,
        };

        if !is_same_direction {
            return Err(());
        }

        let vector = kartopult_shot.direction.into_vector();

        // Check the squares between the kartopult and the target are valid (no walls)
        for power in 1..kartopult_shot.power as usize {
            let x = kartopult_shot.x as isize + power as isize * vector.0;
            let y = kartopult_shot.y as isize + power as isize * vector.1;

            if !self.inbound_isize(x, y) {
                return Err(());
            }

            let (x, y) = (x as usize, y as usize);

            let square = self.board[x][y];

            let is_wall = square
                .map_piece(|piece, _| *piece == Piece::Mur)
                .unwrap_or(false);

            if is_wall {
                return Err(());
            }
        }

        // Check that the target is valid
        let target_x = kartopult_shot.x as isize + kartopult_shot.power as isize * vector.0;
        let target_y = kartopult_shot.y as isize + kartopult_shot.power as isize * vector.1;
        if !self.inbound_isize(target_x, target_y) {
            return Err(());
        }

        let target_square = &mut self.board[target_x as usize][target_y as usize];

        let new_square: Option<Square> = target_square
            .map_piece(|piece, player| {
                if player == self.current_player_number {
                    None
                } else {
                    match piece {
                        Piece::Hovedby(_) => {
                            Some(Square::from_player_number(player, Piece::ØdelagtHovedby))
                        }
                        Piece::By(_) | Piece::Vej(_) | Piece::Mur => Some(Square::Empty),
                        Piece::ØdelagtHovedby => None,
                    }
                }
            })
            .unwrap_or(None);

        if new_square.is_none() {
            return Err(());
        }

        // Update the target square
        *target_square = new_square.unwrap();

        // Remove potatos
        self.current_player_mut().potato_count -= cost;

        Ok(())
    }

    pub fn apply_round(&self, round: Round) -> Result<Self, ()> {
        let mut game_state = self.clone();

        // Get potatos
        let potatoes_produced = self.potatos_produced();

        game_state.current_player_mut().potato_count += potatoes_produced;

        // Build
        for building in round.builds {
            let result = game_state.build(building);

            if result.is_err() {
                return Err(());
            }
        }

        // Move kartopults
        for kartopult_move in round.kartopult_moves {
            let result = game_state.move_kartopult(kartopult_move);

            if result.is_err() {
                return Err(());
            }
        }

        // Shoot kartopults
        for kartopult_shot in round.kartopult_shots {
            let result = game_state.shoot_kartopult(kartopult_shot);

            if result.is_err() {
                return Err(());
            }
        }

        // Check if any player dies
        let hovedbyer = game_state.get_hovedbyer();

        for (i, players_hovedby) in hovedbyer.iter().enumerate() {
            let mut is_alive = false;

            for hovedby_square in players_hovedby {
                if hovedby_square.is_some() {
                    is_alive = true;
                }
            }

            if !is_alive {
                game_state.players[i] = None;
            }
        }

        // Shift the current player
        game_state.current_player_number = game_state.next_player();

        Ok(game_state)
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

impl From<usize> for PlayerNumber {
    fn from(value: usize) -> Self {
        match value {
            0 => PlayerNumber::PlayerOne,
            1 => PlayerNumber::PlayerTwo,
            2 => PlayerNumber::PlayerThree,
            3 => PlayerNumber::PlayerFour,
            x => panic!("{} is too big to turn into a PlayerNumber", x),
        }
    }
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
    pub fn from_player_number(player: PlayerNumber, piece: Piece) -> Self {
        match player {
            PlayerNumber::PlayerOne => Square::PlayerOne(piece),
            PlayerNumber::PlayerTwo => Square::PlayerTwo(piece),
            PlayerNumber::PlayerThree => Square::PlayerThree(piece),
            PlayerNumber::PlayerFour => Square::PlayerFour(piece),
        }
    }

    pub fn map_piece<F, R>(&self, function: F) -> Option<R>
    where
        F: Fn(&Piece, PlayerNumber) -> R,
    {
        match self {
            Self::Empty => None,
            Self::PlayerOne(piece) => Some(function(piece, PlayerNumber::PlayerOne)),
            Self::PlayerTwo(piece) => Some(function(piece, PlayerNumber::PlayerTwo)),
            Self::PlayerThree(piece) => Some(function(piece, PlayerNumber::PlayerThree)),
            Self::PlayerFour(piece) => Some(function(piece, PlayerNumber::PlayerFour)),
        }
    }

    pub fn is_kartopult_friendly(&self, kartopult_player: PlayerNumber) -> Option<Piece> {
        self.map_piece(|piece, square_player| {
            if square_player == kartopult_player {
                match piece {
                    Piece::By(None) | Piece::Hovedby(None) | Piece::Vej(None) => {
                        Some(piece.clone())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or(None)
    }

    pub fn has_kartopult(&self, kartopult_player: PlayerNumber) -> Option<Piece> {
        self.map_piece(|piece, square_player| {
            if square_player == kartopult_player {
                match piece {
                    Piece::By(Some(_)) | Piece::Hovedby(Some(_)) | Piece::Vej(Some(_)) => {
                        Some(piece.clone())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or(None)
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

impl Piece {
    pub fn place_kartopult(&self, kartopult: Kartopult) -> Option<Piece> {
        match self {
            Self::Hovedby(None) => Some(Self::Hovedby(Some(kartopult))),
            Self::By(None) => Some(Self::By(Some(kartopult))),
            Self::Vej(None) => Some(Self::Vej(Some(kartopult))),
            _ => None,
        }
    }

    pub fn remove_kartopult(&self) -> Option<Piece> {
        match self {
            Self::Hovedby(Some(_)) => Some(Self::Hovedby(None)),
            Self::By(Some(_)) => Some(Self::By(None)),
            Self::Vej(Some(_)) => Some(Self::Vej(None)),
            _ => None,
        }
    }
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
    pub kartopult_moves: Vec<KartopultMove>,
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
            Self::Kartopult(_) => panic!(
                "can't turn a buildable kartopult into a piece, because you need to build it on another piece"
            ),
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
    Ret(DirectionRet),
    Diagonal(DirectionDiagonal),
}

impl Direction {
    pub fn into_vector(&self) -> (isize, isize) {
        match self {
            Self::Ret(direction) => direction.into_vector(),
            Self::Diagonal(direction) => direction.into_vector(),
        }
    }
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
