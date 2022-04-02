//#![deny(warnings, missing_docs)]

// binc
// Copyright (C) 2022  Artem Ayrapetov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod syntax;
mod number;
mod operators;
mod history;

use number::{Number, NumberType};

use log::{error, trace, debug};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Event, Cmd, KeyEvent, EventHandler};
use syntax::parse;
use operators::{HandlerResult};
use crate::operators::OperationResult;
use crate::history::History;
use clap::{Arg, command};
use colored::{Colorize, Color};
use std::str::FromStr;
use smallvec::{smallvec};
use std::process::exit;

fn print_ui(number: &Number) {
    let line = format!(
        "{3:>0$} {4:>1$} {5:>2$}",
        number.number_of_digits_in_radix(16) + 2, number.number_of_digits_in_radix(10) + 2, number.number_of_digits_in_radix(8) + 2,
        number.to_string_prefixed(16), number.to_string_prefixed(10), number.to_string_prefixed(8)
    );

    println!();
    println!("{}", line.color(Color::Green));
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

fn interactive_routine(history_size: usize) {
    // `()` can be used when no completer is required
    let mut cli_editor = Editor::<()>::new();
    cli_editor.set_max_history_size(history_size);

    let mut main_buffer = Number::new(NumberType::Integer, true, 32).unwrap();
    let mut buffer_history = History::new( history_size);
    buffer_history.save(&main_buffer);

    // SHIFT+LEFT/SHIFT+RIGHT - undo/redo
    cli_editor.bind_sequence(Event::KeySeq(smallvec![KeyEvent::ctrl('Q')]), EventHandler::from(Cmd::EndOfFile));

    loop {
        print_ui(&main_buffer);
        let input = cli_editor.readline("(binc) ");
        match input {
            Ok(commands) => {
                trace!("interactive: got commands: '{}'", commands);
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
                                    match handler_result {
                                        HandlerResult::Historical => buffer_history.save(&main_buffer),
                                        HandlerResult::Undo => main_buffer = buffer_history.backward(),
                                        HandlerResult::Redo => main_buffer = buffer_history.forward(),
                                        HandlerResult::Nonhistorical => {}
                                    }
                                    if let Some(message) = optional_message {
                                        println!("{}", message)
                                    }
                                }
                                Err(err_msg) => println!("operation error: {}", err_msg)
                            }
                            trace!("buffer: {}, size {}, bits 0b{:b} ", main_buffer.signed(), main_buffer.max_size(), main_buffer.to_u128());
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

fn not_interactive_routine(commands: &str, format: &str) {
    let mut main_buffer = Number::new(NumberType::Integer, true, 32).unwrap();

    trace!("interactive: got commands: '{}'", commands);
    let command_list = commands.split(";").collect::<Vec<_>>();
    for command in command_list {
        if command.is_empty() {
            debug!("nothing to do");
            exit(0);
        }
        match generate_executor(command) {
            Ok(executor) => {
                match executor(&mut main_buffer) {
                    Ok((handler_result, _)) => {
                        match handler_result {
                            HandlerResult::Historical => {},
                            HandlerResult::Undo => {
                                eprintln!("undo does not work in non-interactive mode");
                            },
                            HandlerResult::Redo => {
                                eprintln!("redo does not work in non-interactive mode");
                            },
                            HandlerResult::Nonhistorical => {}
                        }
                    }
                    Err(err_msg) => {
                        eprintln!("operation error: {}", err_msg);
                        exit(1);
                    }
                }
                trace!("buffer: {}, size {}, bits 0b{:b} ", main_buffer.signed(), main_buffer.max_size(), main_buffer.to_u128());
            }
            Err(err_msg) => {
                eprintln!("parsing error: {}", err_msg);
                exit(1);
            }
        }
    }
    trace!("command executing is done, format is '{}'", format);
    // FIXME refactor format parsing
    let output = match format {
        "0x" => main_buffer.to_string(16, true),
        "x" => main_buffer.to_string(16, false),
        "0d" => main_buffer.to_string(10, true),
        "d" => main_buffer.to_string(10, false),
        "0o" => main_buffer.to_string(8, true),
        "o" => main_buffer.to_string(8, false),
        "0b" => main_buffer.to_string(2, true),
        "b" => main_buffer.to_string(2, false),
        _ => "".to_owned()
    };
    if !output.is_empty() {
        println!("{}", output);
    }
}

fn main() {
    // TODO make struct with parameters https://github.com/clap-rs/clap
    // TODO make a cli.yml (https://docs.rs/clap/2.33.3/clap/), and use it like this: let yaml = load_yaml!("cli.yml"); let matches = App::from_yaml(yaml).get_matches();
    let matches = command!()
        .version("1")
        .arg(Arg::new("v")
            .short('v')
            .multiple_occurrences(true)// multiple_values?
            .help("Sets the level of verbosity. More 'v's, more verbosity. Four 'v' are used for the most verbose output"))
        .arg(Arg::new("history")
            .long("history")
            .short('h')
            .default_value("100")
            .takes_value(true)
            .help("Set the size of the command history"))
        .arg(Arg::new("expression")
            .long("expression")
            .short('e')
            .takes_value(true)
            .allow_hyphen_values(true)
            .help("commands to execute, separated by ';'"))
        .arg(Arg::new("format")
            .long("format")
            .short('f')
            .takes_value(true)
            .default_value("b")
            .help("b -binary, o - octal, d - decimal, h - hexadecimal. 0f - with prefix, where f is (b|o|d|h)+"))
        .get_matches();

    let verbosity_level = match matches.occurrences_of("v") {
        0..=4 => matches.occurrences_of("v"),
        _ => {
            println!("Verbosity level must be 0 to 4 inclusive.");
            return;
        }
    } as usize;

    let history_size_raw = matches.value_of("history").unwrap();
    let history_size = match usize::from_str(history_size_raw) {
        Ok(s) => s,
        Err(e) => {
            println!("Consider key 'history', value '{}' for size cannot be parsed: {}", history_size_raw, e);
            return;
        }
    };
    debug!("History size {}", history_size);

    stderrlog::new().module(module_path!()).verbosity(verbosity_level).init().unwrap();

    match matches.value_of("expression") {
        Some(commands) => {
            let format = matches.value_of("format").unwrap();
            not_interactive_routine(commands, format);
        },
        None => {
            interactive_routine(history_size);
        }
    }
}
