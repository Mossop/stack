use crate::{
    config::{Config, Stack},
    exec::ExecOptions,
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

pub fn run_against_stacks(
    command: &str,
    config: &Config,
    stacks: &Vec<&Stack>,
    args: &Vec<String>,
) -> Result<(), String> {
    log::trace!(
        "Executing command `{}` against {} stacks with arguments {:?}",
        command,
        stacks
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>()
            .join(","),
        args
    );

    let exec_options = ExecOptions::new(config, command, args);
    for stack in stacks {
        exec(&exec_options, stack)?;
    }

    Ok(())
}
