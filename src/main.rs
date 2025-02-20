// use dialoguer::{theme::ColorfulTheme, Select};
use scraper::{Html, Node, Selector};

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

fn main() {
    // let theme = ColorfulTheme::default();

    // let syndicate = Select::with_theme(&theme)
    //     .with_prompt("Select a syndicate")
    //     .items(&[
    //         "Steel_Meridian",
    //         "Arbiters of Hexis",
    //         "Cephalon Suda",
    //         "The Perrin Sequence",
    //         "Red Veil",
    //         "New Loka",
    //     ])
    //     .interact_opt()
    //     .unwrap_or_else(|_| process::exit(0))
    //     .map(|index| match index {
    //         0 => "Steel_Meridian",
    //         1 => "Arbiters_of_Hexis",
    //         2 => "Cephalon_Suda",
    //         3 => "The_Perrin_Sequence",
    //         4 => "Red_Veil",
    //         5 => "New_Loka",
    //         _ => unreachable!(),
    //     })
    //     .unwrap_or_else(|| process::exit(0));
    let syndicate = "Arbiters_of_Hexis";

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
                    offerings.push(offering.to_string());
                }
            }
        }
    }

    println!("{:?}", offerings);
}
