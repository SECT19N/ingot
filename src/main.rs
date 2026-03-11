mod data;

use crate::data::*;
use dioxus::prelude::*;
use std::time::Duration;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const HISTORY_LEN: usize = 60;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Battery {},
    #[route("/cpu")]
    Cpu {},
    #[route("/ram")]
    Ram {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON },
        document::Link { rel: "stylesheet", href: TAILWIND_CSS },
        div {
            class: "min-h-screen bg-neutral-950 text-neutral-50 font-mono",
            Router::<Route> {}
        }
    }
}

#[component]
fn NavBar() -> Element {
    rsx! {
        nav {
            class: "flex gap-1 border-b border-neutral-800 px-4 bg-neutral-950 font-mono",
            div {
                class: "text-yellow-300 font-bold tracking-widest text-sm py-4 pr-6",
                "Ingot"
            }
            NavTab { to: Route::Battery {}, label: "Battery" },
            NavTab { to: Route::Cpu {}, label: "CPU" },
            NavTab { to: Route::Ram {}, label: "RAM" },
        }
        main {
            class: "p-4 max-w-sm mx-auto",
            Outlet::<Route> {}
        }
    }
}

#[component]
fn NavTab(to: Route, label: &'static str) -> Element {
    rsx! {
        Link {
            to,
            class: "text-xs uppercase tracking-widest px-4 py-4 text-neutral-500 border-b-2 border-transparent hover:text-yellow-300 transition-colors",
            active_class: "text-yellow-300 border-yellow-300",
            "{label}"
        }
    }
}

#[component]
fn Battery() -> Element {
    let mut info = use_signal(|| read_batteries());
    let mut history = use_signal(|| vec![0.0f32; HISTORY_LEN]);

    use_future(move || async move {
        loop {
            async_std::task::sleep(Duration::from_secs(1)).await;

            let data = read_batteries();
            history.write().push(data.aggregate.level);
            if history.read().len() > HISTORY_LEN {
                history.write().remove(0);
            }
            info.set(data);
        }
    });

    let data = info.read();
    let agg = &data.aggregate;

    rsx! {
        div { class: "flex flex-col gap-3 font-mono text-neutral-100",

            Card {
                div { class: "flex justify-between items-start",
                    div {
                        div { class: "text-5xl font-bold leading-none",
                            "{agg.level:.0}"
                            span { class: "text-lg text-neutral-500", "%" }
                        }
                        div {
                            class: if agg.any_charging {
                                "text-xs mt-2 tracking-widest text-yellow-300"
                            } else {
                                "text-xs mt-2 tracking-widest text-neutral-500"
                            },
                            if agg.all_full { "● FULL" }
                            else if agg.any_charging { "▲ CHARGING" }
                            else { "ON BATTERY" }
                        }
                        if agg.count > 1 {
                            div { class: "text-xs text-neutral-600 mt-1",
                                "{agg.count} batteries"
                            }
                        }
                    }
                    BatteryIcon { level: agg.level, charging: agg.any_charging }
                }
                div { class: "mt-4 h-0.5 bg-neutral-800 rounded overflow-hidden",
                    div {
                        class: "h-full bg-yellow-300 transition-all duration-500",
                        style: "width: {agg.level:.0}%",
                    }
                }
            }

            if agg.count > 1 {
                Card {
                    CardLabel { "Batteries" }
                    for battery in data.batteries.iter() {
                        div { class: "mb-4 last:mb-0",
                            div { class: "text-xs text-neutral-400 mb-2",
                                "Battery {battery.index + 1}"
                            }
                            StatRow {
                                label: "Level",
                                value: format!("{:.0}%", battery.level),
                                unit: ""
                            }
                            StatRow {
                                label: "Voltage",
                                value: format!("{:.2}", battery.voltage),
                                unit: "V"
                            }
                            StatRow {
                                label: "Wattage",
                                value: format!("{:.2}", battery.wattage),
                                unit: "W"
                            }
                            StatRow {
                                label: "State",
                                value: battery.state.clone(),
                                unit: ""
                            }
                        }
                    }
                }
            }

            Card {
                CardLabel {
                    if agg.count > 1 { "Combined Electrical" } else { "Electrical" }
                }
                StatRow {
                    label: "Total Wattage",
                    value: format!("{:.2}", agg.total_wattage),
                    unit: "W"
                }
                if agg.count == 1 {
                    if let Some(b) = data.batteries.first() {
                        StatRow { label: "Voltage", value: format!("{:.2}", b.voltage), unit: "V" }
                        StatRow { label: "Current", value: format!("{:.0}", b.current), unit: "mA" }
                        StatRow { label: "State", value: b.state.clone(), unit: "" }
                    }
                }
            }

            Card {
                CardLabel { "Health" }
                StatRow {
                    label: "Total Capacity",
                    value: format!("{:.0}", agg.total_capacity),
                    unit: "mAh"
                }
                for battery in data.batteries.iter() {
                    if agg.count > 1 {
                        StatRow {
                            label: "Battery {battery.index + 1} health",
                            value: battery.health.clone(),
                            unit: ""
                        }
                    } else {
                        StatRow { label: "Health", value: battery.health.clone(), unit: "" }
                        StatRow { label: "Temperature", value: format!("{:.1}", battery.temperature), unit: "°C" }
                    }
                }
            }

            Card {
                CardLabel { "Level history" }
                Sparkline { data: history.read().clone() }
            }
        }
    }
}

