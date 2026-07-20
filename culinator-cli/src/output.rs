use anyhow::Result;
use clap::ValueEnum;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    Jsonl,
}

impl OutputFormat {
    pub fn value<T: Serialize>(&self, value: &T) -> Result<()> {
        match self {
            Self::Human => println!("{}", serde_json::to_string_pretty(value)?),
            Self::Json => println!("{}", serde_json::to_string_pretty(value)?),
            Self::Jsonl => println!("{}", serde_json::to_string(value)?),
        }
        Ok(())
    }

    pub fn values<T: Serialize>(&self, values: &[T], human: impl Fn(&T) -> String) -> Result<()> {
        match self {
            Self::Human => {
                for value in values {
                    println!("{}", human(value));
                }
            }
            Self::Json => println!("{}", serde_json::to_string_pretty(values)?),
            Self::Jsonl => {
                for value in values {
                    println!("{}", serde_json::to_string(value)?);
                }
            }
        }
        Ok(())
    }

    pub fn message<T: Serialize>(&self, message: &str, value: &T) -> Result<()> {
        match self {
            Self::Human => println!("{message}"),
            Self::Json | Self::Jsonl => self.value(value)?,
        }
        Ok(())
    }
}
