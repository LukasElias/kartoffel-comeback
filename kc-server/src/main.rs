use {
    kc_logic::*, std::{
        io::{
            self, BufRead, BufReader, Write,
        }, net::{
            TcpListener,
            TcpStream,
        }, time::{Duration, Instant},
    },
};

struct Connection {
    buf_reader: BufReader<TcpStream>,
    stream: TcpStream,
    last_ping: Instant,
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

fn main() {
    let listener = TcpListener::bind("127.0.0.1:10799").expect("Couldn't start a server");

    let mut server_game = ServerGame::new(listener);
}
