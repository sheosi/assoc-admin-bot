use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("El nombre de usuario de telegram a añadir")]
    usuario: &'a str,
}

/// Add admin user tool
pub struct AddUser;

impl Tool for AddUser {
    fn name(&self) -> &'static str {
        "Añadir-Admin"
    }

    fn description(&self) -> &'static str {
        "Añade un usuario administrador"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        _ctx: &mut AssociationContext,
        _chat_id: ChatId,
        arguments: &str,
    ) -> Result<ToolCallAction> {
        let args: CallArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow::anyhow!("Error al procesar los argumentos: {}", e))?;

        // Process the username (remove @ prefix, lowercase, trim)
        let processed_user = args.usuario.trim().trim_start_matches('@').to_lowercase();

        // Return a confirmation action
        Ok(ToolCallAction::Confirm(
            format!(
                "Voy a añadir a \"@{processed_user}\" cómo administrador en este bot, de acuerdo?"
            ),
            processed_user,
        ))
    }

    async fn handle_callback(
        &self,
        ctx: &mut AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        tracing::info!("Adding {} as admin", callback_data);

        // Add the user as admin
        match ctx.add_admin(callback_data).await {
            Ok(_) => Ok(format!("👍 Perfecto, @{callback_data} es ahora admin!")),
            Err(e) => {
                tracing::error!("Error adding admin: {}", e);
                Ok(format!("❌ Error al añadir administrador: {}", e))
            }
        }
    }
}
