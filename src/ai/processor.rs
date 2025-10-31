use anyhow::Result;
use scraper::{Html, Selector};

pub struct ContentProcessor;

impl ContentProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_text(&self, html: &str) -> Result<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").unwrap();
        
        let body = document.select(&selector).next();
        if let Some(body) = body {
            Ok(body.text().collect::<Vec<_>>().join(" "))
        } else {
            Ok(String::new())
        }
    }

    pub fn extract_links(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a[href]").unwrap();
        
        let mut links = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                links.push(href.to_string());
            }
        }
        
        Ok(links)
    }

    pub fn extract_title(&self, html: &str) -> Result<Option<String>> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("title").unwrap();
        
        if let Some(title) = document.select(&selector).next() {
            Ok(Some(title.text().collect::<Vec<_>>().join(" ")))
        } else {
            Ok(None)
        }
    }
}
