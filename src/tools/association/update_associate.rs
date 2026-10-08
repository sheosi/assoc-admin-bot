use crate::queries::ListAssociatesRow;
use crate::services::association::AssociationContext;
use crate::tools::Tool;
use crate::utils::fuzzy_search;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the update_associate tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Apodo actual del asociado a modificar")]
    nickname: &'a str,
    #[description("Nuevo apodo (dejar vacío si no se quiere cambiar)")]
    new_nickname: &'a str,
    #[description("Nuevo correo electrónico (dejar vacío si no se quiere cambiar)")]
    new_email: &'a str,
}

/// Update associate tool
pub struct UpdateAssociate;

impl Tool for UpdateAssociate {
    fn name(&self) -> &'static str {
        "Update-Associate"
    }

    fn description(&self) -> &'static str {
        "Actualiza los datos de un asociado (apodo o email)"
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

        if args.new_nickname.is_empty() && args.new_email.is_empty() {
            return Ok("❌ Debes especificar al menos un campo para actualizar (nuevo nombre o nuevo email)".into());
        }

        let associates = ctx
            .list_associates()
            .await?
            .into_iter()
            .map(|a: ListAssociatesRow| a.nickname)
            .collect::<Vec<_>>();

        let Some(target) = fuzzy_search(&associates, args.nickname) else {
            return Ok(format!(
                "No he encontrado a '{}' ni nada parecido como asociado",
                args.nickname
            )
            .into());
        };

        let associate = ctx.get_associate(&target).await?.expect("");
        let mut msg = format!("Voy a actualizar los datos de \"{target}\":\n");
        let new_nickname = if args.new_nickname != "" && args.new_nickname != associate.nickname {
            msg.push_str(&format!(
                "• Nombre:  {} → {}\n",
                associate.nickname, args.new_nickname
            ));
            args.new_nickname
        } else {
            &associate.nickname
        };

        let new_email = if args.new_email != "" && args.new_email != associate.email {
            msg.push_str(&format!(
                "• Email: {} → {}\n",
                associate.email, args.new_email
            ));
            args.new_email
        } else {
            &associate.email
        };

        msg.push_str("¿De acuerdo?");

        let target_data = format!("{target}|{new_nickname}|{new_email}",);

        Ok(ToolCallAction::Confirm(msg, target_data))
    }

    async fn handle_callback(
        &self,
        ctx: &mut crate::services::association::AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        // Parse the target data which contains: currentnickname|newnickname|newEmail
        let parts: Vec<&str> = callback_data.split('|').collect();
        if parts.len() != 3 {
            return Ok("❌ Error: datos inválidos".to_string());
        }

        let current_nickname = parts[0];
        let new_nickname = parts[1];
        let new_email = parts[2];

        tracing::info!(
            "Updating associate {} -> {} ({})",
            current_nickname,
            new_nickname,
            new_email
        );

        // Update the associate in the database
        match ctx
            .update_associate(new_nickname, new_email, current_nickname)
            .await
        {
            Ok(_) => Ok(format!(
                "✅ Asociado actualizado: {} ({})",
                new_nickname, new_email
            )),
            Err(e) => {
                tracing::error!("Error al actualizar el asociado: {}", e);
                Ok("❌ Error al actualizar el asociado".to_string())
            }
        }
    }
}
