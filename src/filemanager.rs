use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::fs;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct Explorer {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected: usize,
    pub is_open: bool,
}

impl Explorer {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let mut explorer = Explorer {
            entries: vec![],
            selected: 0,
            is_open: false,
            current_dir: cwd.clone(),
        };
        explorer.load_dir(&cwd);
        explorer
    }

    pub fn load_dir(&mut self, path: &Path) {
        self.entries.clear();
        self.selected = 0;

        let path = path.canonicalize().unwrap_or(path.to_path_buf());
        let path = path.as_path();

        self.current_dir = path.to_path_buf();

        if let Some(parent) = path.parent() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        if let Ok(read_dir) = fs::read_dir(path) {
            let mut entries: Vec<DirEntry> = read_dir
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = e.file_name().to_string_lossy().to_string();
                    DirEntry { name, path, is_dir }
                })
                .collect();
            entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
            self.entries.extend(entries);
        }
    }
    pub fn enter(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let path = entry.path.clone();
                self.load_dir(&path);
                None
            } else {
                Some(entry.path.clone())
            }
        } else {
            None
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn render(&mut self, start_col: u16, start_row: u16, height: u16) {
        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(start_col, start_row),
            SetForegroundColor(Color::Cyan),
            Print(format!("{}", self.current_dir.display())),
            ResetColor,
        )
        .unwrap();

        let visible: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .skip(0)
            .take(height as usize - 1)
            .collect();

        for (i, (ind, entry)) in visible.iter().enumerate() {
            let row = start_row + 1 + i as u16;
            let is_selected = *ind == self.selected;
            queue!(stdout, cursor::MoveTo(start_col, row)).unwrap();

            if is_selected {
                queue!(stdout, SetForegroundColor(Color::Black)).unwrap();
            }
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let color = if entry.is_dir {
                Color::Blue
            } else {
                Color::White
            };
            let arrow = if is_selected { ">" } else { " " };

            queue!(
                stdout,
                SetForegroundColor(color),
                Print(format!("{}{}{}", arrow, icon, entry.name)),
                ResetColor,
            )
            .unwrap();
        }
        stdout.flush().unwrap();
    }

    pub fn run(&mut self) -> Option<PathBuf> {
        let mut stdout = stdout();
        terminal::enable_raw_mode().unwrap();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();

        loop {
            let (_, height) = terminal::size().unwrap();
            execute!(
                stdout,
                terminal::Clear(terminal::ClearType::All),
                Print("\n")
            )
            .ok();
            self.render(0, 0, height);

            if let Ok(event) = crossterm::event::read() {
                match event {
                    crossterm::event::Event::Key(key) => match (key.modifiers, key.code) {
                        (_, crossterm::event::KeyCode::Up) => self.move_up(),
                        (_, crossterm::event::KeyCode::Down) => self.move_down(),
                        (_, crossterm::event::KeyCode::Enter) => {
                            if let Some(filepath) = self.enter() {
                                terminal::disable_raw_mode().unwrap();
                                execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)
                                    .unwrap();
                                return Some(filepath);
                            }
                        }
                        (
                            crossterm::event::KeyModifiers::CONTROL,
                            crossterm::event::KeyCode::Char('q'),
                        ) => {
                            break;
                        }
                        (
                            crossterm::event::KeyModifiers::CONTROL,
                            crossterm::event::KeyCode::Char('d'),
                        ) => {
                            if let Some(parent) = self.current_dir.parent() {
                                let parent = parent.to_path_buf();
                                self.load_dir(&parent);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().unwrap();
        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        None
    }
}
