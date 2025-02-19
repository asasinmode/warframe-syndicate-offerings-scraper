use std::process;

use dialoguer::{theme::ColorfulTheme, Select};

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

    println!("Will use {}", syndicate);
}
