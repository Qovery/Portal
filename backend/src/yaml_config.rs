use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::constants::DEFAULT_TIMEOUT_IN_SECONDS;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct YamlConfig {
    pub self_service: SelfServiceYamlConfig,
}

impl YamlConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.self_service.validate()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceYamlConfig {
    pub sections: Vec<SelfServiceSectionYamlConfig>,
}

impl SelfServiceYamlConfig {
    pub fn validate(&self) -> Result<(), String> {
        for section in &self.sections {
            section.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceSectionYamlConfig {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub actions: Option<Vec<SelfServiceSectionActionYamlConfig>>,
}

impl SelfServiceSectionYamlConfig {
    pub fn validate(&self) -> Result<(), String> {
        let _ = validate_slug(&self.slug)?;

        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }

        for service in self.actions.as_ref().unwrap_or(&vec![]) {
            service.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceSectionActionYamlConfig {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub fields: Option<Vec<SelfServiceSectionActionFieldYamlConfig>>,
    pub validate: Option<Vec<SelfServiceSectionActionValidateYamlConfig>>,
    pub post_validate: Option<Vec<SelfServiceSectionActionPostValidateYamlConfig>>,
}

impl SelfServiceSectionActionYamlConfig {
    pub fn validate(&self) -> Result<(), String> {
        let _ = validate_slug(&self.slug)?;

        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }

        if self.fields.is_none() && self.validate.is_none() && self.post_validate.is_none() {
            return Err("fields, validate and post_validate are empty".to_string());
        }

        if let Some(fields) = &self.fields {
            for field in fields {
                field.validate()?;
            }
        }

        if let Some(validate) = &self.validate {
            for validate_script in validate {
                validate_script.validate()?;
            }
        }

        if let Some(post_validate) = &self.post_validate {
            for post_validate_script in post_validate {
                post_validate_script.validate()?;
            }
        }

        Ok(())
    }
}

pub trait ExternalCommand {
    fn get_command(&self) -> &Vec<String>;
    fn get_timeout(&self) -> u64;
    fn validate(&self) -> Result<(), String> {
        if self.get_command().is_empty() {
            return Err("command is empty".to_string());
        }

        // check if command is valid by checking if the first element (binary) exists and is executable by the current user
        if self.get_command().len() >= 1 {
            let command = self.get_command().get(0).unwrap();
            if !which::which(command).is_ok() {
                return Err(format!("command '{}' not found", command));
            }
        }

        // check if the second element (file) exists and is executable by the current user
        if self.get_command().len() >= 2 {
            let file = self.get_command().get(1).unwrap();
            if !std::path::Path::new(file).exists() {
                return Err(format!("file '{}' not found", file));
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceSectionActionValidateYamlConfig {
    pub command: Vec<String>,
    pub timeout: Option<u64>,
}

impl ExternalCommand for SelfServiceSectionActionValidateYamlConfig {
    fn get_command(&self) -> &Vec<String> {
        &self.command
    }

    fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(DEFAULT_TIMEOUT_IN_SECONDS)
    }
}

impl Display for SelfServiceSectionActionValidateYamlConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = self.command.join(" ");
        write!(f, "{}", command)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceSectionActionPostValidateYamlConfig {
    pub command: Vec<String>,
    pub timeout: Option<u64>,
    pub output_model: Option<String>,
}

impl ExternalCommand for SelfServiceSectionActionPostValidateYamlConfig {
    fn get_command(&self) -> &Vec<String> {
        &self.command
    }

    fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(DEFAULT_TIMEOUT_IN_SECONDS)
    }
}

impl Display for SelfServiceSectionActionPostValidateYamlConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = self.command.join(" ");
        write!(f, "{}", command)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SelfServiceSectionActionFieldYamlConfig {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    #[serde(rename = "type")]
    pub type_: ActionFieldType,
    pub default: Option<String>,
    pub required: Option<bool>,
    pub autocomplete_fetcher: Option<String>,
}

impl SelfServiceSectionActionFieldYamlConfig {
    pub fn validate(&self) -> Result<(), String> {
        let _ = validate_slug(&self.slug)?;

        if self.title.is_empty() {
            return Err("title is empty".to_string());
        }

        if self.type_ == ActionFieldType::List && self.autocomplete_fetcher.is_none() {
            return Err("autocomplete_fetcher is required for type List".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ActionFieldType {
    Text,
    Textarea,
    Number,
    Boolean,
    Date,
    Datetime,
    Time,
    List,
}

fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err("slug is empty".to_string());
    }

    if slug.contains(" ") || !slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("slug should only contains alphanumeric characters and dashes".to_string());
    }

    Ok(())
}
