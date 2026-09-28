use {
    kc_logic::*,
    smol::{
        io,
        net::{TcpListener, TcpStream},
        prelude::*,
    },
    std::time::{Duration, Instant},
};

struct Connection {
    buf_reader: io::BufReader<TcpStream>,
    stream: TcpStream,
    last_ping: Instant,
    pong_recieved: bool,
}

impl Connection {
    fn new(buf_reader: io::BufReader<TcpStream>, stream: TcpStream) -> Self {
        Self {
            buf_reader,
            stream,
            last_ping: Instant::now(),
            pong_recieved: true,
        }
    }

    async fn ping(&mut self) -> smol::io::Result<()> {
        if !self.pong_recieved {
            let mut buf = String::new();
            self.buf_reader.read_line(&mut buf).await?;

            if buf.contains("PONG") {
                self.pong_recieved = true;
            }

            // TODO: This is probably a message that needs to be used by another part of the program

            return Err(smol::io::Error::new(
                smol::io::ErrorKind::Other,
                "pong is not recieved",
            ));
        }

        // Pong's been recieved since last ping

        let now = Instant::now();
        if now.duration_since(self.last_ping) > Duration::from_secs(10) {
            // Send a ping

            self.stream.write_all(b"PING").await?;

            self.last_ping = Instant::now();

            self.pong_recieved = false;
        }

        Ok(())
    }
}

struct PlayerConnection {
    status: PlayerStatus,
    connection: Connection,
}

struct SpectatorConnection {
    connection: Connection,
}

enum ConnectionType {
    Player(PlayerConnection),
    Spectator(SpectatorConnection),
}

struct ServerLobby {
    players: Vec<PlayerConnection>,
    spectators: Vec<SpectatorConnection>,
    listener: TcpListener,
}

impl ServerLobby {
    async fn new(listener: TcpListener) -> Self {
        let mut lobby = Self {
            players: Vec::new(),
            spectators: Vec::new(),
            listener,
        };

        loop {
            // TODO: Make the parts of this loop a seperate function
            // Handle incoming connections

            if let Ok((stream, _address)) = lobby.listener.accept().await {
                let mut buf_reader = io::BufReader::new(stream.clone());

                let mut buf = String::new();

                let result = buf_reader.read_line(&mut buf).await;

                if let Err(error) = result {
                    eprintln!("{}", error);
                } else {
                    // Succesful read

                    // Check if the buf contains player or spectator

                    let connection = Connection::new(buf_reader, stream);

                    if buf.contains("player") {
                        let status = PlayerStatus {
                            is_ready: false,
                            name: None,
                            preferred_color: Color {
                                red: 0,
                                green: 0,
                                blue: 0,
                            },
                        };

                        let player_connection = PlayerConnection { status, connection };

                        lobby.players.push(player_connection);
                    } else if buf.contains("spectator") {
                        let spectator_connection = SpectatorConnection { connection };

                        lobby.spectators.push(spectator_connection);
                    }
                }
            }

            // Send out heartbeats

            for i in 0..lobby.spectators.len() {
                let spectator_connection = &mut lobby.spectators[i];
                let result = spectator_connection.connection.ping().await;

                if let Err(error) = result {
                    eprintln!("{}", error);

                    match error.kind() {
                        io::ErrorKind::Other => {
                            // Pong not recieved yet, maybe do something here
                        }
                        io::ErrorKind::BrokenPipe
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::NotConnected
                        | io::ErrorKind::WriteZero => {
                            // Connection broken

                            lobby.spectators.remove(i);
                        }
                        _ => (),
                    }
                }
            }

            // Read from every player if they have a new status and send out heartbeats

            let mut status_update = false;

            for i in 0..lobby.players.len() {
                // TODO: WHen we remove elements from the players and spectator vectors, the index
                // will end out of bounds at the end.
                let player_connection = &mut lobby.players[i];
                // If a pong is waiting call ping
                if !player_connection.connection.pong_recieved {
                    let result = player_connection.connection.ping().await;

                    if let Err(error) = result {
                        eprintln!("{}", error);

                        match error.kind() {
                            io::ErrorKind::Other => {
                                // Pong not recieved yet, maybe do something here
                            }
                            io::ErrorKind::BrokenPipe
                            | io::ErrorKind::ConnectionReset
                            | io::ErrorKind::ConnectionAborted
                            | io::ErrorKind::NotConnected
                            | io::ErrorKind::WriteZero => {
                                // Connection broken

                                lobby.players.remove(i);
                                continue;
                            }
                            _ => (),
                        }
                    }
                }

                let mut buf = String::new();
                player_connection
                    .connection
                    .buf_reader
                    .read_line(&mut buf)
                    .await
                    .unwrap();

                let status = serde_json::from_str::<PlayerStatus>(buf.as_str()).unwrap();

                if player_connection.status != status {
                    player_connection.status = status;
                    status_update = true;
                }
            }

            // If any new status we send out a message to everyone
            if status_update {
                let status_json = serde_json::to_vec(
                    &lobby.players
                        .iter()
                        .map(|player| player.status.clone())
                        .collect::<Vec<PlayerStatus>>(),
                )
                .unwrap();

                lobby.write_to_all(&status_json).await.unwrap();
            }

            // If the game is ready to start, break out of the loop
            if lobby.players.len() >= 2
                && lobby.players.len() <= 4
                && lobby.players.iter().all(|player| player.status.is_ready)
            {
                break;
            }
        }

        lobby
    }

    async fn write_to_all(&mut self, buf: &[u8]) -> smol::io::Result<()> {
        for player in &mut self.players {
            player
                .connection
                .stream
                .write_all(buf)
                .await?;
        }

        for spectator in &mut self.spectators {
            spectator
                .connection
                .stream
                .write_all(buf)
                .await?;
        }

        Ok(())
    }
}

struct ServerGame {
    game_state: GameState,
    lobby: ServerLobby,
}

impl ServerGame {
    async fn new(listener: TcpListener) -> Self {
        let lobby = ServerLobby::new(listener).await;

        let game_state = GameState::new(lobby.players.len());

        Self {
            game_state,
            lobby,
        }
    }

    fn current_player_connection(&self) -> &PlayerConnection {
        &self.lobby.players[self.game_state.current_player_number as usize]
    }

    fn current_player_connection_mut(&mut self) -> &mut PlayerConnection {
        &mut self.lobby.players[self.game_state.current_player_number as usize]
    }

    async fn run(&mut self) {
        // Let everybody know the current state of the game

        let game_state_json = serde_json::to_vec(&self.game_state).unwrap();

        self.lobby.write_to_all(&game_state_json).await.unwrap();

        let mut player_move_pending = false;

        loop {
            // Ask the current player for a move.
            if !player_move_pending {
                let current_player = self.current_player_connection_mut();
            }
            // When the player answers validate the move
            // If we can't use it ask again.
            // If it's valid we apply it and send out the new game state to everybody

            // Send out heartbeats.
            break;
        }
    }
}

fn main() {
    smol::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:10799").await?;

        let mut server_game = ServerGame::new(listener).await;

        server_game.run().await;

        Ok::<(), smol::io::Error>(())
    })
    .unwrap();
}
