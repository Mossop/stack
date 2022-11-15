use crate::{config::Config, program::Globals};

pub fn normal_order_run(
    command: &str,
    globals: &Globals,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    log::trace!(
        "Executing command `{}` in normal order against {} stacks with arguments {:?}",
        command,
        if let Some(ref s) = globals.stacks {
            format!("`{}`", s)
        } else {
            "all".to_string()
        },
        args
    );
    Ok(())
}

pub fn reverse_order_run(
    command: &str,
    globals: &Globals,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    log::trace!(
        "Executing command `{}` in reverse order against stacks {:?} with arguments {:?}",
        command,
        if let Some(ref s) = globals.stacks {
            format!("`{}`", s)
        } else {
            "all".to_string()
        },
        args
    );
    Ok(())
}

pub fn single_stack(
    command: &str,
    globals: &Globals,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    log::trace!(
        "Executing command `{}` against a single stack with arguments {:?}",
        command,
        args
    );
    Ok(())
}
