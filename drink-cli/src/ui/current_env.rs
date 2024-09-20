use ratatui::{
	layout::Alignment,
	widgets::{Paragraph, Widget, Wrap},
};

use crate::{app_state::AppState, ui::layout::section};

pub(super) fn build(app_state: &mut AppState) -> impl Widget {
	let current_contract_info = match app_state.contracts.current_contract() {
		Some(contract) => format!("name: {} | address: {}", contract.name, contract.address),
		None => "No deployed contract".to_string(),
	};

	Paragraph::new(format!(
		r#"Current working directory: {}
Block height: {}
Deployed contracts: {}
Current actor: {}
Current contract: {{ {} }}"#,
		app_state.ui_state.cwd.to_str().unwrap(),
		app_state.chain_info.block_height,
		app_state.contracts.count(),
		app_state.chain_info.actor,
		current_contract_info
	))
	.alignment(Alignment::Left)
	.wrap(Wrap { trim: false })
	.block(section("Current environment"))
}
