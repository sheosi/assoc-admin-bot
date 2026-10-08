use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

/// Configuration for runtime settings
#[derive(Debug, Clone)]
pub struct RunningConf {
    pub admins: HashSet<String>,
}

impl RunningConf {
    pub fn new() -> Self {
        Self {
            admins: HashSet::new(),
        }
    }

    pub fn is_admin(&self, name: &str) -> bool {
        self.admins.contains(name)
    }

    pub fn add_admin(&mut self, name: String) {
        self.admins.insert(name);
    }

    pub fn remove_admin(&mut self, name: &str) {
        self.admins.remove(name);
    }
}

impl Default for RunningConf {
    fn default() -> Self {
        Self::new()
    }
}

/// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

impl EmailConfig {
    pub fn from_secrets(
        prefix: &str,
        secrets: &mut HashMap<String, String>,
    ) -> Option<EmailConfig> {
        let mut extract = |name: &str| secrets.remove(&format!("{}_{}", prefix, name));

        Some(EmailConfig {
            smtp_server: extract("SMTP_SERVER")?,
            smtp_port: extract("SMTP_PORT")
                .unwrap_or_else(|| "587".to_string())
                .parse()
                .expect("Couldn't parse port"),
            username: extract("SMTP_USERNAME")?,
            password: extract("SMTP_PASSWORD")?,
            from_address: extract("EMAIL_ADDRESS")?,
        })
    }
}

/// Browser/Payment checker configuration
#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub ruralvia_user: String,
    pub ruralvia_pass: String,
}

impl PaymentConfig {
    pub fn from(prefix: &str, secrets: &mut HashMap<String, String>) -> Option<PaymentConfig> {
        Some(PaymentConfig {
            ruralvia_user: secrets.remove(&format!("{}_BANK_USER", prefix))?,
            ruralvia_pass: secrets.remove(&format!("{}_BANK_PASS", prefix))?,
        })
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub groq_key: String,
    pub data_path: PathBuf,
}

/// Tool call fill state for interactive tool input
pub struct ToolCallFillState {
    pub on_message: Box<dyn FnMut(String) + Send + Sync>,
}

/// Webhook payload for forms.app
#[derive(Debug, Deserialize)]
pub struct FormsAppWebhook {
    pub form: FormsAppWebhookForm,
    pub answer: FormsAppWebhookAnswer,
}

#[derive(Debug, Deserialize)]
pub struct FormsAppWebhookForm {}

#[derive(Debug, Deserialize)]
pub struct FormsAppWebhookAnswer {
    pub answers: Vec<FormsAppWebhookAnswerItem>,
}

#[derive(Debug, Deserialize)]
pub struct FormsAppWebhookAnswerItem {
    #[serde(rename = "t")]
    pub text: Option<String>,
    #[serde(rename = "fn")]
    pub fullname: Option<FormsAppFullname>,
    #[serde(rename = "c")]
    pub selection: Option<Vec<FormsAppSelection>>,
}

#[derive(Debug, Deserialize)]
pub struct FormsAppFullname {
    #[serde(rename = "f")]
    pub first_name: String,
    #[serde(rename = "l")]
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct FormsAppSelection {
    #[serde(rename = "t")]
    pub text: String,
}
