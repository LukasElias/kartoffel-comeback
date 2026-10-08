mod menu;

use kc_logic::{GameState, PlayerNumber, PlayerStatus, ServerMessage};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{List, ListState},
};
use std::{io::BufReader, net::TcpStream, thread::sleep, time::Duration};

use crate::menu::{MenuState, MenuWidget};

fn main() {
    let mut model = Model::default();

    ratatui::run(|terminal| model.run(terminal));
}

impl Model {
    fn run(&mut self, terminal: &mut DefaultTerminal) {
        loop {
            terminal.draw(|frame| self.render(frame)).unwrap();

            sleep(Duration::from_secs(10));

            // Block for update
            let update = Update::Quit;

            self.update(update);

            if let ModelState::Quit = self.state {
                break;
            }
        }
    }
}

// TEA - The Elm Architecture

// Model

#[derive(Debug, Default)]
struct Model {
    state: ModelState,
    settings: Settings,
}

#[derive(Debug, Default)]
struct Settings {
    key_bind_manager: KeyBindManager,
}

#[derive(Debug, Default)]
struct KeyBindManager {}

#[derive(Debug)]
enum ModelState {
    Menu(menu::MenuState),
    Settings,
    ConnectedToLobby {
        player_status_list: Vec<PlayerStatus>,
        connection: Connection,
    },
    ConnectedToGame {
        game_state: GameState,
        player_status_list: Vec<PlayerStatus>,
        connection: Connection,
    },
    ServerDisconnected(String),
    Quit,
}

#[derive(Debug)]
struct Connection {
    connected_as: ConnectionType,
    buf_reader: BufReader<TcpStream>,
    stream: TcpStream,
}

#[derive(Debug, Clone)]
enum ConnectionType {
    Spectator,
    Player(PlayerNumber),
}

impl Default for ModelState {
    fn default() -> Self {
        Self::Menu(MenuState::default())
    }
}

// Update

enum Update {
    ServerMessage(ServerMessage),
    KeyBindAction(KeyBindAction),
    Quit,
}

enum KeyBindAction {}

impl Model {
    fn update(&mut self, update: Update) {
        match update {
            Update::ServerMessage(msg) => {}
            Update::KeyBindAction(action) => {}
            Update::Quit => self.state = ModelState::Quit,
        }
    }
}

// View

impl Model {
    // This has to be mutable since we have a state that can change when passed into the render
    // method when using StatefulWidget
    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);

        let rects = layout.split(frame.area());

        let (main_rect, hint_rect) = (rects[0], rects[1]);

        match &mut self.state {
            ModelState::Menu(state) => {
                MenuWidget::new().render(main_rect, frame.buffer_mut(), state)
            }
            _ => {}
        }

        // Draw the bottom hint box
    }
}
