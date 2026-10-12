mod menu;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, Event};
use kc_logic::{GameState, PlayerNumber, PlayerStatus, ServerMessage};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{List, ListState},
};
use std::{collections::HashMap, io::BufReader, net::TcpStream};

use crate::menu::{MenuState, MenuWidget, MenuEventMessage, MenuKeyBindConfig};

fn main() {
    let mut model = Model::default();

    ratatui::run(|terminal| model.run(terminal));
}

impl Model {
    fn run(&mut self, terminal: &mut DefaultTerminal) {
        loop {
            terminal.draw(|frame| self.render(frame)).unwrap();

            // Block for update
            'event: {
                let event = crossterm::event::read().unwrap();

                let msg_option = self.handle_event(event);

                if let Some(msg) = msg_option {
                    self.update(msg);
                } else {
                    break 'event;
                }
            }

            if let ModelState::Quit = self.state {
                break;
            }
        }
    }

    fn handle_event(&self, event: Event) -> Option<Message> {
        match event {
            Event::FocusGained
            | Event::FocusLost
            | Event::Mouse(_)
            | Event::Paste(_) => None,
            Event::Resize(_, _) => Some(Message::TermEventMessage(TermEventMessage::Resize)),
            Event::Key(key_event) => {
                let key_bind_config = self.current_key_bind_config();

                key_bind_config.handle_key_bind(&key_event)
                    .map(|term_event_message| Message::TermEventMessage(term_event_message))
            },
        }
    }

    fn current_key_bind_config(&self) -> &impl KeyBindConfig {
        match self.state {
            ModelState::Menu(_) => &self.settings.keybind_manager.menu,
            _ => todo!("the other states aren't written yet"),
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
    keybind_manager: KeyBindManager,
}

#[derive(Debug, Default)]
struct KeyBindManager {
    menu: MenuKeyBindConfig,
    // settings: SettingsKeyBindConfig,
    // lobby: LobbyKeyBindConfig,
    // game: GameKeyBindConfig,
}

// TODO: Add simple error handling inside of this macro so I get errors at compile time
#[macro_export]
macro_rules! keybinds {
    (
        $message_enum:ty {
            $(
                $message_variant:ident => [
                    $($key:tt),* $(,)?
                ]
            ),* $(,)?
        }
    ) => {
        [
            $(
                KeyBinds::new(
                    <$message_enum>::$message_variant,
                    vec![
                        $(
                            keybinds!(@key $key)
                        ),*
                    ],
                )
            ),*
        ]
    };

    (@key ($code:expr)) => {
        KeyEvent::new($code, KeyModifiers::empty())
    };

    (@key ($code:expr, $modifiers:expr)) => {
        KeyEvent::new($code, $modifiers)
    };

    (@key ($code:expr, $modifiers:expr, $kind:expr)) => {
        KeyEvent::new_with_kind($code, $modifiers, $kind)
    };
}

#[derive(Debug)]
struct KeyBinds<M> {
    message: M,
    binds: Vec<KeyEvent>,
}

impl<M> KeyBinds<M> {
    pub fn new(message: M, binds: Vec<KeyEvent>) -> Self {
        Self {
            message,
            binds,
        }
    }
}

trait KeyBindConfig
where Self: Sized {
    type Message;

    fn handle_key_bind(&self, key: &KeyEvent) -> Option<TermEventMessage>;
    fn get_all_binds(&self) -> Vec<KeyBinds<Self::Message>>;
    fn from_binds(binds: &[KeyBinds<Self::Message>]) -> Result<Self, ()>;
}

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

enum Message {
    ServerMessage(ServerMessage),
    TermEventMessage(TermEventMessage),
}

enum TermEventMessage {
    MenuEventMessage(MenuEventMessage),
    Resize,
}

impl Model {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::ServerMessage(msg) => todo!("handle server messages"),
            Message::TermEventMessage(msg) => self.update_term_event(msg),
        }
    }

    fn update_term_event(&mut self, msg: TermEventMessage) {
        match msg {
            TermEventMessage::Resize => (),
            TermEventMessage::MenuEventMessage(msg) => {
                if let ModelState::Menu(menu_state) = self.state {
                    menu_state.update(msg);
                }
            },
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
