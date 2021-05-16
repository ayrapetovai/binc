mod syntax;
mod number;
mod operators;

use number::{Number, NumberType};

use log::{info, trace, warn};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use syntax::parse;
use operators::{HandlerResult};

fn print_ui(number: &Number) {
    println!("{}", number);
}

type Executor = dyn FnOnce(&mut Number) -> HandlerResult;

fn generate_executor(command: &str) -> Result<Box<Executor>, String> {
    match parse(command) {
        Ok((left_operand_source, operator_handler, right_operand_source)) => {
            Ok(
                Box::new(move |main_buffer: &mut Number| {
                    operator_handler(main_buffer, left_operand_source, right_operand_source)
                })
            )
        },
        Err(message) => Err(message)
    }
}

fn main() {
    // https://docs.rs/clap/2.33.3/clap/
    // stderrlog::new().module(module_path!()).verbosity(4).init().unwrap();

    // `()` can be used when no completer is required
    let mut editor = Editor::<()>::new();
    // if rl.load_history("history.txt").is_err() {
    //     trace!("No previous history.");
    // }
    let mut main_buffer = Number::new(NumberType::Integer, false, 32);
    loop {
        print_ui(&main_buffer);
        let input = editor.readline("(binc) ");
        match input {
            Ok(commands) => {
                trace!("commands: {}", commands);
                let command_list = commands.split(";").collect::<Vec<_>>();
                for command in command_list {
                    match generate_executor(command) {
                        Ok(executor) => {
                            match executor(&mut main_buffer) {
                                HandlerResult::Error(message) => println!("error: {}", message),
                                HandlerResult::Historical => { editor.add_history_entry(command); },
                                HandlerResult::Nonhistorical => ()
                            }
                        }
                        Err(message) => println!("error: {}", message)
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                trace!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                trace!("CTRL-D");
                break
            },
            Err(err) => {
                trace!("Error: {:?}", err);
                break
            }
        }
    }
}
