use std::io::prelude::*;
use std::{
    collections::HashMap,
    fs, io, process, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dialoguer::{theme::ColorfulTheme, Select};
use scraper::{Html, Node, Selector};
use serde::{Deserialize, Serialize};

const UNTRADEABLE_OFFERINGS: [&'static str; 16] = [
    "adapter",
    "ammo restore",
    "armor set",
    "emote",
    "energy restore",
    "health restore",
    "relic pack",
    "scene",
    "sculpture",
    "shield restore",
    "sigil",
    "simulacrum",
    "specter",
    "stencil",
    "syandana",
    "vosfor",
];

#[derive(Debug, Serialize, Deserialize)]
struct PriceData {
    lowest_5_prices: Vec<u64>,
    checked_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    version: String,
    prices: HashMap<String, PriceData>,
}

fn main() {
    let theme = ColorfulTheme::default();

    let syndicate = Select::with_theme(&theme)
        .with_prompt("Select a syndicate")
        .items(&[
            "Steel_Meridian",
            "Arbiters of Hexis",
            "Cephalon Suda",
            "The Perrin Sequence",
            "Red Veil",
            "New Loka",
        ])
        .interact_opt()
        .unwrap_or_else(|_| process::exit(0))
        .map(|index| match index {
            0 => "Steel_Meridian",
            1 => "Arbiters_of_Hexis",
            2 => "Cephalon_Suda",
            3 => "The_Perrin_Sequence",
            4 => "Red_Veil",
            5 => "New_Loka",
            _ => unreachable!(),
        })
        .unwrap_or_else(|| process::exit(0));

    let reqwest_client = reqwest::blocking::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0",
        )
        .use_rustls_tls()
        .build()
        .expect("failed to create http client");

    let wiki_page = reqwest_client
        .get(format!("https://wiki.warframe.com/w/{}", syndicate))
        .send()
        .expect("failed to get wiki syndicate page")
        .text()
        .expect("failed to parse wiki page");

    let document = Html::parse_document(&wiki_page);
    let offerings_selector = Selector::parse("#Offerings").unwrap();
    let offerings_container = document
        .select(&offerings_selector)
        .next()
        .and_then(|span_heading| span_heading.parent())
        .and_then(|header| header.next_sibling())
        .and_then(|newline| newline.next_sibling())
        .and_then(|toggle_button_container| toggle_button_container.next_sibling())
        .and_then(|newline| newline.next_sibling())
        .and_then(|offerings_container_parent| offerings_container_parent.children().nth(1)) // first is newline
        .expect("failed to find offerings container element");

    let mut offerings: Vec<String> = Vec::new();

    for offering_container in offerings_container.children() {
        if !offering_container.value().is_element() {
            continue;
        }

        let text_node = offering_container
            .children()
            .nth_back(1)
            .and_then(|link_container| link_container.first_child())
            .and_then(|link| link.first_child())
            .and_then(|span| span.first_child());

        if let Some(text) = text_node {
            if let Node::Text(offering) = text.value() {
                if !offering.is_empty()
                    && !UNTRADEABLE_OFFERINGS
                        .iter()
                        .any(|&untradeable| offering.to_lowercase().contains(untradeable))
                {
                    offerings.push(offering.trim().to_string());
                }
            }
        }
    }

    let mut prices: HashMap<String, PriceData> = HashMap::new();
    let version = String::from("v1");
    let cache_file = "./.asasinmode_offerings_cache.json";

    if fs::metadata(cache_file).is_ok() {
        match fs::read_to_string(cache_file) {
            Ok(data) => {
                let parsed_cache: Result<CacheData, _> = serde_json::from_str(&data);
                match parsed_cache {
                    Ok(data) => {
                        if data.version == version {
                            prices = data.prices;
                        } else {
                            fs::remove_file(cache_file)
                                .expect("failed to remove outdated cache file");
                        }
                    }
                    Err(e) => {
                        eprintln!("failed to parse cache, clearing: {}", e);
                        fs::remove_file(cache_file)
                            .expect("failed to remove unparseable cache file");
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to read cache file: {}", e);
                fs::remove_file(cache_file).expect("failed to remove faulty cache file");
            }
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get current time")
        .as_millis() as u64;
    let offerings_len = offerings.len();

    for (i, offering) in offerings.iter_mut().enumerate() {
        if let Some(opening_paren_index) = offering.find('(') {
            offering.truncate(opening_paren_index - 1);
        }

        // edge cases, wiki data doesn't match warframe market
        if offering == "Negation Armor" {
            *offering = "Negation Swarm".to_string();
        } else if offering == "Fluctus Limb" {
            *offering = "Fluctus Limbs".to_string();
        }

        println!("\x1B[32m{}\x1B[0m {}/{}", offering, i + 1, offerings_len);

        if let Some(cache_hit) = prices.get(offering) {
            if (now - cache_hit.checked_at) < 300_000 {
                continue;
            }
        }

        // timeout to respect warframe.market TOS api rate limit
        thread::sleep(Duration::from_millis(400));

        let mut lowest_5_prices: Vec<u64> = Vec::new();

        match get_orders(offering, &reqwest_client) {
            Some(orders) => {
                for order in orders {
                    if order["order_type"] == "buy" || order["user"]["status"] == "offline" {
                        continue;
                    }

                    let platinum = order["platinum"].as_u64().unwrap_or(u16::MAX as u64);
                    match lowest_5_prices.iter().position(|&low| low > platinum) {
                        Some(insert_index) => lowest_5_prices.insert(insert_index, platinum),
                        None => lowest_5_prices.push(platinum),
                    }

                    lowest_5_prices.truncate(5);
                }
            }
            None => continue,
        }

        prices.insert(
            offering.clone(),
            PriceData {
                lowest_5_prices,
                checked_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("failed to get the current time")
                    .as_millis() as u64,
            },
        );
    }

    if !offerings.is_empty() {
        println!("--------------------")
    }

    let mut sorted_prices: Vec<(&String, &PriceData)> = prices.iter().collect();

    sorted_prices.sort_by(|(_, a), (_, b)| {
        a.lowest_5_prices
            .get(0)
            .unwrap_or(&0)
            .cmp(b.lowest_5_prices.get(0).unwrap_or(&0))
    });

    let mut log = String::new();
    for (offering, price_data) in sorted_prices {
        let prices: Vec<String> = price_data
            .lowest_5_prices
            .iter()
            .map(|i| format!("\x1B[33m{}\x1B[0m", i))
            .collect();
        log.push_str(&format!("{}: {}\n", offering, prices.join(", ")));
    }

    println!("{}", log);

    let cache_data = CacheData { version, prices };
    fs::write(
        cache_file,
        serde_json::to_string(&cache_data).expect("failed to stringify cache data"),
    )
    .expect("failed to write cache file");

    // prevent insta closing after finish if someone were to run the exe directly
    println!("\x1B[36mpress enter to exit...\x1B[0m");
    io::stdin().read(&mut [0u8]).unwrap();
}

fn get_orders(
    offering: &str,
    reqwest_client: &reqwest::blocking::Client,
) -> Option<Vec<serde_json::Value>> {
    let url_name = offering
        .replace(' ', "_")
        .replace('&', "and")
        .replace('\'', "")
        .to_lowercase();

    let market_response = reqwest_client
        .get(format!(
            "https://api.warframe.market/v1/items/{}/orders",
            url_name
        ))
        .send()
        .and_then(|response| response.json::<serde_json::Value>());

    match market_response {
        Ok(response) => match response["payload"]["orders"].as_array() {
            Some(orders) => Some(orders.to_vec()),
            None => {
                eprintln!("unexpected market response structure: {:?}", response);
                None
            }
        },
        Err(e) => {
            eprintln!("failed to get market response: {}", e);
            None
        }
    }
}
