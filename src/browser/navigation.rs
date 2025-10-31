use url::Url;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Navigation {
    history: VecDeque<String>,
    current_index: usize,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            current_index: 0,
        }
    }

    pub fn navigate(&mut self, url: String) -> Result<Url, url::ParseError> {
        let parsed = Url::parse(&url)?;
        self.history.push_back(url);
        self.current_index = self.history.len() - 1;
        Ok(parsed)
    }

    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.current_index < self.history.len() - 1
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.can_go_back() {
            self.current_index -= 1;
            self.history.get(self.current_index).cloned()
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if self.can_go_forward() {
            self.current_index += 1;
            self.history.get(self.current_index).cloned()
        } else {
            None
        }
    }

    pub fn current_url(&self) -> Option<&String> {
        self.history.get(self.current_index)
    }
}
