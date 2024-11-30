use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE};
use urlencoding;

use scraper::html::Html;
use scraper::selector::Selector;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
}

#[derive(Debug, Clone)]
pub struct RecipeDetails {
    pub materials: Vec<String>,
    pub instructions: Vec<String>,
}

fn create_client() -> Result<reqwest::blocking::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;
    
    Ok(client)
}

pub fn search_recipes(search_term: &str) -> Result<Vec<SearchResult>> {
    let url = format!("https://www.nefisyemektarifleri.com/ara/page/1/?s={}", urlencoding::encode(search_term));
    
    let client = create_client()?;
    let response = client.get(&url).send()?;
    let body = response.text()?;
    
    let document = Html::parse_document(&body);
    
    let title_selector = Selector::parse("a.title").unwrap();
    
    let results: Vec<SearchResult> = document.select(&title_selector)
        .map(|element| SearchResult {
            title: element.text().collect::<String>(),
            link: element.value().attr("href").unwrap_or("").to_string(),
        })
        .collect();
    
    Ok(results)
}

pub fn fetch_recipe_details(url: &str) -> Result<RecipeDetails> {
    let client = create_client()?;
    let response = client.get(url).send()?;
    let body = response.text()?;
    
    let document = Html::parse_document(&body);
    
    let materials_selector = Selector::parse("ul.recipe-materials li").unwrap();
    let instructions_selector = Selector::parse("ol.recipe-instructions > li").unwrap();
    
    let materials: Vec<String> = document.select(&materials_selector)
        .map(|element| element.text().collect::<String>())
        .collect();
    
    let instructions: Vec<String> = document.select(&instructions_selector)
        .map(|element| element.text().collect::<String>())
        .collect();
    
    Ok(RecipeDetails { 
        materials, 
        instructions 
    })
}
