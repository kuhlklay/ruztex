mod color;
mod registries;
mod register;
mod localization;

#[allow(unused_imports)]
use std::{thread, time::Duration};
use std::collections::HashMap;
use std::borrow::Cow;

use registries::REGISTRY;
use localization::{Language, Translator, TranslationID};
use color::{Color, ColorRef, GradientDirection};

fn main() -> Result<(), String> {
    // Add custom colors
    let _ = color::add_color("custom", "my_red", Color::from_hex("#ff0055"));
    let _ = color::add_color("custom", "my_blue", Color::from_hex("#1e90ff"));
    let _ = color::add_color("custom", "my_green", Color::from_hex("#00ff00"));
    let _ = color::add_color("pastel", "red", Color::from_hex("#ff7f7f"));

    // Print colored text
    /* println!(
        "{}\n\n",
        color::coloredText("Hello, World!", &ColorRef::Named("pastel", "red"))?
    );

    // Print gradient text (horizontal, reset each line)
    let colored = color::gradient_text(
        "aaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\naaaaaaa",
        &[
            ColorRef::Named("custom", "my_red"),
            ColorRef::Direct(Color { r: 255, g: 255, b: 0 }),
            ColorRef::Named("custom", "my_blue"),
        ],
        GradientDirection::Horizontal,
        Some(true)
    )?;
    println!("{}\n\n", colored);

    // Print gradient text (horizontal, no reset)
    println!(
        "{}\n\n",
        color::gradient_text(
            "aaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\naaaaaaa",
            &[
                ColorRef::Named("custom", "my_red"),
                ColorRef::Direct(Color { r: 255, g: 255, b: 0 }),
                ColorRef::Named("custom", "my_blue"),
            ],
            GradientDirection::Horizontal,
            Some(false)
        )?
    );

    // Print gradient text (vertical)
    println!(
        "{}\n\n",
        color::gradient_text(
            "aaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa\naaaaaaa",
            &[
                ColorRef::Named("custom", "my_red"),
                ColorRef::Direct(Color { r: 255, g: 255, b: 0 }),
                ColorRef::Named("custom", "my_blue"),
            ],
            GradientDirection::Vertical,
            None
        )?
    );

    // Print rainbow text (horizontal, reset each line)
    println!(
        "{}\n\n",
        color::rainbow_text(
            "aaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\naaaaaaa",
            GradientDirection::Horizontal,
            Some(true)
        )?
    );

    // Print rainbow text (horizontal, no reset)
    println!(
        "{}\n\n",
        color::rainbow_text(
            "aaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\naaaaaaa",
            GradientDirection::Horizontal,
            Some(false)
        )?
    ); */

    register::register();

    // Print registered tags
    let registry = REGISTRY.lock().unwrap();

    for (tag_id, tag) in &registry.tags {
        println!("Tag: {}", tag);
        for (typ, entity_id) in &tag.entries {
            println!("  {}: {}", typ, entity_id);
        }
    }

    let lang = Language { name: "Deutsch".to_string(), code: "en_US".to_string() };
    let translator = Translator::load(lang.clone(), format!("lang/{}.yaml", lang.code)).unwrap();

    // Ohne Platzhalter
    println!("{}", translator.translate(&TranslationID::from("examplemod:item.hammer"), None)); // z.B. "Hammer" oder fallback "examplemod:item.hammer"

    // Mit Platzhalter
    println!("{}", translator.translate(&TranslationID::from("examplemod:misc.greeting"), Some(&HashMap::from([
        ("p", Cow::Owned(color::colored_text("Kuhly", &ColorRef::Named("custom", "my_red")).unwrap())),
    ])))); // z.B. "Hallo, Kuhly!"

    println!("{}", translator.translate(&TranslationID::from("examplemod:misc.greeting"), Some(&HashMap::from([
        ("p", Cow::Owned(color::rainbow_text("Kuhly", GradientDirection::Horizontal, Some(true)).unwrap())),
    ])))); // z.B. "Hallo, Kuhly!"

    println!("{}", translator.translate(&TranslationID::from("examplemod:misc.coca_cola"), Some(&HashMap::from([
        ("c", Cow::Owned(format!("{}, {} - {}",
        color::gradient_text("Coca Cola Light", &[
            ColorRef::Direct(Color::from_hex("#2A7B9B")),
            ColorRef::Direct(Color::from_hex("#88AA78")),
            ColorRef::Direct(Color::from_hex("#EDDD53")),
        ], GradientDirection::Horizontal, Some(true)).unwrap(),
        color::gradient_text("Coca Cola Normal", &[
            ColorRef::Direct(Color::from_hex("#2A7B9B")),
            ColorRef::Direct(Color::from_hex("#88AA78")),
            ColorRef::Direct(Color::from_hex("#53C9ED")),
        ], GradientDirection::Horizontal, Some(true)).unwrap(),
        color::gradient_text("Coca Cola Z-z-z-zeroooo", &[
            ColorRef::Direct(Color::from_hex("#9B5D2A")),
            ColorRef::Direct(Color::from_hex("#AA7895")),
            ColorRef::Direct(Color::from_hex("#53C9ED")),
        ], GradientDirection::Horizontal, Some(true)).unwrap()))),
    ]))));
    Ok(())
}