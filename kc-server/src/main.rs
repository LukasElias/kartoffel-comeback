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
    fn new(stream: TcpStream) -> Self {
        let buf_reader = io::BufReader::new(stream.clone());

        Self {
            buf_reader,
            stream,
            last_ping: Instant::now(),
            pong_recieved: true,
        }
    }

    async fn ping(&mut self) -> smol::io::Result<()> {
        if !self.pong_recieved {
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

    async fn client_messages(&mut self) -> Vec<ClientMessage> {
        let mut buf = String::new();

        let result = self.buf_reader.read_to_string(&mut buf).await;

        if let Err(error) = result {
            eprintln!("{}", error);
        }

        buf.split("\n")
            .filter_map(|line| serde_json::from_str::<ClientMessage>(line).ok())
            .collect()
    }
}

struct PlayerConnection {
    status: PlayerStatus,
    connection: Connection,
}

struct SpectatorConnection {
    connection: Connection,
}

struct ServerLobby {
    players: Vec<PlayerConnection>,
    spectators: Vec<SpectatorConnection>,
    listener: TcpListener,
}

impl ServerLobby {
    async fn handle_incoming_connection(&mut self) {
        if let Ok((stream, _address)) = self.listener.accept().await {
            let connection = Connection::new(stream);

            let spectator_connection = SpectatorConnection { connection };

            self.spectators.push(spectator_connection);
        }
    }

    fn downgrade_player(&mut self, idx: usize) {
        let player = self.players.remove(idx);

        let spectator = SpectatorConnection {
            connection: player.connection,
        };

        self.spectators.push(spectator);
    }

    fn upgrade_spectator(&mut self, idx: usize, status: PlayerStatus) {
        let spectator = self.spectators.remove(idx);

        let player = PlayerConnection {
            status,
            connection: spectator.connection,
        };

        self.players.push(player);
    }

    async fn new(listener: TcpListener) -> Self {
        let mut lobby = Self {
            players: Vec::new(),
            spectators: Vec::new(),
            listener,
        };

        loop {
            // TODO: Make the parts of this loop a seperate function
            // Handle incoming connections as spectators

            lobby.handle_incoming_connection().await;

            let mut status_update = false;

            // Read all the ClientMessage enums sent from the clients and handle them
            for idx in 0..lobby.players.len() {
                let client_messages = lobby.players[idx].connection.client_messages().await;

                for message in client_messages {
                    match message {
                        ClientMessage::Pong => lobby.players[idx].connection.pong_recieved = true,
                        ClientMessage::Round(_round) => (), // ignore since the game's not started
                        ClientMessage::Status(player_status) => {
                            lobby.players[idx].status = player_status;
                            status_update = true;
                        }
                        ClientMessage::Downgrade => lobby.downgrade_player(idx),
                        ClientMessage::Upgrade(_player_status) => (), //ignore here since it's a
                        //player not a spectator
                        ClientMessage::Disconnected => {
                            lobby.players.remove(idx);
                        }
                    }
                }

                // Do a ping
                let result = lobby.players[idx].connection.ping().await;

                if let Err(error) = result {
                    eprintln!("{}", error);
                }
            }

            for idx in 0..lobby.spectators.len() {
                let client_messages = lobby.spectators[idx].connection.client_messages().await;

                for message in client_messages {
                    match message {
                        ClientMessage::Pong => {
                            lobby.spectators[idx].connection.pong_recieved = true
                        }
                        ClientMessage::Round(_round) => (), // ignore since the spectators don't
                        // send rounds
                        ClientMessage::Status(_player_status) => (), // ignore since it's a spectator
                        ClientMessage::Downgrade => (), // ignore because it's a spectator
                        ClientMessage::Upgrade(player_status) => {
                            lobby.upgrade_spectator(idx, player_status)
                        }
                        ClientMessage::Disconnected => {
                            lobby.spectators.remove(idx);
                        }
                    }
                }

                // Do a ping
                let result = lobby.spectators[idx].connection.ping().await;

                if let Err(error) = result {
                    eprintln!("{}", error);
                }
            }

            // If any new status we send out a message to everyone
            if status_update {
                let player_status = lobby
                    .players
                    .iter()
                    .map(|player| player.status.clone())
                    .collect::<Vec<PlayerStatus>>();

                // send out to spectators
                let buf = serde_json::to_vec(&ServerMessage::StatusUpdateSpectator(
                    player_status.clone(),
                ))
                .unwrap();
                lobby.write_to_spectators(&buf).await.unwrap();

                // write to players
                for idx in 0..lobby.players.len() {
                    let player_number = PlayerNumber::from(idx);

                    let buf = serde_json::to_vec(&ServerMessage::StatusUpdatePlayer(
                        player_status.clone(),
                        player_number,
                    ))
                    .unwrap();
                    lobby.players[idx]
                        .connection
                        .stream
                        .write_all(&buf)
                        .await
                        .unwrap();
                }
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

    async fn write_to_players(&mut self, buf: &[u8]) -> smol::io::Result<()> {
        // TODO: When writing here, I gotta make sure to not return an error if one connection fails,
        // since I should then just handle that one error here, and if a player disconnects etc...
        // I gotta handle that gracefully.

        for player in &mut self.players {
            player.connection.stream.write_all(buf).await?;
        }

        Ok(())
    }

    async fn write_to_spectators(&mut self, buf: &[u8]) -> smol::io::Result<()> {
        // TODO: When writing here, I gotta make sure to not return an error if one connection fails,
        // since I should then just handle that one error here, and if a player disconnects etc...
        // I gotta handle that gracefully.

        for spectator in &mut self.spectators {
            spectator.connection.stream.write_all(buf).await?;
        }

        Ok(())
    }

    async fn write_to_all(&mut self, buf: &[u8]) -> smol::io::Result<()> {
        self.write_to_players(buf).await?;
        self.write_to_spectators(buf).await?;

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

        Self { game_state, lobby }
    }

    fn current_player_connection(&self) -> &PlayerConnection {
        &self.lobby.players[self.game_state.current_player_number as usize]
    }

    fn current_player_connection_mut(&mut self) -> &mut PlayerConnection {
        &mut self.lobby.players[self.game_state.current_player_number as usize]
    }

    async fn apply_round(&mut self, round: Round) -> bool {
        let game_state_result = self.game_state.apply_round(round);

        match game_state_result {
            Err(error) => {
                eprintln!("{}", error);

                // Ask for a round again

                let buf = serde_json::to_vec(&ServerMessage::YourTurn).unwrap();
                self.current_player_connection_mut()
                    .connection
                    .stream
                    .write_all(&buf)
                    .await
                    .unwrap();
            }
            Ok((game_state, did_win)) => {
                let player_that_won = match did_win {
                    true => Some(game_state.current_player_number),
                    false => None,
                };

                let buf =
                    serde_json::to_vec(&ServerMessage::GameState(game_state, player_that_won))
                        .unwrap();
                self.lobby.write_to_all(&buf).await.unwrap();

                if !did_win {
                    let buf = serde_json::to_vec(&ServerMessage::YourTurn).unwrap();
                    self.current_player_connection_mut()
                        .connection
                        .stream
                        .write_all(&buf)
                        .await
                        .unwrap();
                }

                return did_win;
            }
        }

        false
    }

    async fn run(&mut self) {
        // Let everybody know the current state of the game

        let game_state_json = serde_json::to_vec(&self.game_state).unwrap();

        self.lobby.write_to_all(&game_state_json).await.unwrap();

        // Tell the current player it's their turn
        let current_player = self.current_player_connection_mut();
        let buf = serde_json::to_vec(&ServerMessage::YourTurn).unwrap();
        current_player
            .connection
            .stream
            .write_all(&buf)
            .await
            .unwrap();

        loop {
            // Handle incoming messages and send out pings
            for idx in 0..self.lobby.players.len() {
                let client_messages = self.lobby.players[idx].connection.client_messages().await;

                for message in client_messages {
                    match message {
                        ClientMessage::Pong => {
                            self.lobby.players[idx].connection.pong_recieved = true
                        }
                        ClientMessage::Round(round) => {
                            if PlayerNumber::from(idx) == self.game_state.current_player_number {
                                if self.apply_round(round).await {
                                    // a player won
                                    return;
                                }
                            }
                        }
                        ClientMessage::Status(_player_status) => (), // ignore since game's started
                        ClientMessage::Downgrade => (), // ignore since the game's started
                        ClientMessage::Upgrade(_player_status) => (), //ignore here since it's a
                        //player not a spectator
                        ClientMessage::Disconnected => {
                            self.lobby.players.remove(idx);
                            todo!("write some disconnect logic when the game is running");
                        }
                    }
                }

                // Do a ping
                let result = self.lobby.players[idx].connection.ping().await;

                if let Err(error) = result {
                    eprintln!("{}", error);
                }
            }

            for idx in 0..self.lobby.spectators.len() {
                let client_messages = self.lobby.spectators[idx]
                    .connection
                    .client_messages()
                    .await;

                for message in client_messages {
                    match message {
                        ClientMessage::Pong => {
                            self.lobby.spectators[idx].connection.pong_recieved = true
                        }
                        ClientMessage::Round(_round) => (), // ignore since the spectators don't
                        // send rounds
                        ClientMessage::Status(_player_status) => (), // ignore since it's a spectator
                        ClientMessage::Downgrade => (), // ignore because it's a spectator
                        ClientMessage::Upgrade(_player_status) => (), // ignore since the game's
                        // started
                        ClientMessage::Disconnected => {
                            self.lobby.spectators.remove(idx);
                        }
                    }
                }

                // Do a ping
                let result = self.lobby.spectators[idx].connection.ping().await;

                if let Err(error) = result {
                    eprintln!("{}", error);
                }
            }

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
