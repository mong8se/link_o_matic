use once_cell::sync::OnceCell;
use std::path::PathBuf;

use std::io::{stderr, stdin, stdout, Write};
use std::process::exit;

use owo_colors::{OwoColorize, Style};

use crate::fs::get_dot_path;

use crate::delete::DeleteOptions;

pub struct MessageBuilder<'a> {
    log_level: LogLevel,
    path: Option<String>,
    verb: String,
    handle: &'a Messenger,
}

impl MessageBuilder<'_> {
    pub fn with_path(mut self, path: &PathBuf) -> Self {
        self.path = Some(relative_dot_file(path));
        self
    }

    pub fn with_verb(mut self, verb: &str) -> Self {
        self.verb = verb.to_string();
        self
    }

    pub fn conjugate_with(mut self, ending: &str) -> Self {
        self.verb = conjugate_with(&self.verb, &ending);
        self
    }

    pub fn error(mut self, rest: Option<String>) -> String {
        if self.verb.len() < 1 {
            self.verb = String::from("error")
        }
        self.log_level = LogLevel::Error;
        self.log(rest);
        exit(1);
    }

    pub fn warning(mut self, rest: Option<String>) {
        self.log_level = LogLevel::Warning;
        self.log(rest);
    }

    pub fn success(mut self, rest: Option<String>) {
        self.log_level = LogLevel::Success;
        self.log(rest);
    }

    pub fn log(self, rest: Option<String>) {
        self.handle.log_message(self, rest);
    }
}

pub struct Messenger {
    normal_style: Style,
    success_style: Style,
    warning_style: Style,
    error_style: Style,
}

impl Messenger {
    fn init() -> Messenger {
        let style = Style::new();

        Messenger {
            normal_style: style.white(),
            success_style: style.blue(),
            warning_style: style.green(),
            error_style: style.red(),
        }
    }

    pub fn get_instance() -> &'static Messenger {
        INSTANCE.get_or_init(|| Self::init())
    }

    pub fn new<'a>() -> MessageBuilder<'a> {
        MessageBuilder {
            log_level: LogLevel::Normal,
            path: None,
            verb: String::new(),
            handle: Self::get_instance(),
        }
    }

    fn log_message(&self, options: MessageBuilder, rest: Option<String>) {
        let styled_verb = options.verb.style(match options.log_level {
            LogLevel::Normal => self.normal_style,
            LogLevel::Warning => self.warning_style,
            LogLevel::Success => self.success_style,
            LogLevel::Error => self.error_style,
        });

        let result = format!(
            "{:>9} {}\n",
            styled_verb.bold(),
            join_line([options.path, rest])
        );

        let bytes = result.as_bytes();

        match options.log_level {
            LogLevel::Error => stderr().write_all(bytes).unwrap(),
            _ => stdout().write_all(bytes).unwrap(),
        };
    }
}

static INSTANCE: OnceCell<Messenger> = OnceCell::new();

fn join_line(list: [Option<String>; 2]) -> String {
    list.iter()
        .filter_map(|l| l.to_owned())
        .collect::<Vec<String>>()
        .join(" | ")
}

fn relative_dot_file(entry: &PathBuf) -> String {
    entry
        .to_path_buf()
        .strip_prefix(&get_dot_path(None))
        .unwrap()
        .display()
        .to_string()
}

enum LogLevel {
    Normal,
    Warning,
    Error,
    Success,
}

pub fn delete_prompt_help() {
    println!("{}", "y - yes , n - no, a - all, q - quit".yellow());
}

pub fn conjugate_with(template: &str, ending: &str) -> String {
    if !template.contains('%') {
        Messenger::new().error(Some(format!("template must contain a %")));
    }
    let parts: Vec<&str> = template.split('%').collect();

    format!("{}{}{}", parts[0], ending, parts[1])
}

const DEFAULT_CHOICE: char = 'n';

pub fn display_delete_prompt(name: &PathBuf, options: &DeleteOptions) -> char {
    let mut input = String::new();

    print!(
        "{:>9} {} ? [ynaq] ",
        conjugate_with(&options.verb_template, &"e").bold(),
        relative_dot_file(name)
    );

    stdout().flush().unwrap_or_else(|err| {
        eprintln!("Problem flushing stdout: {:?}", err);
        exit(1);
    });
    stdin().read_line(&mut input).unwrap();

    let result = input.trim().chars().nth(0).unwrap_or(DEFAULT_CHOICE);

    if result == 'q' {
        Messenger::new()
            .with_verb("quitting")
            .log(Some(String::from("per user")));
        exit(0);
    } else if result == 'y' || result == 'a' {
        return result;
    } else if result == '?' {
        delete_prompt_help();
        return display_delete_prompt(name, options);
    }

    return DEFAULT_CHOICE;
}
