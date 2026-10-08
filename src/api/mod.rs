//! API module with axum handlers
//! Provides HTTP endpoints for webhooks and health checks

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::FormsAppWebhook;
use crate::services::association::AssociationContext;

/// API state shared across handlers
#[derive(Clone)]
pub struct ApiState {
    pub context: Arc<RwLock<AssociationContext>>,
}

impl ApiState {
    pub fn new(context: Arc<RwLock<AssociationContext>>) -> Self {
        Self { context }
    }
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

/// Create the API router
pub fn create_router(context: Arc<RwLock<AssociationContext>>) -> Router {
    let state = ApiState::new(context);

    Router::new()
        .route("/health", get(health_handler))
        .route("/webhook/new-attendee", post(new_attendee_handler))
        .with_state(state)
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
        }),
    )
}

/// Webhook handler for new fest attendees from forms.app
async fn new_attendee_handler(
    State(state): State<ApiState>,
    Json(payload): Json<FormsAppWebhook>,
) -> impl IntoResponse {
    // Parse the webhook payload
    let answers = &payload.answer.answers;

    // Indices for the form fields (matching Go implementation)
    const LEGAL_NAME_INDEX: usize = 0;
    const CODE_NAME_INDEX: usize = 1;
    const ASSISTS_TO_INDEX: usize = 4;
    const DIET_INDEX: usize = 5;
    const ALLERGIES_INDEX: usize = 6;

    // Extract legal name (full name)
    let legal_name = if let Some(name_data) = answers.get(LEGAL_NAME_INDEX) {
        if let Some(fullname) = &name_data.fullname {
            format!("{} {}", fullname.first_name, fullname.last_name)
        } else {
            return (StatusCode::BAD_REQUEST, "Missing legal name");
        }
    } else {
        return (StatusCode::BAD_REQUEST, "Missing legal name");
    };

    // Extract code name
    let code_name = if let Some(code_data) = answers.get(CODE_NAME_INDEX) {
        code_data.text.clone().unwrap_or_default()
    } else {
        return (StatusCode::BAD_REQUEST, "Missing code name");
    };

    // Extract assists to (which events they attend)
    let mut assists_to: Vec<String> = Vec::new();
    if let Some(assists_data) = answers.get(ASSISTS_TO_INDEX) {
        if let Some(selections) = &assists_data.selection {
            for selection in selections {
                let text = &selection.text;
                if text.contains("Bubu") {
                    assists_to.push("bubu".to_string());
                } else if text.contains("Sundal") {
                    assists_to.push("sundal".to_string());
                } else if text.contains("Esmorçar") || text.contains("Almuerzo") {
                    assists_to.push("almuerzo".to_string());
                }
            }
        }
    }

    // Extract diet preference
    let diet: i64 = if let Some(diet_data) = answers.get(DIET_INDEX) {
        if let Some(selections) = &diet_data.selection {
            if let Some(first) = selections.first() {
                let diet_text = &first.text;
                if diet_text.contains("Vegetarian") {
                    1
                } else if diet_text.contains("Vegan") {
                    2
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    // Extract allergies
    let allergies = if let Some(allergy_data) = answers.get(ALLERGIES_INDEX) {
        allergy_data.text.clone().filter(|s| !s.is_empty())
    } else {
        None
    };

    // Create the fest attendee JSON
    let assists_to_json = match serde_json::to_string(&assists_to) {
        Ok(json) => json,
        Err(_) => "[]".to_string(),
    };

    // Save to database using spawn_blocking
    let legal_name_clone = legal_name.clone();
    let code_name_clone = code_name.clone();
    let assists_to_json_clone = assists_to_json.clone();
    let result = state
        .context
        .read()
        .await
        .on_db(move |c| {
            crate::queries::NewFestAttendee::builder()
                .legalname(&legal_name_clone)
                .codename(&code_name_clone)
                .assiststo(&assists_to_json_clone)
                .allergies(allergies.as_deref())
                .diet(diet)
                .build()
                .execute(c)?;

            Ok::<_, anyhow::Error>(())
        })
        .await;

    match result {
        Ok(()) => (StatusCode::CREATED, "Created"),
        Err(e) => {
            tracing::error!("Failed to create fest attendee: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create attendee",
            )
        }
    }
}
