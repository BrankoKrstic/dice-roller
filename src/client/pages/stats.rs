use leptos::prelude::*;

pub struct ChanceCalculatorResult {
    trials: u32,
    successes: u32,
    accumulated_result: i128,
    success_percent: f64,
    avg_dmg: f64,
    avg_dmg_per_attack: f64,
}

struct ChanceCalculator {
    trials: u32,
    successes: u32,
    accumulated_result: i128,
}

impl ChanceCalculator {
    fn new(trials: u32) -> Self {
        Self {
            trials,
            successes: 0,
            accumulated_result: 0,
        }
    }

    fn record_hit(&mut self, result: i64) {
        self.successes += 1;
        self.accumulated_result += result as i128;
    }

    fn finish(self) -> ChanceCalculatorResult {
        ChanceCalculatorResult {
            trials: self.trials,
            successes: self.successes,
            accumulated_result: self.accumulated_result,
            success_percent: self.successes as f64 / self.trials as f64,
            avg_dmg: self.accumulated_result as f64 / self.successes as f64,
            avg_dmg_per_attack: self.accumulated_result as f64 / self.trials as f64,
        }
    }
}

enum CalculatorVariant {
   Ac,
    Dc
}

#[component]
pub fn StatsPage() -> impl IntoView {

    let (variant, set_variant) = signal(CalculatorVariant::Ac)
    view! {
        <div class="page">

        
        </div>
    }
}
