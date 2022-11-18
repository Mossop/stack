use crate::{
    config::{Config, Stack},
    exec::ExecOptions,
    program::GlobalArguments,
};

use std::process::Command;

fn exec(exec_options: &ExecOptions, stack: &Stack) -> Result<(), String> {
    let exec_options = exec_options.with_stack(stack);

    log::debug!(
        "Executing `{} {}`",
        exec_options.program(),
        exec_options.args().join(" ")
    );

    let mut command = Command::new(exec_options.program());
    command.args(exec_options.args());
    for (k, v) in exec_options.environment.iter() {
        command.env(k, v);
    }

    command.current_dir(&exec_options.working_dir);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Error running docker compose: {}", e))?;
    let status = child
        .wait()
        .map_err(|e| format!("Error running docker compose: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Error running command `{} {}`: {}",
            exec_options.program(),
            exec_options.args().join(" "),
            status
        ))
    }
}

pub fn normal_order_run(
    command: &str,
    globals: &GlobalArguments,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    let stacks = globals.stacks();

    log::trace!(
        "Executing command `{}` in normal order against {} stacks with arguments {:?}",
        command,
        if !stacks.is_empty() {
            format!("`{}`", stacks.join(","))
        } else {
            "all".to_string()
        },
        args
    );

    let exec_options = ExecOptions::new(config, command, args);
    let stacks = config.stacks(stacks, true)?;
    for stack in stacks {
        exec(&exec_options, stack)?;
    }

    Ok(())
}

pub fn reverse_order_run(
    command: &str,
    globals: &GlobalArguments,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    let stacks = globals.stacks();

    log::trace!(
        "Executing command `{}` in reverse order against stacks {} with arguments {:?}",
        command,
        if !stacks.is_empty() {
            format!("`{}`", stacks.join(","))
        } else {
            "all".to_string()
        },
        args
    );

    let exec_options = ExecOptions::new(config, command, args);
    let mut stacks = config.stacks(stacks, true)?;
    stacks.reverse();
    for stack in stacks {
        exec(&exec_options, stack)?;
    }

    Ok(())
}

pub fn single_stack(
    command: &str,
    globals: &GlobalArguments,
    config: &Config,
    args: &Vec<String>,
) -> Result<(), String> {
    let stacks = config.stacks(globals.stacks(), false)?;
    if stacks.len() != 1 {
        return Err(format!(
            "Command {} can only operate on one stack but {} were provided.",
            command,
            stacks.len()
        ));
    }

    let stack = stacks.get(0).unwrap();

    log::trace!(
        "Executing command `{}` against {} with arguments {:?}",
        command,
        stack.key,
        args
    );

    let exec_options = ExecOptions::new(config, command, args);
    exec(&exec_options, stack)
}