#[component]
fn BatteryIcon(level: f32, charging: bool) -> Element {
    let fill_color = if level < 20.0 {
        "bg-red-500"
    } else if charging {
        "bg-yellow-300"
    } else {
        "bg-neutral-400"
    };

    rsx! {
        div { class: "relative flex flex-col items-center",
            div { class: "w-3 h-1 bg-neutral-700 rounded-t mx-auto" }
            div { class: "w-8 h-16 border border-neutral-700 rounded relative overflow-hidden",
                div {
                    class: "absolute bottom-0 left-0 right-0 transition-all duration-500 {fill_color}",
                    style: "height: {level:.0}%",
                }
            }
        }
    }
}

#[component]
fn Sparkline(data: Vec<f32>) -> Element {
    if data.len() < 2 {
        return rsx! { div { class: "h-12 bg-neutral-900 rounded" } };
    }

    let w = 320.0f32;
    let h = 48.0f32;
    let max = data
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        .max(1.0);

    let points: String = data
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = (i as f32 / (data.len() - 1) as f32) * w;
            let y = h - (v / max) * h;
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    rsx! {
        svg {
            width: "{w}",
            height: "{h}",
            view_box: "0 0 {w} {h}",
            polyline {
                points: "{points}",
                fill: "none",
                stroke: "#fde047",
                stroke_width: "1.5",
                stroke_linejoin: "round",
                stroke_linecap: "round",
            }
        }
    }
}

#[component]
fn Card(children: Element) -> Element {
    rsx! {
        div { class: "bg-neutral-900 border border-neutral-800 rounded p-4",
            {children}
        }
    }
}

#[component]
fn CardLabel(children: Element) -> Element {
    rsx! {
        div { class: "text-xs text-neutral-500 uppercase tracking-widest mb-3",
            {children}
        }
    }
}

#[component]
fn StatRow(label: String, value: String, unit: String) -> Element {
    rsx! {
        div { class: "flex justify-between items-baseline border-b border-neutral-800 pb-2 mb-2 last:border-0 last:mb-0 last:pb-0",
            span { class: "text-xs text-neutral-500 uppercase tracking-wider", "{label}" }
            span { class: "text-sm text-neutral-100",
                "{value}"
                span { class: "text-xs text-neutral-500 ml-1", "{unit}" }
            }
        }
    }
}

#[component]
fn Cpu() -> Element {
    rsx! {
        div {
            class: "text-neutral-100 font mono bg-neutral-950",
            "Cpu Screen"
        }
    }
}

#[component]
fn Ram() -> Element {
    rsx! {
        div {
            class: "text-neutral-100 font mono",
            "Ram Screen"
        }
    }
}
