mod filemanager;

use std::env;
use std::fmt::write;
use std::fs::File;
use std::io::{self, BufWriter, Write, stdout};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use termion::color::{self, Yellow};
use termion::event::Key;
use termion::input::TermRead;
use termion::{clear, cursor, style};
use termion::{event::Event, raw::IntoRawMode};

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    row_offset: usize,
    col_offset: usize,
    is_changed: bool,
}

impl Editor {
    fn new() -> Self {
        Editor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            col_offset: 0,
            is_changed: false,
        }
    }

    fn scroll(&mut self) {
        let (width, height) = termion::terminal_size().unwrap();

        let visible_height = (height - 1) as usize;
        let visible_width = width as usize;

        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }

        if self.cursor_y >= self.row_offset + visible_height {
            self.row_offset = self.cursor_y - visible_height + 1;
        }

        if self.cursor_x < self.col_offset {
            self.col_offset = self.cursor_x;
        }

        if self.cursor_x >= self.col_offset + visible_width {
            self.col_offset = self.cursor_x - visible_width + 1;
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_y];
        let byte_ind = line
            .char_indices()
            .nth(self.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_ind, c);
        self.cursor_x += 1;
        self.is_changed = true;
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            let byte_ind = line
                .char_indices()
                .nth(self.cursor_x - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            line.remove(byte_ind);
            self.cursor_x -= 1;
            self.is_changed = true;
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].chars().count();
            self.lines[self.cursor_y].push_str(&current_line);
            self.is_changed = true;
        }
    }

    fn insert_new_line(&mut self) {
        let line = &self.lines[self.cursor_y];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let right = line[byte_pos..].to_string();
        self.lines[self.cursor_y].truncate(byte_pos);
        self.lines.insert(self.cursor_y + 1, right);

        self.cursor_y += 1;
        self.cursor_x = 0;
        self.is_changed = true;
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    let line_len = self.lines[self.cursor_y].chars().count();
                    if self.cursor_x > line_len {
                        self.cursor_x = line_len;
                    }
                }
            }
            Key::Down => {
                if self.cursor_y + 1 < self.lines.len() {
                    println!("1");
                    self.cursor_y += 1;
                    let line_len = self.lines[self.cursor_y].chars().count();
                    if self.cursor_x > line_len {
                        self.cursor_x = line_len;
                    }
                }
            }
            Key::Right => {
                if self.cursor_x < self.lines[self.cursor_y].chars().count() {
                    self.cursor_x += 1;
                } else if self.cursor_y + 1 < self.lines.len() {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
            Key::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                } else if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x = self.lines[self.cursor_y].chars().count();
                }
            }

            _ => todo!(),
        }
    }

    fn write_file(&mut self, filename: &String) -> io::Result<()> {
        let file = File::create(format!("{}", filename))?;
        let mut writer = BufWriter::new(file);

        for line in &self.lines {
            writeln!(writer, "{}", line);
        }
        writer.flush()?;
        self.is_changed = false;
        Ok(())
    }

    fn load_file(filename: &String) -> Self {
        if let Ok(content) = std::fs::read_to_string(filename) {
            let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            Editor {
                lines: if lines.is_empty() {
                    vec![String::new()]
                } else {
                    lines
                },
                cursor_x: 0,
                cursor_y: 0,
                row_offset: 0,
                col_offset: 0,
                is_changed: false,
            }
        } else {
            Editor::new()
        }
    }

    fn confirm<W: Write>(&self, stdout: &mut W, message: &str) -> io::Result<bool> {
        let (_, height) = termion::terminal_size()?;

        write!(
            stdout,
            "{}{}{} (Y/n): {}",
            cursor::Goto(1, height),
            style::Invert,
            message,
            style::Reset,
        )?;
        stdout.flush()?;

        let stdin = io::stdin();
        for evt in stdin.events() {
            match evt? {
                Event::Key(Key::Char('y')) | Event::Key(Key::Char('Y')) => return Ok(true),
                Event::Key(Key::Char('n')) | Event::Key(Key::Char('N')) => return Ok(false),
                _ => {}
            }
        }
        Ok(false)
    }

    fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        write!(
            stdout,
            "{}{}{}{}",
            style::Reset,
            color::Bg(color::Rgb(40, 42, 54)), // установить чёрный фон
            clear::All,              // залить им весь экран
            cursor::Goto(1, 1)       // вернуть курсор в начало
        )?;

       
        let (width, height) = termion::terminal_size()?;
        let visible_height = (height - 1) as usize;

        for i in 0..visible_height {
            let file_row = i + self.row_offset;
            if file_row < self.lines.len() {
                let line = &self.lines[file_row];
                write!(stdout, "{}{}{}", cursor::Goto(1, i as u16 + 1), color::Fg(color::Rgb(248, 248, 242)), line)?;
            }
        }

        write!(
            stdout,
            "{}{}{}Press Ctr+Q to quit | Line {}/{} Col {}",
            cursor::Goto(1, height),
            termion::style::Invert,
            color::Fg(Yellow),
            self.cursor_y + 1,
            self.lines.len(),
            self.cursor_x + 1,
        )?;

        write!(
            stdout,
            "{}{}",
            cursor::Goto(
                (self.cursor_x - self.col_offset + 1) as u16,
                (self.cursor_y - self.row_offset + 1) as u16
            ),
            cursor::Show
        )?;
        stdout.flush()
    }

    fn run(&mut self, filename: &String) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = stdout().into_raw_mode()?;

        self.scroll();
        self.draw(&mut stdout)?;

        for evt in stdin.events() {
            match evt? {
                Event::Key(Key::Ctrl('q')) => {
                    if self.is_changed {
                        let save = self.confirm(&mut stdout, "Save changes?(Y/n):")?;
                        if save {
                            self.write_file(filename)?;
                        }
                        write!(
                            stdout,
                            "{}{}{}\n",
                            style::Reset,
                            clear::All,
                            cursor::Goto(1, 1)
                        );
                        break;
                    } else {
                        write!(
                            stdout,
                            "{}{}{}\n",
                            style::Reset,
                            clear::All,
                            cursor::Goto(1, 1)
                        );

                        break;
                    }
                }
                Event::Key(Key::Char('\n')) => self.insert_new_line(),
                Event::Key(Key::Char(c)) => self.insert_char(c),
                Event::Key(Key::Backspace) => self.delete_char(),
                Event::Key(Key::Ctrl('s')) => self.write_file(filename)?,
                Event::Key(Key::Ctrl('w')) => {
                    self.write_file(filename)?;
                    break;
                }

                Event::Key(key @ Key::Up)
                | Event::Key(key @ Key::Down)
                | Event::Key(key @ Key::Left)
                | Event::Key(key @ Key::Right) => self.move_cursor(key),
                _ => {}
            }

            self.scroll();

            self.draw(&mut stdout)?;
        }
        write!(stdout, "{}", clear::All)?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("use kk --<filename>");
    }
    let filename = &args[1];

    let path = Path::new(filename.trim());
    if path.is_dir() {
        let mut explorer = filemanager::Explorer::new();
        explorer.load_dir(path);
        if let Some(selected_file) = explorer.run() {
            let name = selected_file.to_string_lossy().to_string();
            let mut editor = Editor::load_file(&name);
            editor.run(&name)?;
        }
    } else {
        let mut editor = Editor::load_file(filename);
        editor.run(filename)?;
    }

    Ok(())
}
