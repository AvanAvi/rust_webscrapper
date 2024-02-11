
use reqwest::Error;
use std::env; 
use std::fs::File;
use std::io::Write;


#[tokio::main]
async fn main() {
    
    if let Err(e) = run().await {
        eprintln!("Application error: {}", e);
    }
}


async fn run() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    let url = args.get(1).expect("Usage: cargo run <URL>");


    let res = reqwest::get(url).await?;

    if res.status().is_success() {
        let body = res.text().await?;

       
        let mut raw_file = File::create("scrapped_data(raw).html")?;
        writeln!(raw_file, "{}", body)?;

        let filtered_text = "Filtered and translated text here.";
        let mut filtered_file = File::create("filtered_data(only_text_with_translation).txt")?;
        writeln!(filtered_file, "{}", filtered_text)?;

        println!("Scraping done and data stored in 'scrapped_data(raw).html' and 'filtered_data(only_text_with_translation).txt'");
    } else {
        // Handle HTTP errors
        eprintln!("HTTP Request failed with status: {}", res.status());
    }

    Ok(())
}
