use std::io;
use std::io::Write;
use std::time::Duration;
use clap::Parser;
use anyhow::Result;
use tokio::time::Instant;
use notation::notion::client::NotionClient;

const BANNER: &str = r#"
 _,  _,____, ____,____,____,__, ____, _,  _,
(-|\ |(-/  \(-|  (-/_|(-|  (-| (-/  \(-|\ |
 _| \|,_\__/,_|, _/  |,_|,  _|_,_\__/,_| \|,
(     (     (   (     (    (   (     (
"#;

#[derive(Parser, Debug)]
#[clap(name = "notation")]
#[clap(bin_name = "notation")]
enum NotationCLI {
    Clear,
    Ship(ShipParams)
}

#[derive(clap::Args, Debug)]
#[clap(author, version, about, long_about = None)]
struct ShipParams {
    #[clap(short, long, value_parser)]
    pub src: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = NotationCLI::parse();
    let nc = NotionClient::new()?;
    let parent_page_id = nc.get_parent_id_by_name(nc.parent_page_name()).await?;
    let parsed_page_name = nc.parent_page_name().replace(" ", "-").to_lowercase();
    let parent_page_url = format!("https://www.notion.so/{}-{}", parsed_page_name, parent_page_id.replace("-", ""));

    println!("\n{}\n", BANNER);
    println!("👋👋 Notation workspace hosted by parent page \"{}\"", nc.parent_page_name());
    println!("🔗🔗 {}\n", parent_page_url);

    match args {
        NotationCLI::Clear => {
            let page_content = nc.get_page_content_by_id(nc.get_parent_id_by_name(nc.parent_page_name()).await?).await?;
            let page_content_len = page_content.results.len();
            println!("This page has {} pieces of content on it.", page_content_len);
            if page_content_len > 0 {
                println!("\nFor example...\n");
                for (i, r) in page_content.results.iter().take(5).enumerate() {
                    println!("Content ({}): {}", i, r.content_type);
                }
                println!();
                println!("Press ENTER to proceed with clearing this Notation workspace...");
                let mut line = String::new();
                let _ = io::stdin().read_line(&mut line).unwrap();
            }
            nc.clear().await?;
            println!("\n🧹🧹 Notation workspace cleared! ✅ ");
        }
        NotationCLI::Ship(params) => {
            let nc_clone = nc.clone();
            let mut h = tokio::spawn(async move {
                nc_clone.create_pages(params.src, false).await
            });
            let start = Instant::now();
            loop {
                tokio::select! {
                    r = &mut h => {
                        r??;
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {
                        print!("\r🚢🚢 Shipping pages, one moment... {}s", start.elapsed().as_secs());
                        io::stdout().flush().unwrap();
                    }
                }
            }
            println!("\n\nNotation pages shipped! ✅ \nSee you next time 🫡");
        }
    }

    Ok(())
}