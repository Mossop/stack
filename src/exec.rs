use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::{Config, Stack};

#[derive(Default, Clone)]
pub struct ExecOptions {
    pub binary: Vec<String>,
    pub global_args: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    pub environment: HashMap<String, String>,
    pub working_dir: PathBuf,
}

impl ExecOptions {
    pub fn new<S: AsRef<str>>(config: &Config, command: &str, args: &[S]) -> Self {
        Self {
            binary: config.command.clone(),
            command: command.to_owned(),
            working_dir: config.base_dir.clone(),
            args: args.iter().map(|s| s.as_ref().to_string()).collect(),
            ..Default::default()
        }
    }

    pub fn with_stack(&self, stack: &Stack) -> Self {
        let mut options = self.clone();

        options
            .global_args
            .extend(["-p".to_string(), stack.name.clone()]);
        options.working_dir = stack.directory(&self.working_dir);
        options.environment.extend(
            stack
                .environment
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        options
    }

    pub fn program(&self) -> &str {
        self.binary.get(0).unwrap()
    }

    pub fn args(&self) -> Vec<&str> {
        let mut args: Vec<&str> = self
            .binary
            .iter()
            .skip(1)
            .chain(self.global_args.iter())
            .map(AsRef::<str>::as_ref)
            .collect();

        args.push(self.command.as_ref());

        args.extend(self.args.iter().map(AsRef::<str>::as_ref));

        args
    }
}
