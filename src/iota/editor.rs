use super::Response;
use input::Input;
use buffer::Direction;
use keyboard::Key;
use view::View;
use frontends::{Frontend, EditorEvent};
use modes::Mode;
use overlay::{OverlayType};
use utils;


#[derive(Copy, Show)]
pub enum Command {
    SaveBuffer,
    ExitEditor,

    MoveCursor(Direction),
    LineEnd,
    LineStart,

    Delete(Direction),
    InsertTab,
    InsertChar(char),

    SetOverlay(OverlayType),

    Undo,
    Redo,

    Unknown,
    None,
}

impl Command {
    #[inline]
    pub fn from_str(string: &str) -> Command {
        match string {
            "q" | "quit" => Command::ExitEditor,
            "w" | "write" => Command::SaveBuffer,

            _            => Command::Unknown,
        }
    }
}

pub struct Editor<'e, T: Frontend> {
    view: View<'e>,
    running: bool,
    frontend: T,
    mode: Box<Mode + 'e>,
}

impl<'e, T: Frontend> Editor<'e, T> {
    pub fn new(source: Input, mode: Box<Mode + 'e>, frontend: T) -> Editor<'e, T> {
        let height = frontend.get_window_height();
        let width = frontend.get_window_width();
        let view = View::new(source, width, height);

        Editor {
            view: view,
            running: true,
            frontend: frontend,
            mode: mode,
        }
    }

    fn handle_key_event(&mut self, key: Option<Key>) {
        let command = self.mode.handle_key_event(key, &mut self.view);
        let response = self.handle_command(command);

        if let Response::Quit = response {
            self.running = false
        }
    }

    fn draw(&mut self) {
        self.view.draw(&mut self.frontend);
        self.view.draw_status(&mut self.frontend);
    }

    fn handle_command(&mut self, c: Command) -> Response {
        match c {
            // Editor Commands
            Command::ExitEditor      => return Response::Quit,
            Command::SaveBuffer      => utils::save_buffer(&self.view.buffer),

            // Navigation
            Command::MoveCursor(dir) => self.view.move_cursor(dir),
            Command::LineEnd         => self.view.move_cursor_to_line_end(),
            Command::LineStart       => self.view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => self.view.delete_chars(dir),
            Command::InsertTab       => self.view.insert_tab(),
            Command::InsertChar(c)   => self.view.insert_char(c),
            Command::Redo            => self.view.redo(),
            Command::Undo            => self.view.undo(),

            Command::SetOverlay(o)   => self.view.set_overlay(o),

            _ => {},
        }
        Response::Continue
    }

    pub fn start(&mut self) {
        while self.running {
            self.draw();
            self.frontend.present();
            let event = self.frontend.poll_event();

            if let EditorEvent::KeyEvent(key) = event {
                self.handle_key_event(key)
            }
        }
    }
}
