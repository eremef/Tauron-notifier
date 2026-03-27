use crate::api_logic::{AlertSource, UnifiedAlert};
use regex::Regex;
use serde::Deserialize;

pub const ENERGA_BASE_URL: &str = "https://energa-operator.pl";
pub const ENERGA_PAGE_URL: &str =
    "https://energa-operator.pl/uslugi/awarie-i-wylaczenia/wylaczenia-planowane";

#[derive(Debug, Deserialize)]
pub struct EnergaResponse {
    pub document: EnergaDocument,
}

#[derive(Debug, Deserialize)]
pub struct EnergaDocument {
    pub payload: EnergaPayload,
}

#[derive(Debug, Deserialize)]
pub struct EnergaPayload {
    pub shutdowns: Vec<EnergaShutdown>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnergaShutdown {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub message: Option<String>,
}

impl EnergaShutdown {
    pub fn to_unified(&self) -> UnifiedAlert {
        UnifiedAlert {
            source: AlertSource::Energa,
            startDate: self.start_date.clone(),
            endDate: self.end_date.clone(),
            message: self.message.clone(),
            description: None,
            address_index: None,
            is_local: None,
        }
    }

    pub fn matches_address(
        &self,
        city: &str,
        street_name_1: &str,
        street_name_2: &Option<String>,
    ) -> bool {
        let Some(message) = &self.message else {
            return false;
        };

        fn word_match(text: &str, word: &str) -> bool {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(word));
            regex::Regex::new(&pattern)
                .map(|r| r.is_match(text))
                .unwrap_or(false)
        }

        // Match city
        if !word_match(message, city) {
            return false;
        }

        // Match street logic
        let mut candidates: Vec<String> = Vec::new();

        if let Some(n2) = street_name_2 {
            let compound = format!("{} {}", n2.trim(), street_name_1.trim());
            candidates.push(compound);
        }

        // Add significant individual words (>= 3 chars)
        for word in street_name_1.split_whitespace() {
            if word.len() >= 3 {
                candidates.push(word.to_string());
            }
        }
        if let Some(n2) = street_name_2 {
            for word in n2.split_whitespace() {
                if word.len() >= 3 {
                    candidates.push(word.to_string());
                }
            }
        }

        if candidates.is_empty() {
            return true;
        }

        candidates.iter().any(|c| word_match(message, c))
    }
}

pub async fn extract_energa_api_url(client: &reqwest::Client) -> Result<String, String> {
    let res = client
        .get(ENERGA_PAGE_URL)
        .header("accept", "text/html")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Energa page: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Energa HTML page HTTP error: {}", res.status()));
    }

    let html = res
        .text()
        .await
        .map_err(|e| format!("Failed to read Energa HTML text: {}", e))?;

    let re = Regex::new(r#"data-shutdowns="([^"]+)""#)
        .map_err(|e| format!("Regex compilation failed: {}", e))?;

    if let Some(caps) = re.captures(&html) {
        if let Some(suffix) = caps.get(1) {
            let url = format!("{}{}", ENERGA_BASE_URL, suffix.as_str());
            return Ok(url);
        }
    }

    Err("Could not extract data-shutdowns URL suffix from Energa page HTML".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energa_address_match() {
        let shutdown = EnergaShutdown {
            start_date: None,
            end_date: None,
            message: Some("Gdańsk ulica św. Brata Alberta 14, 15, 17, 19".to_string()),
        };

        // Complete match -> true
        assert!(shutdown.matches_address("Gdańsk", "Brata Alberta", &Some("św.".to_string())));

        // Match with just the street components -> true
        assert!(shutdown.matches_address("Gdańsk", "Brata Alberta", &None));

        // Missing city should fail
        assert!(!shutdown.matches_address("Warszawa", "Brata Alberta", &Some("św.".to_string())));

        // Matching city but completely wrong street -> should fail
        assert!(!shutdown.matches_address("Gdańsk", "Długa", &None));
    }
}
