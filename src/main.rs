use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

mod api;
mod bot;
mod db;
mod models;
mod queries;
mod services;
mod tools;
mod utils;

use crate::models::{AppConfig, PaymentConfig};
use crate::services::association::AssociationContext;
use crate::services::browser::{create_payment_checker, PaymentChecker};
use crate::services::calendar::create_calendar_service;
use botframework::infisical::{load_infisical_secrets, InfisicalConfig};
use botframework::telegram::start_bot;
use botframework::utils::run_health_check;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = botframework::init();

    // Handle --check
    if cli.check {
        return run_health_check().await;
    }

    // Handle --migrate (database is already initialized by open_db)
    if cli.migrate {
        // TODO: This
        info!("Database initialized successfully");
        return Ok(());
    }

    // Load Infisical secrets if configured
    let mut secrets = load_infisical_secrets(InfisicalConfig::from_env()).await;

    // Load configuration
    let config = load_config(&mut secrets)?;
    info!("Configuration loaded");

    // Create association context
    // Note: For rusqlite, we open a new connection per context since Connection is not Send
    let context = AssociationContext::new("assoc", &config, &mut secrets)
        .await
        .context("Failed to create association context")?;
    info!("Association context initialized");

    // Create payment checker if configured
    let payment_checker =
        PaymentConfig::from("assoc", &mut secrets).and_then(|c| create_payment_checker(c));

    if payment_checker.is_some() {
        info!("Payment checker initialized");
    }

    // Create calendar service if configured
    let calendar_key = config.data_path.join("assoc/calendar.key");
    let calendar_service = create_calendar_service(&calendar_key).await;

    // Clone context for tasks
    let api_context = context.clone();
    let payment_context = context.clone();
    let calendar_context = context.clone();
    let bot_context = context.clone();

    // Start background tasks
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = run_api(api_context, config.port).await {
            error!("API server error: {}", e);
        }
    });

    let _payment_handle = if payment_checker.is_some() {
        Some(tokio::spawn(async move {
            run_payment_checker(payment_context, payment_checker.unwrap()).await;
        }))
    } else {
        None
    };

    let _calendar_handle = if calendar_service.is_some() {
        Some(tokio::spawn(async move {
            run_assembly_reminders(calendar_context, calendar_service.unwrap()).await;
        }))
    } else {
        None
    };

    // Run the bot (this blocks)
    info!("Starting Telegram bot...");
    let result = start_bot(bot_context).await;

    // Shutdown signal handling
    let _ = signal::ctrl_c().await;
    info!("Shutting down...");

    result
}

/// Load configuration from environment variables
fn load_config(secrets: &mut HashMap<String, String>) -> Result<AppConfig> {
    Ok(AppConfig {
        port: botframework::get_port(),

        groq_key: secrets
            .remove("GROQ_API_KEY")
            .expect("GROQ_API_KEY in infisical is needed"),

        data_path: PathBuf::from(
            std::env::var("DATA_PATH").unwrap_or_else(|_| "/data".to_string()),
        ),
    })
}

/// Run the API server
async fn run_api(context: Arc<RwLock<AssociationContext>>, port: u16) -> Result<()> {
    let router = api::create_router(context.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("API server listening on port {}", port);

    axum::serve(listener, router).await?;

    Ok(())
}

/// Run payment checker in background
async fn run_payment_checker(
    context: Arc<RwLock<AssociationContext>>,
    checker: crate::services::browser::APaymentChecker,
) {
    let mut interval = interval(Duration::from_secs(600)); // 10 minutes

    loop {
        interval.tick().await;

        let payments = checker.get_last_payments().await;

        for user in payments {
            // Mark user as paid using AssociationContext
            let user_clone = user.clone();
            let result = context
                .read()
                .await
                .legal_name_set_paid(&user_clone, 1)
                .await;

            match result {
                Ok(()) => info!("Marked {} as paid", user),
                Err(e) => warn!("Failed to mark {} as paid: {}", user, e),
            }
        }
    }
}

/// Run assembly reminders in background
async fn run_assembly_reminders(
    context: Arc<RwLock<AssociationContext>>,
    calendar: crate::services::calendar::CalendarService,
) {
    let mut interval = interval(Duration::from_secs(21600)); // 6 hours

    loop {
        interval.tick().await;

        match calendar.check_today_assembly().await {
            Some(assembly) => {
                info!("Found assembly today: {}", assembly.summary);

                // Get all associate emails
                match context.read().await.get_all_associate_emails().await {
                    Ok(emails) if !emails.is_empty() => {
                        let reminder_data = crate::services::calendar::AssemblyReminderData {
                            date: assembly
                                .date_time
                                .map(|dt| dt.format("%d/%m/%Y").to_string())
                                .unwrap_or_else(|| "Hoy".to_string()),
                            time: assembly
                                .date_time
                                .map(|dt| dt.format("%H:%M").to_string())
                                .unwrap_or_else(|| "Todo el día".to_string()),
                            description: assembly.description,
                        };

                        let subject =
                            format!("Recordatorio: Asamblea hoy - {}", reminder_data.date);
                        let body = crate::services::calendar::create_assembly_reminder_email(
                            &reminder_data,
                        );

                        if let Err(e) = context
                            .read()
                            .await
                            .send_group_mail(emails, &subject, &body, None)
                            .await
                        {
                            error!("Failed to send assembly reminder emails: {}", e);
                        } else {
                            info!("Sent assembly reminders");
                        }
                    }
                    Ok(_) => {
                        warn!("No associates found to send reminders to");
                    }
                    Err(e) => {
                        error!("Failed to get associate emails: {}", e);
                    }
                }
            }
            None => {
                info!("No assembly found for today");
            }
        }
    }
}
