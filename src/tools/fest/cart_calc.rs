//! Calculate cart tool for AI function calling

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {
    #[description("El total de la compra")]
    total: f64,

    #[description("El dinero que el cliente ha dado")]
    dinero: f64,
}

/// Calculate cart tool
pub struct CartCalc;

impl Tool for CartCalc {
    fn name(&self) -> &'static str {
        "Calculate-Cart"
    }

    fn description(&self) -> &'static str {
        "Calcula el cambio a devolver al cliente"
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

        let cambio = args.dinero - args.total;

        Ok(ToolCallAction::MarkDown(format!("**Cambio:** {}€", cambio)))
    }
}
