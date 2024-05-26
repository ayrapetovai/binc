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
mod buffer;
mod operators;
mod history;

use buffer::{BincBuffer, BincBufferType};

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
use crossterm_cursor::cursor;

fn print_ui(number: &BincBuffer) {
    let line = format!(
        "{0:>1$} {2:>3$} {4:>5$} {6:>7$}",
        number.to_string_as_char(), 3,
        number.to_string_prefixed(16), number.number_of_digits_in_radix(16) + 2,
        number.to_string_prefixed(10), number.number_of_digits_in_radix(10) + 2,
        number.to_string_prefixed(8), number.number_of_digits_in_radix(8) + 2,
    );

    println!();
    println!("{}", line.color(Color::Green));
    println!("{}", number);
}

type Executor = dyn FnOnce(&mut BincBuffer) -> OperationResult;

fn generate_executor(command: &str) -> Result<Box<Executor>, String> {
    match parse(command) {
        Ok((left_operand_source, operator_handler, right_operand_source)) => {
            Ok(
                Box::new(move |main_buffer: &mut BincBuffer| {
                    operator_handler(main_buffer, left_operand_source, right_operand_source)
                })
            )
        },
        Err(message) => Err(message)
    }
}

fn interactive_routine(history_size: usize, append_output: bool) {
    // `()` can be used when no completer is required
    let mut cli_editor = Editor::<()>::new();
    cli_editor.set_max_history_size(history_size);

    let mut main_buffer = BincBuffer::new(BincBufferType::Integer, true, 32).unwrap();
    let mut buffer_history = History::new( history_size);
    buffer_history.save(&main_buffer);

    // TODO SHIFT+LEFT/SHIFT+RIGHT and Ctrl-u/Ctrl-r (bash intercepts this) - undo/redo
    cli_editor.bind_sequence(Event::KeySeq(smallvec![KeyEvent::ctrl('Q')]), EventHandler::from(Cmd::EndOfFile));

    let mut lines_printed = 0u16;
    loop {
        print_ui(&main_buffer);
        lines_printed = 6;

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
                                        println!("{}", message);
                                        lines_printed += 1;
                                    }
                                }
                                Err(err_msg) => {
                                    println!("operation error: {}", err_msg);
                                    lines_printed += 1;
                                }
                            }
                            trace!("buffer: {}, size {}, bits 0b{:b} ", main_buffer.signed(), main_buffer.max_size(), main_buffer.to_u128());
                        }
                        Err(err_msg) => {
                            println!("parsing error: {}", err_msg);
                            lines_printed += 1;
                        }
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

        if !append_output {
            // for _ in 6..lines_printed {
            //     println!();
            //     lines_printed += 1;
            // }

            let mut cursor = cursor();
            cursor.move_up(lines_printed).expect("TODO: panic message");
        }
    }
}

fn not_interactive_routine(commands: &str, format: &str, prepend0: bool) -> String {
    let mut main_buffer = BincBuffer::new(BincBufferType::Integer, true, 32).unwrap();

    trace!("interactive: got commands: '{}'", commands);
    let command_list = commands.split(";").collect::<Vec<_>>();
    if (command_list.is_empty()) {
        debug!("nothing to do");
        exit(1);
    }
    for command in command_list {
        if command.is_empty() {
            debug!("skipping empty command");
            continue;
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
    trace!("command executing is done, format is '{}', prepend is '{}'", format, prepend0);
    // FIXME refactor format parsing
    let output = match format {
        "0x" | "0h" => main_buffer.to_string(16, true, prepend0),
        "x" | "h" => main_buffer.to_string(16, false, prepend0),
        "0d" => main_buffer.to_string(10, true, prepend0),
        "d" => main_buffer.to_string(10, false, prepend0),
        "0o" => main_buffer.to_string(8, true, prepend0),
        "o" => main_buffer.to_string(8, false, prepend0),
        "0b" => main_buffer.to_string(2, true, prepend0),
        "b" => main_buffer.to_string(2, false, prepend0),
        _ => "".to_owned()
    };
    output
}

fn main() {
    // TODO make struct with parameters https://github.com/clap-rs/clap
    // TODO make a cli.yml (https://docs.rs/clap/2.33.3/clap/), and use it like this: let yaml = load_yaml!("cli.yml"); let matches = App::from_yaml(yaml).get_matches();
    let matches = command!()
        .version("0.2.1")
        .arg(Arg::new("v")
            .short('v')
            .multiple_occurrences(true)// multiple_values?
            .help("Sets the level of verbosity. More 'v's, more verbosity. Four 'v' are used for the most verbose logging."))
        .arg(Arg::new("history")
            .long("history")
            .short('h')
            .default_value("100")
            .takes_value(true)
            .help("Sets the size of the command history."))
        .arg(Arg::new("append_output")
            .long("append-output")
            .short('a')
            .takes_value(false)
            .help("If given, the text output will appended after every command, not overwritten."))
        .arg(Arg::new("expression")
            .long("expression")
            .short('e')
            .takes_value(true)
            .allow_hyphen_values(true)
            .help("Enables batch mode. Commands to execute, separated by ';' are mandatory"))
        .arg(Arg::new("format") // TODO this functionality must be implemented in `printf` operator
            .long("format")
            .short('f')
            .takes_value(true)
            .default_value("0b")
            .help("Specifies prefix for output: b -binary, o - octal, d - decimal, h - hexadecimal. 0f - with prefix, where f is (b|o|d|h)+."))
        .arg(Arg::new("prepend0")
            .long("prepend0")
            .short('p')
            .takes_value(false)
            .help("If given the output will be prepended with zeroes."))
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

    let append_output = matches.is_present("append_output");
    stderrlog::new().module(module_path!()).verbosity(verbosity_level).init().unwrap();

    match matches.value_of("expression") {
        Some(commands) => {
            let format = matches.value_of("format").unwrap();
            let prepend0 = matches.is_present("prepend0");
            let output = not_interactive_routine(commands, format, prepend0);
            if !output.is_empty() {
                println!("{}", output);
            }
        },
        None => {
            interactive_routine(history_size, append_output);
        }
    }
}

#[test]
pub fn empty_commands_dont_cause_any_errors() {
    assert!(not_interactive_routine("", "", false).is_empty());
    assert!(not_interactive_routine(";", "", false).is_empty());
    assert!(not_interactive_routine(";;", "", false).is_empty());
    assert!(not_interactive_routine("1", "", false).is_empty());
    assert_eq!("0", not_interactive_routine(";;;", "d", false));
    assert_eq!("1", not_interactive_routine("1", "d", false));
    assert_eq!("1", not_interactive_routine("1;", "d", false));
    assert_eq!("1", not_interactive_routine("1;", "d", false));
}
