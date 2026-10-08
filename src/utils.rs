use chrono::{DateTime, Duration, Utc};
use teloxide::types::ChatId;

use crate::services::association::AssociationContext;

pub fn fuzzy_search<'a>(hay: &'a [String], needle: &str) -> Option<&'a str> {
    hay.iter()
        .map(|s| (s, edit_distance::edit_distance(s, needle)))
        .min_by(|a, b| a.1.cmp(&b.1))
        .map(|a| a.0.as_str())
}

pub trait ErrMsg {
    async fn err_msg(self, chat_id: ChatId, msg: &str, ctx: &mut AssociationContext) -> Self;
}

impl<T, E> ErrMsg for Result<T, E> {
    async fn err_msg(self, chat_id: ChatId, msg: &str, ctx: &mut AssociationContext) -> Self {
        if let Err(_) = self {
            if let Err(e) = ctx.send_raw(chat_id, msg).await {
                tracing::error!("Failed to send error msg: {e}")
            }
        }

        self
    }
}

pub fn get_assoc_env(prefix: &str, name: &str) -> String {
    std::env::var(&format!("{}_{}", prefix, name)).unwrap_or_else(|_| String::new())
}

pub fn create_ics_file(date: DateTime<Utc>, description: &str) -> (String, String) {
    // 1. Create a unique filename
    // Go's "02-01-2006" -> Rust's "%d-%m-%Y"
    let filename = format!("Asamblea ({}).ics", date.format("%d-%m-%Y"));

    // 2. Format dates for ICS (UTC format)
    let format_str = "%Y%m%dT%H%M%SZ";
    let start_date = date.format(format_str).to_string();
    let end_date = (date + Duration::hours(2)).format(format_str).to_string();
    let now = Utc::now().format(format_str).to_string();
    let uid = format!("assembly-{}@assoc.es", date.timestamp());

    // 3. Build ICS content
    // Rust uses \n for newlines; the indentation here is literal inside the raw string
    let ics_content = format!(
        "BEGIN:VCALENDAR\n\
        VERSION:2.0\n\
        PRODID:-//Assoc//Asamblea//ES\n\
        CALSCALE:GREGORIAN\n\
        METHOD:PUBLISH\n\
        BEGIN:VEVENT\n\
        UID:{uid}\n\
        DTSTAMP:{now}\n\
        DTSTART:{start_date}\n\
        DTEND:{end_date}\n\
        SUMMARY:Asamblea\n\
        DESCRIPTION:{description}\n\
        END:VEVENT\n\
        END:VCALENDAR"
    );

    // Return the path as a string
    (filename, ics_content)
}
