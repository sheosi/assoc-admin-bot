//! Fest shop tool for AI function calling

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the fest_shop tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Action to perform")]
    action: &'a str,
    #[description("Item name (for price/buy)")]
    item: Option<&'a str>,
}

/// Shop functionality
pub struct OnShop;

impl Tool for OnShop {
    fn name(&self) -> &'static str {
        "En-Tienda"
    }

    fn description(&self) -> &'static str {
        "Access fest shop functionality (list items, prices)"
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
        let args: CallArgs = serde_json::from_str(arguments)?;

        let msg = match args.action {
            "list" => {
                format!("*Tienda del Fest*\n\nArtículos disponibles:\n• Camiseta - 15€\n• Taza - 8€\n• Pegatina - 2€")
            }
            "price" => {
                let item_name = args.item.unwrap_or("item");
                format!("Precio de {}: Consultar en tienda", item_name)
            }
            "buy" => {
                let item_name = args.item.unwrap_or("item");
                format!(
                    "Para comprar {}, contacta con un organizador en el evento",
                    item_name
                )
            }
            _ => {
                format!("Acción no reconocida")
            }
        };

        Ok(ToolCallAction::Message(msg))
    }
}
