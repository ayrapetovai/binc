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
use clap::{App, Arg};
use colored::{Colorize, Color};
use std::str::FromStr;
use smallvec::{smallvec, SmallVec};
use std::rc::Rc;
use std::cell::{RefCell, Cell};

fn print_ui(number: &Number) {
    let line = format!(
        "{3:>0$} {4:>1$} {5:>2$}",
        number.number_of_digits_in_radix(16) + 2, number.number_of_digits_in_radix(10) + 2, number.number_of_digits_in_radix(8) + 2,
        number.to_string(16), number.to_string(10), number.to_string(8)
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

fn main() {
    // TODO make struct with parameters https://github.com/clap-rs/clap
    // TODO make a cli.yml (https://docs.rs/clap/2.33.3/clap/), and use it like this: let yaml = load_yaml!("cli.yml"); let matches = App::from_yaml(yaml).get_matches();
    let matches = App::new("binc")
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity. More 'v's, more verbosity. Four 'v' are used for the most verbose output"))
        .arg(Arg::with_name("history")
            .long("history")
            .default_value("100")
            .takes_value(true)
            .help("Set the size of the command history"))
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
                trace!("main: got commands: '{}'", commands);
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
