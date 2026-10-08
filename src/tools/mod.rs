//! Tools module for AI function calling
//! Implements various tools that the AI can invoke

use std::sync::Arc;
use teloxide::types::ChatId;

use botframework::{
    telegram::{Properties, ToolCallAction},
    utils::create_ai_fun,
};

// Import tool modules
pub mod association;
pub mod fest;
pub mod main;

pub use association::*;
use async_openai::types::chat::ChatCompletionTools;
pub use fest::*;
pub use main::*;

use crate::services::association::AssociationContext;

type Tools = Vec<ToolsTrait>;

/// Tool trait for AI function calling
#[enum_dispatch::enum_dispatch]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Properties;
    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        chat_id: ChatId,
        arguments: &str,
    ) -> anyhow::Result<ToolCallAction>;
    async fn handle_callback(
        &self,
        _ctx: &mut AssociationContext,
        _callback_data: &str,
    ) -> anyhow::Result<String> {
        Ok("Done".to_string())
    }
}

#[enum_dispatch::enum_dispatch(Tool)]
pub enum ToolsTrait {
    // Main
    AddUser,
    ListUsers,
    RemoveUser,

    // Fest
    AtteendeePaidFest,
    CheckAttendees,
    CuantosVienen,
    OnShop,
    CartCalc,
    CalcBocatas,

    // Assoc
    SetChat,
    GetChat,
    RegisterTreasuryUpdate,
    GetTreasuryReport,
    CreateAssembly,
    PostAssemblyMinutes,
    AddAssociate,
    RemoveAssociate,
    ListAssociates,
    UpdateAssociate,
}

/// Returns all available tools
pub fn all_tools() -> Vec<ToolsTrait> {
    let mut tools: Vec<ToolsTrait> = Vec::new();

    main_tools(&mut tools);
    fest_tools(&mut tools);
    assoc_tools(&mut tools);

    tools
}

fn main_tools(tools: &mut Tools) {
    tools.push(ToolsTrait::AddUser(main::AddUser));
    tools.push(ToolsTrait::ListUsers(main::ListUsers));
    tools.push(ToolsTrait::RemoveUser(main::RemoveUser));
}

fn fest_tools(tools: &mut Tools) {
    tools.push(ToolsTrait::AtteendeePaidFest(fest::AtteendeePaidFest));
    tools.push(ToolsTrait::CheckAttendees(fest::CheckAttendees));
    tools.push(ToolsTrait::CuantosVienen(fest::CuantosVienen));
    tools.push(ToolsTrait::OnShop(fest::OnShop));
    tools.push(ToolsTrait::CartCalc(fest::CartCalc));
    tools.push(ToolsTrait::CalcBocatas(fest::CalcBocatas));
}

fn assoc_tools(tools: &mut Tools) {
    // Association tools
    tools.push(ToolsTrait::SetChat(association::SetChat));
    tools.push(ToolsTrait::GetChat(association::GetChat));
    tools.push(ToolsTrait::RegisterTreasuryUpdate(
        association::RegisterTreasuryUpdate,
    ));
    tools.push(ToolsTrait::GetTreasuryReport(
        association::GetTreasuryReport,
    ));
    tools.push(ToolsTrait::CreateAssembly(association::CreateAssembly));
    tools.push(ToolsTrait::PostAssemblyMinutes(
        association::PostAssemblyMinutes,
    ));
    tools.push(ToolsTrait::AddAssociate(association::AddAssociate));
    tools.push(ToolsTrait::RemoveAssociate(association::RemoveAssociate));
    tools.push(ToolsTrait::ListAssociates(association::ListAssociates));
    tools.push(ToolsTrait::UpdateAssociate(association::UpdateAssociate));
}

pub struct ToolsResult {
    pub dict: std::collections::HashMap<String, Arc<ToolsTrait>>,
    pub ai_descr: Vec<ChatCompletionTools>,
}

/// Create a dictionary of tools by name
pub fn new_tools_dict() -> ToolsResult {
    let tools = all_tools();
    let mut dict = std::collections::HashMap::new();
    let mut ai_descr = Vec::new();

    for tool in tools {
        let params_org = tool.parameters();
        let name = tool.name().to_string();
        let description = tool.description().to_string();
        dict.insert(name.clone(), Arc::new(tool));

        ai_descr.push(create_ai_fun(name, description, params_org));
    }

    ToolsResult { dict, ai_descr }
}
