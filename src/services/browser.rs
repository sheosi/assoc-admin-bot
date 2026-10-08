//! Browser automation service for payment checking
//! Uses thirtyfour (Selenium WebDriver) to check Ruralvia bank for new payments

use anyhow::{Context, Result};
use std::time::Duration;
use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

use crate::models::PaymentConfig;

pub struct Browser {
    driver: WebDriver,
}

impl Browser {
    /// Create a new browser instance with Chrome in headless mode
    pub async fn new() -> Result<Self> {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--headless")?;
        caps.add_arg("--no-sandbox")?;
        caps.add_arg("--disable-dev-shm-usage")?;
        caps.add_arg("--disable-gpu")?;
        caps.add_arg("--window-size=1920,1080")?;

        let driver = WebDriver::new("http://localhost:4444", caps)
            .await
            .context(
                "Failed to connect to ChromeDriver. Make sure chromedriver is running on port 4444",
            )?;

        Ok(Self { driver })
    }

    /// Input text into an element found by CSS selector
    async fn input_text(&self, selector: &str, text: &str) -> Result<()> {
        let element = self.driver.find(By::Css(selector)).await?;
        element.send_keys(text).await?;
        Ok(())
    }

    /// Click a button found by XPath
    async fn click_button_xpath(&self, xpath: &str) -> Result<()> {
        let button = self.driver.find(By::XPath(xpath)).await?;
        button.click().await?;
        Ok(())
    }

    /// Get all texts from elements found by XPath
    async fn get_all_texts_xpath(&self, xpath: &str) -> Result<Vec<String>> {
        let elements = self.driver.find_all(By::XPath(xpath)).await?;
        let mut texts = Vec::new();

        for element in elements {
            let text = element.text().await?;
            texts.push(text);
        }

        Ok(texts)
    }

    /// Wait for page to load
    async fn wait_page_load(&self) {
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    /// Login to Ruralvia
    async fn login_ruralvia(&self, user: &str, pass: &str) -> Result<()> {
        self.driver
            .goto("https://bancadigital.ruralvia.com/CA-FRONT/NBE/web/particulares/#/login")
            .await?;

        self.input_text("[name=\"dniNie\"]", user).await?;
        self.input_text("[name=\"Alias\"]", pass).await?;

        self.click_button_xpath("//form//button[@type=\"submit\"]")
            .await?;

        self.wait_page_load().await;

        Ok(())
    }

    /// Navigate to account page
    async fn go_to_account(&self) -> Result<()> {
        self.click_button_xpath("//button[.//span[text() = 'CUENTA CORRIENTE']]")
            .await?;
        self.wait_page_load().await;
        Ok(())
    }

    /// List paid transactions from the account
    async fn list_payed(&self) -> Result<Vec<String>> {
        let texts = self
            .get_all_texts_xpath("//ol[1]/li/div[1]/button[1]/span[2]/p[1]")
            .await?;

        let results: Vec<String> = texts
            .into_iter()
            .map(|t| t.trim_start_matches("Trf. ").to_string())
            .collect();

        Ok(results)
    }

    /// Check for new payments
    pub async fn check_new_payments(user: &str, pass: &str) -> Result<Vec<String>> {
        let browser = Browser::new().await?;

        browser.login_ruralvia(user, pass).await?;
        browser.go_to_account().await?;

        let payed = browser.list_payed().await?;

        // Close browser
        browser.driver.quit().await?;

        Ok(payed)
    }
}

/// Payment checker for periodic payment checking
pub struct RuralviaChecker {
    pub user: String,
    pub pass: String,
}

impl RuralviaChecker {
    pub fn new(user: String, pass: String) -> Self {
        Self { user, pass }
    }

    /// Get last payments from Ruralvia
    pub async fn get_last_payments(&self) -> Vec<String> {
        match Browser::check_new_payments(&self.user, &self.pass).await {
            Ok(payments) => payments,
            Err(e) => {
                tracing::error!("Failed to check payments: {}", e);
                Vec::new()
            }
        }
    }
}

/// Payment checker trait for dependency injection
#[enum_dispatch::enum_dispatch]
pub trait PaymentChecker: Send + Sync {
    async fn get_last_payments(&self) -> Vec<String>;
}

impl PaymentChecker for RuralviaChecker {
    async fn get_last_payments(&self) -> Vec<String> {
        self.get_last_payments().await
    }
}

#[enum_dispatch::enum_dispatch(PaymentChecker)]
pub enum APaymentChecker {
    RuralviaChecker,
}

/// Create a payment checker if credentials are provided
pub fn create_payment_checker(config: PaymentConfig) -> Option<APaymentChecker> {
    if !config.ruralvia_user.is_empty() && !config.ruralvia_pass.is_empty() {
        Some(APaymentChecker::RuralviaChecker(RuralviaChecker::new(
            config.ruralvia_user,
            config.ruralvia_pass,
        )))
    } else {
        None
    }
}
