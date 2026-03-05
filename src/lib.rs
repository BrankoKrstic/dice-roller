pub mod app;
pub mod client;
pub mod dsl;
pub mod server;
pub mod shared;

use serde::{Deserialize, Serialize};

use crate::dsl::{
    interpreter::{CryptoDiceRng, Interpreter},
    parser::Parser,
    RollError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChanceResult {
    pub trials: u32,
    pub success_count: u64,
    pub dmg: i128,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
enum WorkerSimulationResponse {
    Result(ChanceResult),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkerSimulationRequest {
    to_hit_expression: String,
    damage_expression: String,
    target: i64,
    trials: u32,
    ac_mode: bool,
}

fn worker_response_json(result: Result<ChanceResult, RollError>) -> String {
    let payload = match result {
        Ok(result) => WorkerSimulationResponse::Result(result),
        Err(error) => WorkerSimulationResponse::Error(error.to_string()),
    };

    serde_json::to_string(&payload).unwrap_or_else(|error| {
        format!(r#"{{"ok":false,"result":null,"error":"serialization failed: {error}"}}"#)
    })
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::client::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

pub fn run_trials(
    to_hit_expression: String,
    damage_expression: String,
    target: i64,
    trials: u32,
    is_ac_variant: bool,
) -> Result<ChanceResult, RollError> {
    let mut result = ChanceResult {
        trials: 0,
        success_count: 0,
        dmg: 0,
    };

    let mut to_hit_ast = Parser::new(&to_hit_expression).parse()?;
    let mut dmg_ast = Parser::new(&damage_expression).parse()?;
    let mut interpreter = Interpreter::new(CryptoDiceRng::new());
    while result.trials < trials {
        let roll = interpreter.eval_ast(&to_hit_ast)?.total();
        let mut is_hit = match is_ac_variant {
            true => roll >= target,
            false => roll < target,
        };

        if is_hit {
            result.success_count += 1;
            result.dmg += interpreter.eval_ast(&dmg_ast)?.total() as i128;
        }

        result.trials += 1;
    }

    Ok(result)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn worker_run_simulation(
    to_hit_expression: String,
    damage_expression: String,
    target_ac: i64,
    trials: u32,
    is_ac_variant: bool,
) -> String {
    worker_response_json(run_trials(
        to_hit_expression,
        damage_expression,
        target_ac,
        trials,
        is_ac_variant,
    ))
}
