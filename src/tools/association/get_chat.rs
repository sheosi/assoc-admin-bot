use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};

use teloxide::types::ChatId;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {
    #[description("Chat ID for notifications")]
    chat_type: ChatType,
}

#[derive(serde::Deserialize, ToolParameters)]
pub enum ChatType {
    #[serde(rename = "tesorería")]
    Tesoreria,

    #[serde(rename = "publi_actas")]
    PubliActas,
}

/// Get treasury notifications chat tool
pub struct GetChat;

impl Tool for GetChat {
    fn name(&self) -> &'static str {
        "Get-Chat"
    }

    fn description(&self) -> &'static str {
        "Obtiene el ID del chat de Telegram configurado para un tipo específico de notificaciones. Tipos soportados: 'tesorería' (notificaciones de tesorería), 'publi_actas' (publicación de actas)"
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
        let args: CallArgs = serde_json::from_str(arguments)?;

        match args.chat_type {
            ChatType::Tesoreria => {
                let Some(chat_id) = ctx.get_treasury_notifications_chat_id().await? else {
                    return Ok("No hay ningún chat configurado para tesorería.".into());
                };

                Ok(format!("Chat de tesorería: {chat_id}").into())
            }
            ChatType::PubliActas => {
                let Some(chat_id) = ctx.get_assembly_minutes_chat_id().await? else {
                    return Ok("No hay ningún chat configurado para publicación de actas".into());
                };

                Ok(format!("Chat de publicación de actas: {chat_id}").into())
            }
        }
    }
}
