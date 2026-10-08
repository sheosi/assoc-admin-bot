use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the list_admins tool (empty - no parameters needed)
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {}

/// List admin users tool
pub struct ListUsers;

impl Tool for ListUsers {
    fn name(&self) -> &'static str {
        "Listar-Admins"
    }

    fn description(&self) -> &'static str {
        "Lista usuarios administradores de este bot, aunque no la directiva"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        _chat_id: ChatId,
        _arguments: &str,
    ) -> Result<ToolCallAction> {
        let mut msg = "Administradores:\n\n".to_string();
        for a in ctx.get_admins().await {
            msg.push_str(&format!(" • {} \n", a));
        }

        Ok(msg.into())
    }
}
