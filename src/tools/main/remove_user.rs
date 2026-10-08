use crate::services::association::AssociationContext;
use crate::tools::Tool;
use crate::utils::fuzzy_search;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the remove_admin tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("El nombre de usuario de telegram a quitar")]
    usuario: &'a str,
}

/// Remove admin user tool
pub struct RemoveUser;

impl Tool for RemoveUser {
    fn name(&self) -> &'static str {
        "Quitar-Admin"
    }

    fn description(&self) -> &'static str {
        "Quita un usuario administrador"
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

        // Get list of admins
        let admins = match ctx.get_admins().await {
            admin_list => admin_list,
        };

        // Use fuzzy search to find the closest match
        let target = fuzzy_search(&admins, &processed_user);

        if let Some(target) = target {
            Ok(ToolCallAction::Confirm(
                format!("Voy a quitar a \"@{target}\" cómo administrador en este bot, de acuerdo?"),
                target.to_string(),
            ))
        } else {
            Ok(ToolCallAction::Message(format!(
                "No he encontrado a '{}' ni nada parecido cómo administrador",
                args.usuario
            )))
        }
    }

    async fn handle_callback(
        &self,
        ctx: &mut AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        tracing::info!("Removing {} from admin", callback_data);

        // Remove the user from admin
        match ctx.remove_admin(callback_data).await {
            Ok(_) => Ok(format!("👍 Perfecto, @{callback_data} ya no es admin!")),
            Err(e) => {
                tracing::error!("Error removing admin: {}", e);
                Ok(format!("❌ Error al quitar administrador: {}", e))
            }
        }
    }
}
