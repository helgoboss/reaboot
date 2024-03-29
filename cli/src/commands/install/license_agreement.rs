use anyhow::Context;
use console::Key;
use reaboot_core::reaper_util::get_reaper_eula;
use std::fs;
use std::time::Duration;

pub async fn confirm_license() -> anyhow::Result<bool> {
    let term = console::Term::stdout();
    loop {
        term.clear_screen();
        println!("ReaBoot is going to download and install REAPER because it's not yet installed at the location of your choice.\n\nIn order to continue, you need to accept the REAPER license agreement.\n");
        println!("Please choose:");
        println!("- [S]how the REAPER license agreement");
        println!("- [A]gree");
        println!("- [D]isagree");
        match term.read_key().ok() {
            Some(Key::Char('s')) => {
                let eula = get_reaper_eula().await.context("Couldn't download REAPER EULA. Consider using command line option `--non-interactive`.")?;
                let _ = term.clear_screen();
                for line in eula.lines() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    println!("{line}");
                }
                println!("\n\n<Press any key to go back>");
                let _ = term.read_key();
            }
            Some(Key::Char('a')) => {
                let _ = term.clear_screen();
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }
}
