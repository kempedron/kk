use std::fs;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};
use termion::{color, screen};
use termion::event::Key;
use termion::input::TermRead;
use termion::{clear, cursor, style};
use termion::raw::IntoRawMode;


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
    pub scroll_offset: usize,
}

impl Explorer {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let mut explorer = Explorer {
            entries: vec![],
            selected: 0,
            is_open: false,
            current_dir: cwd.clone(),
            scroll_offset: 0,
        };
        explorer.load_dir(&cwd);
        explorer
    }

    pub fn load_dir(&mut self, path: &Path) {
        self.entries.clear();
        self.scroll_offset = 0;
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

    pub fn update_scroll(&mut self,visible_count: usize){
        if self.selected < self.scroll_offset{
            self.scroll_offset = self.selected;
        }

        if self.selected >= self.scroll_offset + visible_count{
            self.scroll_offset = self.selected - visible_count + 1;
        }
    }

    

pub fn render<W: Write>(&self, stdout: &mut W, height: u16) {
    write!(
        stdout,
        "{}{}{}{}{}",
        cursor::Goto(1, 1),
        color::Bg(color::Black),
        color::Fg(color::Yellow),
        self.current_dir.display(),
        style::Reset,
    ).unwrap();

    let visible_count = (height as usize).saturating_sub(2);


        for (i, entry) in self.entries.iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(visible_count)
        {
            let row = 2 + (i -self.scroll_offset) as u16;
            let is_select = i == self.selected;
            let icon = if entry.is_dir {"📁 "} else {"📄 "};
            let arrow = if is_select {"> "} else {" "};

            if is_select {
            write!(stdout, "{}{}{}{}{}{}{}",
                cursor::Goto(1, row),
                color::Bg(color::Rgb(60, 60, 60)),
                color::Fg(color::Yellow),
                clear::CurrentLine,
                arrow, icon, entry.name,
            ).unwrap();
        } else if entry.is_dir {
            write!(stdout, "{}{}{}{}{}{}",
                cursor::Goto(1, row),
                color::Bg(color::Black),
                color::Fg(color::Yellow),
                arrow, icon, entry.name,
            ).unwrap();
        } else {
            write!(stdout, "{}{}{}{}{}{}",
                cursor::Goto(1, row),
                color::Bg(color::Black),
                color::Fg(color::White),
                arrow, icon, entry.name,
            ).unwrap();
        }        
       let total = self.entries.len();
        write!(stdout, "{}{}{}  {}/{}  {}",
            cursor::Goto(1, height),
            color::Bg(color::Black),
            color::Fg(color::Yellow),
            self.selected + 1,
            total,
            style::Reset,
        ).unwrap();

        write!(stdout, "{}", style::Reset).unwrap();
    }

    stdout.flush().unwrap();
}

    pub fn run(&mut self) -> Option<PathBuf>{
        let stdin = std::io::stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(
        stdout,
        "{}{}{}",
        cursor::Hide,
        color::Bg(color::Black),
        clear::All,
    ).unwrap();
    stdout.flush().unwrap();

    let (_,height) = termion::terminal_size().unwrap();
    self.render(&mut stdout,height);
    
    for key in stdin.keys(){
            let (_,height) = termion::terminal_size().unwrap();
            let visible_count = (height as usize).saturating_sub(2); 

            match key.unwrap() {
                Key::Up => {
                    self.move_up();
                    self.update_scroll(visible_count);
                }
                Key::Down => {
                    self.move_down();
                    self.update_scroll(visible_count);
                }
                Key::Char('\n') => {
                    if let Some(filepath) = self.enter(){
                        write!(stdout,"{}{}{}",style::Reset,clear::All,cursor::Show).unwrap();
                        stdout.flush().unwrap();
                        drop(stdout);
                        return Some(filepath); 
                    }
                    self.update_scroll(visible_count);
                }
                Key::Ctrl('q') => {
                    write!(stdout,"{}{}{}",style::Reset,clear::All,cursor::Show).unwrap();
                    break;
                },
            Key::Ctrl('d') => {
                    if let Some(parent_dir) = self.current_dir.parent(){
                        let parent_dir = parent_dir.to_path_buf();
                        self.load_dir(&parent_dir);
                    }
                }
                _ => {}
            }
            write!(stdout,"{}{}",color::Bg(color::Black),clear::All).unwrap();
            self.render(&mut stdout,height); 
        }  
        write!(stdout,"{}{}{}",style::Reset,clear::All,cursor::Show).unwrap();
        stdout.flush().unwrap();
        None
    } 
}

