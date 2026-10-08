//! Google Calendar service for assembly reminders

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, Timelike};
use google_calendar3::api::Event;
use google_calendar3::CalendarHub;
use std::path::Path;
use tracing::{error, info};
use yup_oauth2 as oauth2;

/// Google Calendar service
pub struct CalendarService {
    hub: CalendarHub<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    >,
}

impl CalendarService {
    /// Create a new calendar service from credentials file
    pub async fn new(credentials_path: &Path) -> Result<Self> {
        // Load the service account credentials
        let service_account_key = oauth2::read_service_account_key(credentials_path)
            .await
            .context("Failed to read service account key")?;

        // Build the authenticator
        let auth = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .context("Failed to create authenticator")?;

        let _connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_only()
            .enable_http1()
            .build();

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .unwrap()
                        .https_only()
                        .enable_http1()
                        .build(),
                );

        let hub = CalendarHub::new(client, auth);

        Ok(Self { hub })
    }

    /// Get events for today
    pub async fn get_today_events(&self) -> Result<Vec<Event>> {
        let now = chrono::Utc::now();
        let start_of_day = now
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();
        let end_of_day = start_of_day + Duration::days(1);

        let (_, events) = self
            .hub
            .events()
            .list("primary")
            .time_min(start_of_day)
            .time_max(end_of_day)
            .single_events(true)
            .order_by("startTime")
            .doit()
            .await
            .context("Failed to fetch calendar events")?;

        Ok(events.items.unwrap_or_default())
    }

    /// Find assembly events for today
    pub async fn find_today_assembly(&self) -> Result<Option<AssemblyEvent>> {
        let events = self.get_today_events().await?;

        for event in events {
            if let Some(summary) = &event.summary {
                if summary.to_lowercase().contains("asamblea") {
                    let date_time = event
                        .start
                        .as_ref()
                        .and_then(|s| s.date_time.clone())
                        .map(|dt| dt.with_timezone(&Local));

                    return Ok(Some(AssemblyEvent {
                        summary: summary.clone(),
                        description: event.description.clone().unwrap_or_default(),
                        date_time,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Check if there's an assembly today and return details
    pub async fn check_today_assembly(&self) -> Option<AssemblyEvent> {
        match self.find_today_assembly().await {
            Ok(event) => event,
            Err(e) => {
                error!("Failed to check for assembly: {}", e);
                None
            }
        }
    }
}

/// Assembly event data
#[derive(Debug, Clone)]
pub struct AssemblyEvent {
    pub summary: String,
    pub description: String,
    pub date_time: Option<DateTime<Local>>,
}

/// Assembly reminder service
pub struct AssemblyReminder {
    calendar: CalendarService,
}

impl AssemblyReminder {
    /// Create new assembly reminder
    pub async fn new(credentials_path: &Path) -> Result<Self> {
        let calendar = CalendarService::new(credentials_path).await?;
        Ok(Self { calendar })
    }

    /// Check for assemblies and return reminder data if found
    pub async fn check_assembly_today(&self) -> Option<AssemblyReminderData> {
        let assembly = self.calendar.check_today_assembly().await?;

        Some(AssemblyReminderData {
            date: assembly
                .date_time
                .map(|dt| dt.format("%d/%m/%Y").to_string())
                .unwrap_or_else(|| Local::now().format("%d/%m/%Y").to_string()),
            time: assembly
                .date_time
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "Todo el día".to_string()),
            description: assembly.description,
        })
    }
}

/// Data for assembly reminder email
#[derive(Debug, Clone)]
pub struct AssemblyReminderData {
    pub date: String,
    pub time: String,
    pub description: String,
}

/// Create assembly reminder email body
pub fn create_assembly_reminder_email(data: &AssemblyReminderData) -> String {
    format!(
        r#"Hola,

Te recordamos que hoy tienes una asamblea:

📅 Fecha: {}
🕐 Hora: {}
📝 Descripción: {}

¡No olvides asistir!

Saludos."#,
        data.date, data.time, data.description
    )
}

/// Create calendar service if credentials are available
pub async fn create_calendar_service(credentials_path: &Path) -> Option<CalendarService> {
    if credentials_path.exists() {
        match CalendarService::new(credentials_path).await {
            Ok(service) => {
                info!("Google Calendar service initialized");
                Some(service)
            }
            Err(e) => {
                error!("Failed to initialize Google Calendar: {}", e);
                None
            }
        }
    } else {
        None
    }
}
