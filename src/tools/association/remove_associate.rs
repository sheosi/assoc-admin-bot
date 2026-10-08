use crate::services::association::AssociationContext;
use crate::tools::Tool;
use crate::utils::fuzzy_search;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Remove associate tool
pub struct RemoveAssociate;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Nombre en clave del asociado a eliminar (sin @)")]
    nickname: &'a str,
}

impl Tool for RemoveAssociate {
    fn name(&self) -> &'static str {
        "remove_associate"
    }

    fn description(&self) -> &'static str {
        "Elimina un asociado de la base de datos"
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

        // Get list of all associates for fuzzy search
        let associates = match ctx.list_associates().await {
            Ok(list) => list,
            Err(e) => {
                return Ok(ToolCallAction::Message(format!(
                    "❌ Error al obtener la lista de asociados: {}",
                    e
                )))
            }
        };

        // Extract nicknames for fuzzy search
        let nick_names: Vec<String> = associates.into_iter().map(|a| a.nickname).collect();

        // Use fuzzy search to find the closest match
        match fuzzy_search(&nick_names, args.nickname) {
            Some(target) => {
                // Ask for confirmation with the matched name
                Ok(ToolCallAction::Confirm(
                    format!(
                        "Voy a eliminar a \"{}\" de los asociados, ¿de acuerdo?",
                        target
                    ),
                    target.to_string(),
                ))
            }
            None => Ok(ToolCallAction::Message(format!(
                "No he encontrado a '{}' ni nada parecido como asociado",
                args.nickname
            ))),
        }
    }

    async fn handle_callback(
        &self,
        ctx: &mut AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        tracing::info!("Removing associate {}", callback_data);

        // Remove the associate from the database (target is already lowercase)
        match ctx.remove_associate(callback_data).await {
            Ok(_) => Ok(format!("✅ Asociado eliminado: {}", callback_data)),
            Err(e) => {
                tracing::error!("Error al eliminar el asociado: {}", e);
                Ok(format!("❌ Error al eliminar el asociado: {}", e))
            }
        }
    }
}
