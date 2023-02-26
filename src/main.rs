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
    loop {
        term.clear_line()?;
        term.write_all(b"> ")?;
        term.write_all(input.as_bytes())?;
        match term.read_key()? {
            console::Key::Char(x) => {
                term.write_all(x.to_string().as_bytes())?;
                input.insert(input.len(), x)
            }
            console::Key::Backspace => {
                if input != "".to_string() {
                    input.pop();
                    term.clear_chars(1)?;
                }
            }
            console::Key::ArrowUp => {
                if index == 0 && cache.len() >= index + 1 {
                    cache.insert(0, input.to_string());
                    index += 2;
                    input = cache[index - 1].to_owned();
                } else if cache.len() >= index + 1 {
                    index += 1;
                    input = cache[index - 1].to_owned();
                }
            }
            console::Key::ArrowDown => match index {
                2 => {
                    index -= 2;
                    input = cache.remove(0);
                }
                0 => (),
                _ => {
                    index -= 1;
                    input = cache[index - 1].to_owned();
                }
            },
            console::Key::Enter => {
                index = 0;
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
                term.write_all(b"> ")?;
            }
            _ => (),
        };
    }
}
