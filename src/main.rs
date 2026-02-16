use std::fmt::write;
use std::io::{self, Write, stdout, BufWriter};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
use termion::event::Key;
use termion::input::TermRead;
use termion::{clear, cursor, style};
use termion::{event::Event, raw::IntoRawMode};
use std::fs::File;

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
}

impl Editor {
    fn new() -> Self {
        Editor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_y];
        line.insert(self.cursor_x, c);
        self.cursor_x += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
        }
    }

    fn insert_new_line(&mut self) {
        let current_line = self.lines[self.cursor_y].clone();
        let (left, right) = current_line.split_at(self.cursor_x);
        self.lines[self.cursor_y] = left.to_string();
        self.lines.insert(self.cursor_y + 1, right.to_string());

        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::Up if self.cursor_y > 0 => {
                self.cursor_y -= 1;
                let line_len = self.lines[self.cursor_y].len();
                if self.cursor_x > line_len {
                    self.cursor_x = line_len;
                }
            }
            Key::Down => {
                self.cursor_y += 1;
                let line_len = self.lines[self.cursor_y].len();
                if self.cursor_x > line_len {
                    self.cursor_x = line_len;
                }
            }
            Key::Right => {
                if self.cursor_x < self.lines[self.cursor_y].len() {
                    self.cursor_x += 1;
                } else {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
            Key::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                } else {
                    self.cursor_y -= 1;
                    self.cursor_x = self.lines[self.cursor_y].len()
                }
            }
            _ => todo!(),
        }
    }

    fn write_file(&self) -> io::Result<()>{
        let file = File::create("output.txt")?;
        let mut writer = BufWriter::new(file);
        
        for line in &self.lines {
            writeln!(writer,"{}",line);
        }
        writer.flush()?;
        Ok(())
        
    }

    fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;

        for (i, line) in self.lines.iter().enumerate() {
            write!(stdout, "{}{}\r\n", cursor::Goto(1, i as u16 + 1), line)?;
        }
        let (_, height) = termion::terminal_size()?;
        write!(
            stdout,
            "{}{}Press Ctr+Q to quit | Line {}/{} Col {}",
            cursor::Goto(1, height),
            termion::style::Invert,
            self.cursor_y + 1,
            self.lines.len(),
            self.cursor_x + 1,
        )?;

        write!(stdout,"{}",style::Reset)?;

        write!(
            stdout,
            "{}{}",
            cursor::Goto((self.cursor_x + 1) as u16,(self.cursor_y + 1) as u16),
            cursor::Show
        )?;
    
        stdout.flush()
    }


   
    fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = stdout().into_raw_mode()?;
    
        self.draw(&mut stdout)?;
        

        for evt in stdin.events() {
            match evt? {
                Event::Key(Key::Ctrl('q')) => break,
                Event::Key(Key::Char('\n')) => self.insert_new_line(),
                Event::Key(Key::Char(c)) => self.insert_char(c),
                Event::Key(Key::Backspace) => self.delete_char(),
                Event::Key(Key::Ctrl('s')) => self.write_file()?,
                Event::Key(Key::Ctrl('w')) => { self.write_file()?; break}
                Event::Key(key @ Key::Up)
                | Event::Key(key @ Key::Down)
                | Event::Key(key @ Key::Left)
                | Event::Key(key @ Key::Right) => self.move_cursor(key),
                _ => {}
            }

            self.draw(&mut stdout)?;

          
        }
        write!(stdout, "{}", clear::All)?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut editor = Editor::new();
    editor.run()
}
