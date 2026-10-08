use crate::utils::ErrMsg;
use crate::{services::association::AssociationContext, tools::Tool};
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};

use teloxide::types::ChatId;

/// Add associate tool
pub struct AddAssociate;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Nombre en clave del asociado (ejemplo: Motero) no relacionado con Telegram")]
    nickname: &'a str,
    #[description("Correo electrónico del asociado")]
    email: &'a str,
}

impl Tool for AddAssociate {
    fn name(&self) -> &'static str {
        "Add-associate"
    }

    fn description(&self) -> &'static str {
        "Registra un nuevo asociado con su nombre en clave y su correo electrónico"
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
        let args: CallArgs = serde_json::from_str(arguments)
            .err_msg(chat_id, "❌ Error: datos inválidos", ctx)
            .await?;

        let nickname_lower = args.nickname.to_lowercase();
        if ctx.get_associate(&nickname_lower).await?.is_some() {
            return Ok(format!(" El asociado {} ya existe", args.nickname).into());
        }

        Ok(ToolCallAction::Confirm(
            format!(
                "Voy a registrar a \"{}\" con email \"{}\" como asociado, ¿de acuerdo?",
                args.nickname, args.email
            ),
            format!("{}|{}", args.nickname, args.email),
        ))
    }

    async fn handle_callback(
        &self,
        ctx: &mut crate::services::association::AssociationContext,
        callback_data: &str,
    ) -> anyhow::Result<String> {
        let parts: Vec<&str> = callback_data.split('|').collect();
        if parts.len() != 2 {
            return Ok("❌ Error: datos inválidos".into());
        }

        let nickname = parts[0];
        let email = parts[1];

        tracing::info!("[{}] Adding associate {} ({})", "", nickname, email);
        let msg = match ctx.add_associate(nickname, email).await? {
            Some(_) => {
                format!("✅ Asociado registrado: {nickname} ( {email} )",)
            }
            None => "❌ Error al registrar el asociado".into(),
        };

        Ok(msg)
    }
}
