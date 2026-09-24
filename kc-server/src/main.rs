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

struct ServerGame {
    game_state: GameState,
    players: Vec<PlayerConnection>,
    spectators: Vec<SpectatorConnection>,
}

impl ServerGame {
    async fn new(listener: TcpListener) -> Self {
        let mut players = Vec::new();
        let mut spectators = Vec::new();

        loop {
            // TODO: Make the parts of this loop a seperate function
            // Handle incoming connections

            if let Ok((stream, _address)) = listener.accept().await {
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

                        players.push(player_connection);
                    } else if buf.contains("spectator") {
                        let spectator_connection = SpectatorConnection { connection };

                        spectators.push(spectator_connection);
                    }
                }
            }

            // Send out heartbeats

            for i in 0..spectators.len() {
                let spectator_connection = &mut spectators[i];
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

                            spectators.remove(i);
                        }
                        _ => (),
                    }
                }
            }

            // Read from every player if they have a new status and send out heartbeats

            let mut status_update = false;

            for i in 0..players.len() {
                // TODO: WHen we remove elements from the players and spectator vectors, the index
                // will end out of bounds at the end.
                let player_connection = &mut players[i];
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

                                players.remove(i);
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
                    &players
                        .iter()
                        .map(|player| player.status.clone())
                        .collect::<Vec<PlayerStatus>>(),
                )
                .unwrap();

                for player in &mut players {
                    player
                        .connection
                        .stream
                        .write_all(&status_json)
                        .await
                        .unwrap();
                }

                for spectator in &mut spectators {
                    spectator
                        .connection
                        .stream
                        .write_all(&status_json)
                        .await
                        .unwrap();
                }
            }

            // If the game is ready to start, break out of the loop
            if players.len() >= 2
                && players.len() <= 4
                && players.iter().all(|player| player.status.is_ready)
            {
                break;
            }
        }

        let game_state = GameState::new(players.len());

        Self {
            game_state,
            players,
            spectators,
        }
    }
}

fn main() {
    smol::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:10799").await?;

        let mut server_game = ServerGame::new(listener);

        Ok::<(), smol::io::Error>(())
    })
    .unwrap();
}
