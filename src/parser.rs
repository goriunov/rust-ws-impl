use fnv::FnvHashMap;
use http_muncher::{Parser, ParserHandler};

pub struct HttpHeaders {
    pub headers: FnvHashMap<String, String>,
    pub field: String,
    pub value: String,
}

impl HttpHeaders {
    pub fn new() -> Self {
        HttpHeaders {
            headers: FnvHashMap::default(),
            field: String::new(),
            value: String::new(),
        }
    }
    pub fn get_all_headers(self) -> FnvHashMap<String, String> {
        self.headers
    }
}

impl ParserHandler for HttpHeaders {
    fn on_header_field(&mut self, _: &mut Parser, header: &[u8]) -> bool {
        self.field = String::from_utf8(header.to_vec()).unwrap();
        true
    }
    fn on_header_value(&mut self, _: &mut Parser, value: &[u8]) -> bool {
        self.value = String::from_utf8(value.to_vec()).unwrap();
        if !self.field.is_empty() && !self.value.is_empty() {
            self.headers.insert(self.field.clone(), self.value.clone());
            // reset values
            self.field = String::new();
            self.value = String::new();
        }
        true
    }
}
