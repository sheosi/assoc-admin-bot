use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};

use teloxide::types::ChatId;

/// Arguments for the set_chat tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {
    #[description("El tipo de chat a configurar. Valores válidos: 'tesorería' (notificaciones de tesorería), 'publi_actas' (publicación de actas)")]
    chat_type: ChatType,
}

#[derive(serde::Deserialize, ToolParameters)]
pub enum ChatType {
    #[serde(rename = "tesorería")]
    Tesoreria,

    #[serde(rename = "publi_actas")]
    PubliActas,
}

/// Set treasury notifications chat tool
pub struct SetChat;

impl Tool for SetChat {
    fn name(&self) -> &'static str {
        "Set-Chat"
    }

    fn description(&self) -> &'static str {
        "Establece el chat actual para un tipo específico de notificaciones. Tipos soportados: 'tesorería' (notificaciones de tesorería), 'publi_actas' (publicación de actas)"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        chat_id: ChatId,
        arguments: &str,
    ) -> Result<ToolCallAction> {
        let args: CallArgs = serde_json::from_str(arguments)?;
        let msg = match args.chat_type {
            ChatType::Tesoreria => {
                ctx.set_treasury_notifications_chat_id(chat_id).await?;
                "✅ Chat de tesorería configurado correctamente"
            }
            ChatType::PubliActas => {
                ctx.set_assembly_minutes_chat_id(chat_id).await?;
                "✅ Chat de publicación de actas configurado correctamente"
            }
        };

        Ok(ToolCallAction::Message(msg.into()))
    }
}
