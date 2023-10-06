use std::{
    env,
    ffi::OsString,
    fs::{DirEntry, ReadDir},
    io::{Error, Write},
    path::Path,
    process::Command,
};

fn main() -> Result<(), Error> {
    let mut shell = Shell::new();
    loop {
        shell.tick();
    }
}

struct Shell {
    term: console::Term,
    path: String,
    current_directory: String,
    cache: Vec<String>,
    input: String,
    suggestion: String,
    highlighting: bool,
    showing_entries: bool,
    highlighted_entry: (usize, usize),
    index: usize,
    pos: usize,
}
impl Shell {
    fn tick(&mut self) {
        if self.showing_entries {
            let dir = std::fs::read_dir("./").unwrap();
            let entries = dir_filter(self.input.to_owned(), dir);
            let len = entries.len();
            if len != 0 {
                self.term.write_all(b"\n").unwrap();
                self.term.clear_line().unwrap();
                let temp_input = self.input.split(" ").last().unwrap();
                let mut pos = 0;
                if self.highlighted_entry.0 == len {
                    self.highlighted_entry.0 = 0;
                }
                entries.into_iter().for_each(|x| {
                    let mut entry = x.unwrap().file_name().into_string().unwrap();
                    match self.highlighting {
                        false => {
                            entry.insert_str(temp_input.len(), "\x1b[0;37m");
                            self.term
                                .write_all(format!("\x1b[4;36m{} ", entry).as_bytes())
                                .unwrap()
                        }
                        true => {
                            if pos == self.highlighted_entry.0 {
                                self.term
                                    .write_all(
                                        format!("\x1b[30;46m{}\x1b[30;40m ", entry).as_bytes(),
                                    )
                                    .unwrap();
                            } else {
                                entry.insert_str(temp_input.len(), "\x1b[0;37m");
                                self.term
                                    .write_all(format!("\x1b[4;36m{} ", entry).as_bytes())
                                    .unwrap()
                            }
                        }
                    }
                    pos += 1;
                });
                self.term.move_cursor_up(1).unwrap();
            } else {
                (
                    self.showing_entries,
                    self.highlighting,
                    self.highlighted_entry.0,
                ) = (false, false, 0);
            }
        }
        self.term.clear_line().unwrap();
        self.term
            .write_all(
                format!(
                    "\x1b[32;40m{}\x1b[37;40m {}> ",
                    env::var("USER").unwrap(),
                    self.current_directory.to_owned()
                )
                .as_bytes(),
            )
            .unwrap();
        self.term.write_all(self.input.as_bytes()).unwrap();
        self.term
            .move_cursor_left(self.input.len() - self.pos)
            .unwrap();
        match self.term.read_key().unwrap() {
            console::Key::Char(x) => {
                self.term.write_all(x.to_string().as_bytes()).unwrap();
                self.input.insert(self.pos, x);
                self.pos += 1;
                if self.showing_entries {
                    self.term.move_cursor_down(1).unwrap();
                    self.term.clear_line().unwrap();
                    self.term.move_cursor_up(1).unwrap();
                    (
                        self.showing_entries,
                        self.highlighting,
                        self.highlighted_entry.0,
                    ) = (false, false, 0);
                }
            }
            console::Key::Tab => {
                if self.highlighting {
                    self.highlighted_entry.0 += 1;
                }
                if !self.showing_entries {
                    self.showing_entries = true;
                } else {
                    self.highlighting = true;
                }
            }
            console::Key::Backspace => {
                if self.pos != 0 {
                    self.input.remove(self.pos - 1);
                    self.term.clear_chars(1).unwrap();
                    self.pos -= 1;
                }
                if self.showing_entries {
                    self.term.move_cursor_down(1).unwrap();
                    self.term.clear_line().unwrap();
                    self.term.move_cursor_up(1).unwrap();
                    (
                        self.showing_entries,
                        self.highlighting,
                        self.highlighted_entry.0,
                    ) = (false, false, 0);
                }
            }
            console::Key::ArrowUp => {
                if self.index == 0 && self.cache.len() >= self.index + 1 {
                    self.index += 1;
                    self.input = self.cache[self.index - 1].to_owned();
                    self.pos = self.input.len();
                } else if self.cache.len() >= self.index + 1 {
                    self.index += 1;
                    self.input = self.cache[self.index - 1].to_owned();
                    self.pos = self.input.len();
                }
            }
            console::Key::ArrowDown => match self.index {
                0 => (),
                1 => {
                    self.index -= 1;
                    self.input = "".to_owned();
                    self.pos = 0;
                }
                _ => {
                    self.index -= 1;
                    self.input = self.cache[self.index - 1].to_owned();
                    self.pos = self.input.len();
                }
            },
            console::Key::ArrowLeft => {
                if self.pos != 0 {
                    self.pos -= 1;
                }
            }
            console::Key::ArrowRight => {
                if self.pos != self.input.len() {
                    self.pos += 1;
                }
            }
            console::Key::Enter => {
                if self.showing_entries {
                    self.term.move_cursor_down(1).unwrap();
                    self.term.clear_line().unwrap();
                    self.term.move_cursor_up(1).unwrap();
                    let mut temp_input = self.input.split(" ").collect::<Vec<&str>>();
                    temp_input.pop();
                    let dir = std::fs::read_dir("./").unwrap();
                    let entry =
                        dir_filtered_index(self.input.to_owned(), dir, self.highlighted_entry.0);
                    temp_input.push(entry.to_str().unwrap());
                    self.input = temp_input.join(" ");
                    self.pos = self.input.len();
                    (
                        self.showing_entries,
                        self.highlighting,
                        self.highlighted_entry.0,
                    ) = (false, false, 0);
                } else {
                    self.index = 0;
                    self.pos = 0;
                    self.term.write_all(b"\n").unwrap();
                    let mut parts = self.input.trim().split_whitespace();
                    let command = parts.next().unwrap_or("");
                    let args = parts;
                    if self.cache.get(0).unwrap_or(&"".to_owned()) != &self.input.to_owned()
                        || self.input != ""
                    {
                        self.cache.insert(0, self.input.to_owned());
                    }
                    match command {
                        "" => (),
                        "cd" => {
                            let new_dir = args.peekable().peek().map_or("/", |x| x);
                            let root = Path::new(new_dir);
                            if let Err(e) = env::set_current_dir(&root) {
                                eprintln!("{}", e);
                            }
                            self.current_directory = get_dir();
                        }
                        "exit" => std::process::exit(0),
                        command => {
                            let child = Command::new(command).args(args).spawn();
                            match child {
                                Ok(mut child) => {
                                    child.wait().unwrap();
                                }
                                Err(e) => {
                                    if e.raw_os_error().unwrap() == 2 {
                                        eprintln!("Unknown Command: \"{command}\"");
                                    } else {
                                        eprintln!("{}", e);
                                    }
                                }
                            };
                        }
                    }
                    self.input = String::new();
                    self.term
                        .write_all((self.current_directory.to_string() + "> ").as_bytes())
                        .unwrap();
                }
            }
            _ => (),
        };
    }
    fn new() -> Shell {
        Shell {
            term: console::Term::stdout(),
            path: std::env::var("PATH").unwrap_or("./".to_owned()),
            current_directory: get_dir(),
            cache: Vec::new(),
            input: String::new(),
            suggestion: String::new(),
            highlighting: false,
            showing_entries: false,
            highlighted_entry: (0, 0),
            index: 0,
            pos: 0,
        }
    }
}

// I just really dont want to look at this.
// And neither do you lets be honest.
fn get_dir() -> String {
    env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .split("/")
        .last()
        .unwrap()
        .to_string()
}
fn dir_filtered_index(input: String, dir: ReadDir, entry: usize) -> OsString {
    dir_filter(input, dir)
        .get(entry)
        .unwrap()
        .as_ref()
        .unwrap()
        .file_name()
}
fn dir_filter(input: String, dir: ReadDir) -> Vec<Result<DirEntry, Error>> {
    dir.filter(|x| {
        x.as_ref()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .starts_with(&input.split(" ").last().unwrap())
    })
    .collect::<Vec<Result<DirEntry, Error>>>()
}
