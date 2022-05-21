use once_cell::sync::OnceCell;
use std::path::PathBuf;

use std::io::{stderr, stdin, stdout, Write};
use std::process::exit;

use owo_colors::{OwoColorize, Style};

use crate::fs::get_dot_path;

use crate::delete::DeleteOptions;

pub struct MessageBuilder {
    log_level: LogLevel,
    path: Option<String>,
    verb: String,
}

impl MessageBuilder {
    pub fn with_path(mut self, path: &PathBuf) -> Self {
        self.path = Some(relative_dot_file(path));
        self
    }

    pub fn with_verb(mut self, verb: &str) -> Self {
        self.verb = verb.to_string();
        self
    }

    pub fn error(mut self, rest: Option<String>) {
        self.log_level = LogLevel::Error;
        self.log(rest);
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
        Messenger::display().log_message(self, rest);
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

    pub fn display() -> &'static Messenger {
        INSTANCE.get_or_init(|| Messenger::init())
    }

    pub fn new() -> MessageBuilder {
        MessageBuilder {
            log_level: LogLevel::Normal,
            path: None,
            verb: String::new(),
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

pub fn display_delete_prompt(name: &PathBuf, options: &DeleteOptions) -> char {
    let template = &options.verb_template;
    let parts: Vec<&str> = template.split('%').collect();

    let conjugate_with = |ending| format!("{}{}{}", parts[0], ending, parts[1]);

    let mut input = String::new();
    let result = if options.without_prompting.get() {
        "y"
    } else {
        print!(
            "{:>9} {} ? [y, n, a, q] ",
            &conjugate_with(&"e").bold(),
            relative_dot_file(name)
        );

        stdout().flush().unwrap_or_else(|err| {
            eprintln!("Problem flushing stdout: {:?}", err);
            exit(1);
        });
        stdin().read_line(&mut input).unwrap();
        input.trim()
    };

    if result == "q" {
        Messenger::new()
            .with_verb("quitting")
            .log(Some(String::from("per user")));
        exit(0);
    } else if result == "y" || result == "a" {
        Messenger::new()
            .with_verb(&conjugate_with(&"ing"))
            .with_path(name)
            .success(None);
        return result.chars().nth(0).unwrap();
    } else if result == "?" {
        delete_prompt_help();
        return display_delete_prompt(name, options);
    }

    Messenger::new()
        .with_verb(&"skipping")
        .with_path(name)
        .warning(None);
    return 'n';
}
