use crate::{Buffer, Constraint, HashMap, KeyBindConfig, KeyBinds, KeyCode, KeyEvent, KeyModifiers, List, ListState, Rect, StatefulWidget, Style, TermEventMessage, keybinds};

// Model

#[derive(Debug, Copy, Clone)]
pub struct MenuState {
    list_state: ListState,
}

impl Default for MenuState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select_first();

        Self { list_state }
    }
}

// Update

impl MenuState {
    pub fn update(&mut self, msg: MenuEventMessage) {
        match msg {
            MenuEventMessage::Up => self.list_state.select_previous(),
            MenuEventMessage::Down => self.list_state.select_next(),
            MenuEventMessage::SelectMenuItem => {},
        }
    }
}

const VALID_MESSAGES: [MenuEventMessage; 3] = [
    MenuEventMessage::Up,
    MenuEventMessage::Down,
    MenuEventMessage::SelectMenuItem,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MenuEventMessage {
    Up,
    Down,
    SelectMenuItem,
}

#[derive(Debug)]
pub struct MenuKeyBindConfig {
    binds: HashMap<KeyEvent, <MenuKeyBindConfig as KeyBindConfig>::Message>,
}

impl Default for MenuKeyBindConfig {
    fn default() -> Self {
        let binds = keybinds!(
            <Self as KeyBindConfig>::Message {
                Up => [
                    (KeyCode::Up),
                    (KeyCode::Char('k')),
                ],
                Down => [
                    (KeyCode::Down),
                    (KeyCode::Char('j')),
                ],
                SelectMenuItem => [
                    (KeyCode::Enter),
                    (KeyCode::Char(' ')),
                ],
            }
        );

        println!("{:?}", binds);

        Self::from_binds(&binds).expect("default binds are invalid")
    }
}

impl KeyBindConfig for MenuKeyBindConfig {
    type Message = MenuEventMessage;

    fn handle_key_bind(&self, key: &KeyEvent) -> Option<TermEventMessage> {
        self.binds.get(key).copied()
            .map(|menu_event| TermEventMessage::MenuEventMessage(menu_event))
    }

    fn get_all_binds(&self) -> Vec<KeyBinds<Self::Message>> {
        let mut message_binds = HashMap::<Self::Message, Vec<KeyEvent>>::new();

        for (key_event, message) in &self.binds {
            message_binds.entry(*message)
                .and_modify(|binds| binds.push(*key_event))
                .or_insert(vec![*key_event]);
        }

        message_binds
            .into_iter()
            .map(|(message, binds)| KeyBinds::new(message, binds))
            .collect()
    }

    fn from_binds(binds: &[KeyBinds<Self::Message>]) -> Result<Self, ()> {
        let mut hash_binds = HashMap::new();

        // Validate that all the Message variants have atleast one bind
        // Validate that no duplicate KeyEvents exists

        let mut valid_messages = Vec::from(VALID_MESSAGES);

        for key_binds in binds {
            for i in 0..valid_messages.len() {
                if valid_messages[i] == key_binds.message {
                    valid_messages.remove(i);
                    break;
                }

                // The message variant was invalid
                if i == valid_messages.len() - 1 {
                    return Err(());
                }
            };

            for bind in &key_binds.binds {
                let bind_exists = hash_binds.insert(*bind, key_binds.message);

                // A single KeyEvent was present twice
                if let Some(_old_map_value) = bind_exists {
                    return Err(());
                }
            }
        }

        if !valid_messages.is_empty() {
            return Err(());
        }

        Ok(Self {
            binds: hash_binds,
        })
    }
}

// View

#[derive(Debug, Copy, Clone)]
pub struct MenuWidget;

impl MenuWidget {
    pub fn new() -> Self {
        Self
    }
}

impl StatefulWidget for MenuWidget {
    type State = MenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let items = ["Start Game", "Settings", "Quit"];

        let highlight_style = Style::new().on_red();

        let list = List::new(items)
            // .style(style)
            .highlight_style(highlight_style)
            .highlight_symbol(">");

        let width = 2 + items
            .iter()
            .map(|item| item.len())
            .max()
            .unwrap_or(0) as u16;

        let height = list.len() as u16;

        let centered = area.centered(
            Constraint::Length(width), // Horizontal
            Constraint::Length(height), // Vertical
        );

        list.render(centered, buf, &mut state.list_state);
    }
}
