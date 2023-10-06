use std::{
    env,
    fs::DirEntry,
    io::{Error, Write},
    path::Path,
    process::Command,
};

fn main() -> Result<(), Error> {
    let (mut cache, mut input) = (Vec::<String>::new(), String::new());
    let (mut highlighting, mut highlighted_entry) = (false, (0, 0));
    let (mut index, mut pos) = (0, 0);
    let mut term = console::Term::stdout();
    let mut current_directory = get_dir();
    let mut showing_entries = false;
    loop {
        if showing_entries {
            let dir = std::fs::read_dir("./")?;
            let entries = dir
                .filter(|x| {
                    x.as_ref()
                        .unwrap()
                        .file_name()
                        .into_string()
                        .unwrap()
                        .starts_with(&input.split(" ").last().unwrap())
                })
                .collect::<Vec<Result<DirEntry, Error>>>();
            let len = entries.len();
            if len != 0 {
                term.write_all(b"\n")?;
                term.clear_line()?;
                let mut pos = 0;
                entries.into_iter().for_each(|x| {
                    let mut entry = x.unwrap().file_name().into_string().unwrap();
                    match highlighting {
                        false => {
                            entry.insert_str(input.len(), "\x1b[0;37m");
                            term.write_all(format!("\x1b[4;36m{} ", entry).as_bytes())
                                .unwrap()
                        }
                        true => {
                            if pos == highlighted_entry.0 - 1 {
                                term.write_all(
                                    format!("\x1b[30;46m{}\x1b[30;40m ", entry).as_bytes(),
                                )
                                .unwrap();
                            } else {
                                entry.insert_str(input.len(), "\x1b[0;37m");
                                term.write_all(format!("\x1b[4;36m{} ", entry).as_bytes())
                                    .unwrap()
                            }
                        }
                    }
                    pos += 1;
                });
                if highlighted_entry.0 > len - 1 {
                    highlighted_entry.0 = 0;
                }
                term.move_cursor_up(1)?;
            }
        }
        term.clear_line()?;
        term.write_all(
            format!(
                "\x1b[32;40m{}\x1b[37;40m {}> ",
                env::var("USER").unwrap(),
                current_directory.to_owned()
            )
            .as_bytes(),
        )?;
        term.write_all(input.as_bytes())?;
        term.move_cursor_left(input.len() - pos)?;
        match term.read_key()? {
            console::Key::Char(x) => {
                term.write_all(x.to_string().as_bytes())?;
                input.insert(pos, x);
                pos += 1;
                if showing_entries {
                    term.move_cursor_down(1)?;
                    term.clear_line()?;
                    term.move_cursor_up(1)?;
                    (showing_entries, highlighting, highlighted_entry.0) = (false, false, 0);
                }
            }
            console::Key::Tab => {
                if !showing_entries {
                    showing_entries = true;
                } else {
                    highlighting = true;
                    highlighted_entry.0 += 1;
                }
            }
            console::Key::Backspace => {
                if pos != 0 {
                    input.remove(pos - 1);
                    term.clear_chars(1)?;
                    pos -= 1;
                }
                if showing_entries {
                    term.move_cursor_down(1)?;
                    term.clear_line()?;
                    term.move_cursor_up(1)?;
                    (showing_entries, highlighting, highlighted_entry.0) = (false, false, 0);
                }
            }
            console::Key::ArrowUp => {
                if index == 0 && cache.len() >= index + 1 {
                    index += 1;
                    input = cache[index - 1].to_owned();
                    pos = input.len();
                } else if cache.len() >= index + 1 {
                    index += 1;
                    input = cache[index - 1].to_owned();
                    pos = input.len();
                }
            }
            console::Key::ArrowDown => match index {
                0 => (),
                1 => {
                    index -= 1;
                    input = "".to_owned();
                    pos = 0;
                }
                _ => {
                    index -= 1;
                    input = cache[index - 1].to_owned();
                    pos = input.len();
                }
            },
            console::Key::ArrowLeft => {
                if pos != 0 {
                    pos -= 1;
                }
            }
            console::Key::ArrowRight => {
                if pos != input.len() {
                    pos += 1;
                }
            }
            console::Key::Enter => {
                index = 0;
                pos = 0;
                term.write_all(b"\n")?;
                let mut parts = input.trim().split_whitespace();
                let command = parts.next().unwrap_or("");
                let args = parts;
                if cache.get(0).unwrap_or(&"".to_owned()) != &input.to_owned() || input != "" {
                    cache.insert(0, input.to_owned());
                }
                match command {
                    "" => (),
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        }
                        current_directory = get_dir();
                    }
                    "exit" => return Ok(()),
                    command => {
                        let child = Command::new(command).args(args).spawn();
                        match child {
                            Ok(mut child) => {
                                child.wait()?;
                            }
                            Err(e) => eprintln!("{}", e),
                        };
                    }
                }
                input = String::new();
                term.write_all((current_directory.to_string() + "> ").as_bytes())?;
            }
            _ => (),
        };
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
