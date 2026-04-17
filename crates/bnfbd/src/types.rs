#[derive(Debug, Clone)]
pub struct SearchResults {
    pub records: Vec<Record>,
    pub total: u32,
    pub(crate) cql: String,
    pub(crate) next_start: u32,
    pub(crate) page_size: u32,
}

impl SearchResults {
    pub fn has_more(&self) -> bool {
        self.next_start <= self.total
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub ark: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub publisher: Option<String>,
    pub pub_date: Option<String>,
    pub pages: Option<String>,
    pub dimensions: Option<String>,
    pub isbn: Option<String>,
    pub ean: Option<String>,
    pub series: Option<String>,
    pub volume: Option<i32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub first_name: Option<String>,
    pub dates: Option<String>,
    pub role_code: Option<String>,
    pub bnf_id: Option<String>,
    pub isni: Option<String>,
}

impl Author {
    pub fn display_name(&self) -> String {
        match &self.first_name {
            Some(first) => format!("{} {}", first, self.name),
            None => self.name.clone(),
        }
    }
}
