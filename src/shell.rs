use clap::ValueEnum;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T, E = ShellError> = std::result::Result<T, E>;

#[allow(dead_code)]
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
    /// Find the history file in the current user's home directory
    fn find_history_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ShellError::FindError)?;
        Ok(match &self {
            ShellType::Zsh => home.join(".zsh_history"),
            ShellType::Bash => home.join(".bash_history"),
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
    pub command_count: Option<usize>,
}

impl Shell {
    /// Sets the `command_count` field and returns it, 
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
        self.command_count = Some(line_count);

        self.command_count.ok_or(ShellError::CountError)
    }
    /// Returns a map of the frequency of each command and sets the `command_count` field
    pub fn command_frequency(&mut self) -> Result<HashMap<String, usize>> {
        let history_path = self.shell_type.find_history_path()?;

        let file = File::open(&history_path).map_err(|_| {
            let history_path = history_path.to_str();

            if let Some(history_path) = history_path {
                ShellError::OpenError(history_path.to_string())
            } else {
                ShellError::InvalidUTF8
            }
        })?;

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

        self.command_count = Some(count);

        Ok(freq)
    }
}

impl From<ShellType> for Shell {
    fn from(value: ShellType) -> Self {
        Self {
            shell_type: value,
            command_count: None,
        }
    }
}
