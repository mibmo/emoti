mod config;
use config::*;

use eyre::{Result as EResult, WrapErr};
use rofi::{
    pango::{FontSize, Pango},
    Rofi,
};
use arboard::Clipboard;
use std::thread::sleep;

fn main() -> EResult<()> {
    let config = smart()?;

    let left_padding = config
        .mappings
        .iter()
        .map(|(k, _)| k.len())
        .max()
        .expect("no entries");

    let pango = pango_builder(&config.style);

    let entries: Vec<_> = config
        .mappings
        .iter()
        .map(|(key, value)| {
            let content = format!("{:left$}\t{}", key, value, left = left_padding);
            pango.build_content(&content)
        })
        .collect();

    let mut clipboard = Clipboard::new().wrap_err("failed to get access to the clipboard")?;
    let index = Rofi::new(&entries).pango().run_index().wrap_err("failed getting selection")?;
    let text = config.mappings.values().nth(index).unwrap();
    clipboard.set_text(text.into()).wrap_err("failed to use clipboard")?;

    // keep this process alive because of clipboard
    sleep(std::time::Duration::from_secs(60));

    Ok(())
}

fn pango_builder(style: &config::Style) -> Pango {
    let mut pango = Pango::new("");
    pango.size(FontSize::Small)
        .size(style.size)
        .fg_color(&style.fg_color);

    pango
}
