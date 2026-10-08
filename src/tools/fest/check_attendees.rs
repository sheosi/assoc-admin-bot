//! Check fest attendees tool for AI function calling

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the check_attendees tool
/// No parameters required
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {}

/// Check fest attendees tool
pub struct CheckAttendees;

impl Tool for CheckAttendees {
    fn name(&self) -> &'static str {
        "Lista-Asistentes"
    }

    fn description(&self) -> &'static str {
        "Vamos a revisar todos los asistentes"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        _chat_id: ChatId,
        _arguments: &str,
    ) -> Result<ToolCallAction> {
        // TODO: Check whether they are in bubu
        // Get attendees who go to Sundal
        let attendees = match ctx.who_goes_to_sundal().await {
            Ok(list) => list,
            Err(_) => {
                return Ok(ToolCallAction::Message(
                    "No he podido listar los asistentes, lo siento".to_string(),
                ))
            }
        };

        // Format the attendees list
        let attendee_names: Vec<String> = attendees.into_iter().map(|a| a.codename).collect();

        // Send custom list with message

        Ok(ToolCallAction::List {
            msg: "Aquí los tienes, haz click en uno para eliminarlo".to_string(),
            items: attendee_names,
        })
    }
}
