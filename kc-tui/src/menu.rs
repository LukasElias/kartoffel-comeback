use crate::{Buffer, List, ListState, Rect, StatefulWidget, Style};

// Model

#[derive(Debug)]
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

        list.render(area, buf, &mut state.list_state);
    }
}
