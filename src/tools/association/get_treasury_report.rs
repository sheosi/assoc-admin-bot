use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use chrono::{DateTime, TimeZone, Utc};

use teloxide::types::ChatId;

/// Arguments for the get_treasury_report tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Fecha de inicio del informe en formato YYYY-MM-DD")]
    fecha_inicio: &'a str,

    #[description(
        "Fecha de fin del informe en formato YYYY-MM-DD (opcional, por defecto es la fecha actual)"
    )]
    fecha_fin: &'a str,
}

/// Get treasury report tool
pub struct GetTreasuryReport;

impl Tool for GetTreasuryReport {
    fn name(&self) -> &'static str {
        "Get-Treasury-Report"
    }

    fn description(&self) -> &'static str {
        "Genera un informe de tesorería con todas las actualizaciones en un período de tiempo especificado, mostrando ingresos, gastos y balance"
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
        fn parse_date(input: &str) -> Result<DateTime<Utc>> {
            let date = chrono::NaiveDate::parse_from_str(input, FORMATO_FECHA)?;
            let datetime_naive = date.and_hms_opt(0, 0, 0).expect("This should work");
            Ok(Utc.from_utc_datetime(&datetime_naive))
        }

        fn write_date(output: &mut String, date: &DateTime<Utc>) {
            date.format("%d/%m/%Y").write_to(output).expect("");
        }

        const FORMATO_FECHA: &str = "%Y-%m-%d";

        let arguments: CallArgs = serde_json::from_str(arguments)?;
        let start_date = parse_date(arguments.fecha_inicio)?;
        let end_date = if arguments.fecha_fin == "" {
            chrono::Utc::now()
        } else {
            parse_date(arguments.fecha_fin)?
        };

        let updates = ctx
            .get_treasury_updates_in_period(start_date, end_date)
            .await?;

        let mut report = "📊 *Informe de Tesorería*\n\n".to_string();

        report.push_str("📅 *Período:* ");
        write_date(&mut report, &start_date);
        report.push_str(" - ");
        write_date(&mut report, &end_date);
        report.push_str("\n\n");

        if updates.is_empty() {
            report.push_str("No hay actualizaciones de tesorería en este período.");
        } else {
            let transactions = updates.len();
            report.push_str(&format!("*Total de transacciones:* {transactions}\n\n"));
            report.push_str("*Detalle de movimientos:*\n");
            let mut total_income = 0.0;
            let mut total_expenses = 0.0;

            for update in updates {
                let sign = if update.amount >= 0.0 { "+" } else { "" };
                report.push_str(&format!(
                    "• {}: {sign}{:.2}€\n",
                    update.description, update.amount
                ));

                if update.amount >= 0.0 {
                    total_income += update.amount;
                } else {
                    total_expenses += update.amount;
                }
            }

            let net_balance = total_income + total_expenses;

            report.push_str("\n*Resumen:*\n");
            report.push_str(&format!("💰 Ingresos: +{total_income:.2}€\n"));
            report.push_str(&format!("💸 Gastos: {total_expenses:.2}€\n"));
            report.push_str(&format!("📈 Balance Neto: {net_balance:.2}€"));
        }

        Ok(ToolCallAction::MarkDown(report))
    }
}
