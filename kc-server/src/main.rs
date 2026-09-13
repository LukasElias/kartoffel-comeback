use {
    kc_logic::*,
    std::{
        io::{BufRead, BufReader, BufWriter, Write},
        net::{TcpListener, TcpStream},
        time::Duration,
    },
};

enum ConnectionType {
    PlayerConnection(PlayerConnectionLobby),
    SpectatorConnection(SpectatorConnection),
}

struct PlayerConnectionLobby {
    is_ready: bool,
    preferred_color: Color,
    name: Option<String>,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl From<&PlayerConnectionLobby> for PlayerStatus {
    fn from(value: &PlayerConnectionLobby) -> Self {
        Self {
            is_ready: value.is_ready,
            preferred_color: value.preferred_color,
            name: value.name.clone(),
        }
    }
}

struct PlayerConnection {
    preferred_color: Color,
    name: Option<String>,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl From<PlayerConnectionLobby> for PlayerConnection {
    fn from(value: PlayerConnectionLobby) -> Self {
        Self {
            preferred_color: value.preferred_color,
            name: value.name,
            reader: value.reader,
            writer: value.writer,
        }
    }
}

struct SpectatorConnection {
    writer: BufWriter<TcpStream>,
}

struct ServerGameBuilder {
    players: Vec<PlayerConnectionLobby>,
    spectators: Vec<SpectatorConnection>,
}

impl ServerGameBuilder {
    fn new() -> Self {
        let server_game_builder = Self {
            players: Vec::new(),
            spectators: Vec::new(),
        };

        server_game_builder
    }

    fn write_to_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        for player_i in 0..self.players.len() {
            let writer = &mut self.players[player_i].writer;

            writer.write_all(buf)?;
            writer.flush()?;
        }

        for spectator_i in 0..self.spectators.len() {
            let writer = &mut self.players[spectator_i].writer;

            writer.write_all(buf)?;
            writer.flush()?;
        }

        Ok(())
    }

    fn new_player(&mut self, player: PlayerConnectionLobby) -> Result<(), ()> {
        if self.players.len() > 4 {
            return Err(());
        }

        self.players.push(player);

        Ok(())
    }

    fn listen(mut self, listener: TcpListener) -> Self {
        listener
            .set_nonblocking(true)
            .expect("couldn't set tcp_listener to non-blocking");

        println!("Lobby open! Waiting for players...");

        loop {
            if let Ok((stream, _)) = listener.accept() {
                let connection_type = self.handle_connection(stream);

                match connection_type {
                    Err(e) => eprintln!("{}", e),
                    Ok(connection_type) => {
                        match connection_type {
                            ConnectionType::PlayerConnection(player_connection) => {
                                self.new_player(player_connection).unwrap_or(())
                            } // TODO: Don't just ignore ig. Do something smart future me
                            ConnectionType::SpectatorConnection(spectator_connection) => {
                                self.spectators.push(spectator_connection)
                            }
                        }
                    }
                }
            }

            // write to all players and spectators
            let players: Vec<PlayerStatus> = self
                .players
                .iter()
                .map(|player| PlayerStatus::from(player))
                .collect();
            let buf = serde_json::to_string(&players).expect("couldn't serialize to json");
            let result = self.write_to_all(buf.as_bytes());

            if let Err(error) = result {
                eprintln!("{}", error);
            }

            // read from all players if they changed something like they're color or ready up status
            for i in 0..self.players.len() {
                let mut buf = String::new();
                let result = self.players[i].reader.read_line(&mut buf);

                if result.is_err() {
                    continue;
                }

                let player_status = serde_json::from_str::<PlayerStatus>(buf.as_str());

                if player_status.is_err() {
                    continue;
                }

                let player_status = player_status.unwrap();

                self.players[i].is_ready = player_status.is_ready;
            }

            // check if players are ready and enough players are here
            if self.players.len() >= 2
                && self.players.len() <= 4
                && self.players.iter().all(|player| player.is_ready)
            {
                break;
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        self
    }

    fn handle_connection(
        &mut self,
        connection: TcpStream,
    ) -> Result<ConnectionType, std::io::Error> {
        // Split the connection into a reader and a writer
        let mut reader = BufReader::new(connection.try_clone()?);
        let writer = BufWriter::new(connection.try_clone()?);

        // Read a line and determine if the connection is a player or spectator.
        // This blocks, so if the player never responds it will be quite bad.
        // I suppose I gotta set a time limit or something.
        let mut buf = String::new();
        reader.read_line(&mut buf)?;

        connection.set_nonblocking(true)?;

        let connection_type = match buf.as_str() {
            "player" => ConnectionType::PlayerConnection(PlayerConnectionLobby {
                is_ready: false,
                // TODO: Make a better default color for the players
                preferred_color: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
                name: None,
                reader,
                writer,
            }),
            "spectator" => ConnectionType::SpectatorConnection(SpectatorConnection { writer }),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "the client didn't specify if they wanted to be a player or spectator",
                ));
            }
        };

        Ok(connection_type)
    }

    fn build(self) -> ServerGame {
        let game_state = GameState::new(self.players.len());

        let players = self
            .players
            .into_iter()
            .map(PlayerConnection::from)
            .collect();

        ServerGame {
            game_state,
            players,
            spectators: self.spectators,
        }
    }
}

struct ServerGame {
    game_state: GameState,
    players: Vec<PlayerConnection>,
    spectators: Vec<SpectatorConnection>,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:10799").expect("Couldn't start a server");

    let mut server_game = ServerGameBuilder::new().listen(listener).build();
}
