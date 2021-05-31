//#![deny(warnings, missing_docs)]
mod syntax;
mod number;
mod operators;

use number::{Number, NumberType};

use log::{error, info, warn, trace};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use syntax::parse;
use operators::{HandlerResult};
use crate::operators::OperationResult;
use clap::{App, Arg};
use colored::{Colorize, Color};

// TODO string builder with formatting
fn print_ui(number: &Number) {
    let line = format!(
        "{3:>0$} {4:>1$} {5:>2$}",
        number.number_of_digits_in_radix(16) + 2, number.number_of_digits_in_radix(10) + 2, number.number_of_digits_in_radix(8) + 2,
        number.to_string(16), number.to_string(10), number.to_string(8)
    );

    println!();
    println!("{}", line.color(Color::BrightYellow));
    println!("{}", number);
}

type Executor = dyn FnOnce(&mut Number) -> OperationResult;

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
    let matches = App::new("binc")
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity. More 'v's, more verbosity. Four 'v' used for the most verbose output"))
        .get_matches();

    let verbosity_level = match matches.occurrences_of("v") {
        0..=4 => matches.occurrences_of("v"),
        _ => {
            println!("Verbosity level must be 0 to 4 inclusive.");
            return;
        }
    } as usize;

    stderrlog::new().module(module_path!()).verbosity(verbosity_level).init().unwrap();

    // `()` can be used when no completer is required
    let mut cli_editor = Editor::<()>::new();
    let mut main_buffer = Number::new(NumberType::Integer, true, 32).unwrap();
    loop {
        print_ui(&main_buffer);
        let input = cli_editor.readline("(binc) ");
        match input {
            Ok(commands) => {
                trace!("main loop: got commands: '{}'", commands);
                let command_list = commands.split(";").collect::<Vec<_>>();
                for mut command in command_list {
                    if command.is_empty() {
                        match cli_editor.history().last() {
                            Some(last_command) => command = last_command,
                            None => continue
                        }
                    } else {
                        cli_editor.add_history_entry(command);
                    }
                    match generate_executor(command) {
                        Ok(executor) => {
                            match executor(&mut main_buffer) {
                                Ok((handler_result, optional_message)) => {
                                    if handler_result == HandlerResult::Historical {
                                        // TODO add to undo/redo history
                                    }
                                    if let Some(message) = optional_message {
                                        println!("{}", message)
                                    }
                                }
                                Err(err_msg) => println!("operation error: {}", err_msg)
                            }
                            trace!("BUFFER: signed {}, size {}, {:b} ", main_buffer.signed(), main_buffer.max_size(), main_buffer.to_u128());
                        }
                        Err(err_msg) => println!("parsing error: {}", err_msg)
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
