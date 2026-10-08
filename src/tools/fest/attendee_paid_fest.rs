//! Fest tools for AI function calling
//! Implements tools for managing fest attendees and shop

use crate::queries::GetAttendessNamesRow;
use crate::services::association::AssociationContext;
use crate::tools::Tool;
use crate::utils::fuzzy_search;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for registering fest payment
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("El nombre de usuario que ha pagado el fest")]
    usuario: &'a str,

    #[description("Si ha pagado para el fest")]
    asistente_pagado: bool,
}

/// Register fest payment tool
pub struct AtteendeePaidFest;

impl Tool for AtteendeePaidFest {
    fn name(&self) -> &'static str {
        "Asistente-Pagado-Fest"
    }

    fn description(&self) -> &'static str {
        "Registra el pago de un asistente para el fest"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        _chat_id: ChatId,
        arguments: &str,
    ) -> Result<ToolCallAction> {
        let args: CallArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow::anyhow!("Error al procesar los argumentos: {}", e))?;

        // Process the username (remove @ prefix, lowercase, trim)
        let processed_user = args.usuario.trim().trim_start_matches('@').to_lowercase();

        // Get attendee names
        let names = match ctx.get_fest_attendees_names().await {
            Ok(list) => list,
            Err(_) => {
                return Ok(ToolCallAction::Message(
                    "Lo siento ha habido un problema".to_string(),
                ))
            }
        };

        // Search in names (both legal names and code names)
        match search_in_names(names.as_slice(), &processed_user) {
            Some(attendee) => {
                // Send confirmation
                Ok(ToolCallAction::Confirm(
                    format!(
                        "Entonces {} / @{} ha pagado no?",
                        attendee.legalname, attendee.codename
                    ),
                    attendee.legalname,
                ))
            }
            None => Ok(ToolCallAction::Message(format!(
                "Disculpa, {} no está registrado cómo asistente",
                processed_user
            ))),
        }
    }

    async fn handle_callback(
        &self,
        ctx: &mut AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        // Set the user as paid (haspaid = 1)
        match ctx.legal_name_set_paid(callback_data, 1).await {
            Ok(_) => Ok(format!("Añadido @{} cómo pagado", callback_data)),
            Err(e) => {
                tracing::error!("Error setting payment: {}", e);
                Ok(format!("❌ Error al registrar el pago: {}", e))
            }
        }
    }
}

/// Search in names (both legal names and code names)
fn search_in_names(names: &[GetAttendessNamesRow], term: &str) -> Option<GetAttendessNamesRow> {
    // Extract legal names and code names
    let legal_names: Vec<String> = names.iter().map(|n| n.legalname.clone()).collect();
    let code_names: Vec<String> = names.iter().map(|n| n.codename.clone()).collect();

    // Search in legal names first
    if let Some(matched) = fuzzy_search(&legal_names, term) {
        // Find the index of the matched name
        if let Some(index) = legal_names.iter().position(|n| n == matched) {
            return Some(GetAttendessNamesRow {
                codename: names[index].codename.clone(),
                legalname: names[index].legalname.clone(),
            });
        }
    }

    // Search in code names
    if let Some(matched) = fuzzy_search(&code_names, term) {
        // Find the index of the matched name
        if let Some(index) = code_names.iter().position(|n| n == matched) {
            return Some(GetAttendessNamesRow {
                codename: names[index].codename.clone(),
                legalname: names[index].legalname.clone(),
            });
        }
    }

    None
}
