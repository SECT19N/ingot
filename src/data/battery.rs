use battery::{Manager, State};

#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub index: usize,
    pub level: f32,
    pub voltage: f32,
    pub current: f32,
    pub wattage: f32,
    pub temperature: f32,
    pub capacity: f32,
    pub state: String,
    pub is_charging: bool,
    pub health: String,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryData {
    pub batteries: Vec<BatteryInfo>,
    pub aggregate: BatteryAggregate,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryAggregate {
    pub level: f32,
    pub total_wattage: f32,
    pub total_capacity: f32,
    pub any_charging: bool,
    pub all_full: bool,
    pub count: usize,
}

pub fn read_batteries() -> BatteryData {
    let Ok(manager) = Manager::new() else {
        return BatteryData::default();
    };
    let Ok(batteries_iter) = manager.batteries() else {
        return BatteryData::default();
    };

    let batteries: Vec<BatteryInfo> = batteries_iter
        .enumerate()
        .filter_map(|(index, b)| {
            let b = b.ok()?;
            let level = b.state_of_charge().value * 100.0;
            let voltage = b.voltage().value;
            let current = b.energy_rate().value * 1000.0;
            let wattage = (voltage * current.abs()) / 1000.0;
            let temperature = b.temperature().map(|t| t.value - 273.15).unwrap_or(0.0);
            let capacity = b.energy_full_design().value / 3.6;
            let is_charging = matches!(b.state(), State::Charging | State::Full);
            let state = format!("{:?}", b.state());
            let health = format!("{:.0}%", b.state_of_health().value * 100.0);

            Some(BatteryInfo {
                index,
                level,
                voltage,
                current: current.abs(),
                wattage,
                temperature,
                capacity,
                state,
                is_charging,
                health,
            })
        })
        .collect();

    let count = batteries.len();

    let aggregate = if count == 0 {
        BatteryAggregate::default()
    } else {
        let total_capacity: f32 = batteries.iter().map(|b| b.capacity).sum();
        let level = if total_capacity > 0.0 {
            batteries
                .iter()
                .map(|b| b.level * (b.capacity / total_capacity))
                .sum()
        } else {
            batteries.iter().map(|b| b.level).sum::<f32>() / count as f32
        };

        BatteryAggregate {
            level,
            total_wattage: batteries.iter().map(|b| b.wattage).sum(),
            total_capacity,
            any_charging: batteries.iter().any(|b| b.is_charging),
            all_full: batteries.iter().all(|b| b.state == "Full"),
            count,
        }
    };

    BatteryData {
        batteries,
        aggregate,
    }
}
