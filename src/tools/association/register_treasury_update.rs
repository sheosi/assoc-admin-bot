use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the register_treasury_update tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Descripción de la actualización de tesorería")]
    description: &'a str,
    #[description(
        "Cantidad en euros de la actualización (positiva para ingresos, negativa para gastos)"
    )]
    cantidad: f64,
}

/// Register treasury update tool
pub struct RegisterTreasuryUpdate;

impl Tool for RegisterTreasuryUpdate {
    fn name(&self) -> &'static str {
        "Register-Treasury-Update"
    }

    fn description(&self) -> &'static str {
        "Registra una actualización de tesorería con una descripción y cantidad, y la envía al chat de tesorería configurado si existe"
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

        ctx.add_treasury_update(args.description, args.cantidad)
            .await?;

        let treasury_chat = ctx.get_treasury_notifications_chat_id().await?;
        let sign = if args.cantidad > 0.0 { "+" } else { "" };

        let mut announced = false;

        if let Some(treasury_chat) = treasury_chat {
            let msg = format!(
                "📊 *Actualización de Tesorería*\n\n*Descripción:* {}\n*Cantidad:* {sign}{:.2}€",
                args.description, args.cantidad
            );

            match ctx.send_md(ChatId(treasury_chat), &msg).await {
                Ok(_) => {
                    announced = true;
                }
                Err(e) => {
                    tracing::info!("[ERROR] Failed to send treasury update: {}", e.to_string());
                }
            }
        }

        // Update announced status if notification was sent
        if announced {
            Ok("✅ Actualización de tesorería registrada y anunciada".into())
        } else {
            Ok("✅ Actualización de tesorería registrada (pendiente de anuncio)".into())
        }
    }
}
