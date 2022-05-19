use crate::messages::display_delete_prompt;
// use std::fs::remove_file;
use std::path::PathBuf;

pub struct DeleteOptions {
    pub implode: bool,
    pub without_prompting: bool,
    pub verb_template: Option<String>,
}

pub fn delete_prompt(path: &PathBuf, options: DeleteOptions) -> bool {
    let temp = match options.verb_template {
        Some(txt) => txt,
        None => String::from("delet%"),
    };

    let result = if options.without_prompting {
        'y'
    } else {
        display_delete_prompt(
            path,
            DeleteOptions {
                implode: options.implode,
                without_prompting: options.without_prompting,
                verb_template: Some(temp),
            },
        )
    };

    if result == 'a' {
        // options.without_prompting = true;
    }

    if result == 'y' || result == 'a' {
        // remove_file(path);

        // return true;
        return false
    }
    return false;
}
