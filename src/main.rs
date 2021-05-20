mod syntax;
mod number;
mod operators;

use number::{Number, NumberType};

use log::{error, info, warn, trace};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use syntax::parse;
use operators::{HandlerResult};

fn print_ui(number: &Number) {
    println!();
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
                    if !command.is_empty() {
                        match generate_executor(command) {
                            Ok(executor) => {
                                editor.add_history_entry(command);
                                match executor(&mut main_buffer) {
                                    HandlerResult::Message(message) => println!("{}", message),
                                    HandlerResult::Historical => (),
                                    HandlerResult::Nonhistorical => ()
                                }
                            }
                            Err(message) => println!("error: {}", message)
                        }
                    } else {
                        trace!("empty command")
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
                error!("Error: {:?}", err);
                break
            }
        }
    }
}
