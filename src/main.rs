use std::{
    env,
    io::{Error, Write},
    path::Path,
    process::Command,
};

fn main() -> Result<(), Error> {
    let mut cache: Vec<String> = Vec::new();
    let mut term = console::Term::stdout();
    let mut input = String::new();
    let mut index = 0;
    let mut pos = 0;
    let mut current_directory = get_dir();
    loop {
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
            }
            console::Key::Backspace => {
                if pos != 0 {
                    input.remove(pos - 1);
                    term.clear_chars(1)?;
                    pos -= 1;
                }
            }
            console::Key::ArrowUp => {
                if index == 0 && cache.len() >= index + 1 {
                    cache.insert(0, input.to_string());
                    index += 2;
                    input = cache[index - 1].to_owned();
                    pos = input.len();
                } else if cache.len() >= index + 1 {
                    index += 1;
                    input = cache[index - 1].to_owned();
                    pos = input.len();
                }
            }
            console::Key::ArrowDown => match index {
                2 => {
                    index -= 2;
                    input = cache.remove(0);
                    pos = input.len();
                }
                0 => (),
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
                cache.insert(0, input.to_owned());
                let mut parts = input.trim().split_whitespace();
                let command = parts.next().unwrap_or("");
                let args = parts;
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
                term.write_all(("\n".to_string() + current_directory.as_str() + "> ").as_bytes())?;
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
