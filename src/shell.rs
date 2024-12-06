use clap::ValueEnum;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T, E = ShellError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("failed to find the home directory")]
    FindError,
    #[error("failed to open the history file `{0}`")]
    OpenError(String),
    #[error("filename contains invalid UTF-8")]
    InvalidUTF8,
    #[error("failed to read a line")]
    ReadError,
    #[error("failed to parse the history file `{0}`")]
    ParseError(String),
    #[error("for some reason, the command count failed")]
    CountError,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum ShellType {
    /// the `.zsh_history` file in the current user's home directory
    Zsh,
    /// the `.bash_history` file in the current user's home directory
    Bash,
}

impl ShellType {
    /// Find the path to the history file in the current user's home directory
    fn find_history_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ShellError::FindError)?;
        Ok(match &self {
            ShellType::Zsh => home.join(".zsh_history"),
            ShellType::Bash => home.join(".bash_history"),
        })
    }

    /// Open the history file in the current user's home directory
    fn open_history_file(&self) -> Result<File> {
        let history_path = self.find_history_path()?;
        File::open(&history_path).map_err(|_| {
            let history_path = history_path.to_str();

            if let Some(history_path) = history_path {
                ShellError::OpenError(history_path.to_string())
            } else {
                ShellError::InvalidUTF8
            }
        })
    }
}

impl Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let res = match &self {
            ShellType::Zsh => "zsh".to_string(),
            ShellType::Bash => "bash".to_string(),
        };
        write!(f, "{res}")
    }
}

pub struct Shell {
    /// The type of shell to read the history from
    shell_type: ShellType,
    /// The command count in the history file
    /// to find the command amount, use `command_frequency` or `commands_ran`
    pub invocation_count: Option<usize>,
}

impl Shell {
    pub fn from_custom(path: &str) -> Self {
        todo!()
        // Self {
        //     shell_type: Custom,
        //     invocation_count: None,
        // }
    }

    /// Sets the `invocation_count` field and returns it,
    /// or a `ShellError` on failure
    pub fn commands_ran(&mut self) -> Result<usize> {
        let history_path = self.shell_type.find_history_path()?;

        let file = File::open(&history_path).map_err(|_| {
            let history_path = history_path.to_str();

            if let Some(history_path) = history_path {
                ShellError::OpenError(history_path.to_string())
            } else {
                ShellError::InvalidUTF8
            }
        })?;

        let line_count = BufReader::new(file).lines().count();
        self.invocation_count = Some(line_count);

        self.invocation_count.ok_or(ShellError::CountError)
    }

    /// Returns a map of the frequency of each command
    pub fn command_frequency(&self) -> Result<HashMap<String, usize>> {
        let file = self.shell_type.open_history_file()?;

        let buf = BufReader::new(file);
        let mut freq = HashMap::new();
        buf.lines()
            .collect::<std::io::Result<Vec<String>>>()
            .map_err(|_| ShellError::ReadError)?
            .into_iter()
            .for_each(|line| {
                // TODO: add support for | and && and \ commands
                // get the first command that isn't a VAR
                let Some(command) = line
                    .split(' ')
                    .filter(|arg| !arg.contains('=') && !arg.is_empty())
                    .nth(0)
                else {
                    // continue interating in for_each
                    return;
                };

                *freq.entry(command.to_string()).or_insert(0) += 1;
            });

        Ok(freq)
    }

    /// Returns a map of the frequency of each invocation and sets the `invocation_count` field
    pub fn invocation_frequency(&mut self) -> Result<HashMap<String, usize>> {
        let file = self.shell_type.open_history_file()?;

        let buf = BufReader::new(file);
        let mut freq = HashMap::new();
        let mut count = 0;
        buf.lines()
            .collect::<std::io::Result<Vec<String>>>()
            .map_err(|_| ShellError::ReadError)?
            .into_iter()
            .for_each(|line| {
                count += 1;
                *freq.entry(line).or_insert(0) += 1;
            });

        self.invocation_count = Some(count);

        Ok(freq)
    }

    /// Returns the top five commands and invocations in the history file
    /// Left: the top five commands
    /// Right: the top five invocations
    pub fn top_commands_and_invocations(&mut self) -> Result<(Vec<String>, Vec<String>)> {
        // sorts by most executed
        // TODO: for values that are the same sort by name
        let mut invocation_freq = self
            .invocation_frequency()?
            .into_iter()
            .collect::<Vec<(String, usize)>>();
        invocation_freq.sort_by(|(_, b), (_, d)| d.cmp(b));
        invocation_freq.truncate(5);

        let invocations = invocation_freq
            .into_iter()
            .map(|(a, _)| a)
            .collect::<Vec<String>>();

        let mut command_freq = self
            .command_frequency()?
            .into_iter()
            .collect::<Vec<(String, usize)>>();
        command_freq.sort_by(|(_, b), (_, d)| d.cmp(b));
        command_freq.truncate(5);

        let commands = command_freq
            .into_iter()
            .map(|(a, _)| a)
            .collect::<Vec<String>>();

        println!("{commands:?}\n\n\n{invocations:?}");

        Ok((commands, invocations))
    }
}

impl From<ShellType> for Shell {
    fn from(value: ShellType) -> Self {
        Self {
            shell_type: value,
            invocation_count: None,
        }
    }
}
