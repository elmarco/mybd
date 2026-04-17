use crate::{Client, Record, Result, parse, types::SearchResults};

impl Client {
    pub async fn search_by_title(&self, query: &str, max: u32) -> Result<SearchResults> {
        let cql = format!(r#"bib.title all "{query}" and bib.doctype all "a""#);
        self.sru_query(&cql, max, 1).await
    }

    pub async fn search_by_isbn(&self, isbn: &str) -> Result<SearchResults> {
        let cql = format!(r#"bib.isbn all "{isbn}""#);
        self.sru_query(&cql, 10, 1).await
    }

    pub async fn search_by_author(&self, author: &str, max: u32) -> Result<SearchResults> {
        let cql = format!(r#"bib.author all "{author}" and bib.doctype all "a""#);
        self.sru_query(&cql, max, 1).await
    }

    pub async fn search_by_series(&self, series_title: &str, max: u32) -> Result<SearchResults> {
        let fetch = (max * 3).max(50);
        let cql = format!(r#"bib.title all "{series_title}" and bib.doctype all "a""#);
        let mut results = self.sru_query(&cql, fetch, 1).await?;
        results.records.retain(|r| r.series.is_some());
        results.records.truncate(max as usize);
        Ok(results)
    }

    pub async fn get_by_ark(&self, ark: &str) -> Result<Option<Record>> {
        let cql = format!(r#"bib.persistentid all "{ark}""#);
        let mut results = self.sru_query(&cql, 1, 1).await?;
        Ok(results.records.pop())
    }

    pub async fn fetch_more(&self, prev: &SearchResults) -> Result<SearchResults> {
        self.sru_query(&prev.cql, prev.page_size, prev.next_start)
            .await
    }

    #[tracing::instrument(skip(self))]
    async fn sru_query(&self, cql: &str, max: u32, start: u32) -> Result<SearchResults> {
        let url = format!(
            "{}/api/SRU?version=1.2&operation=searchRetrieve&query={}&recordSchema=unimarcXchange&maximumRecords={max}&startRecord={start}",
            self.base_url,
            urlencoding::encode(cql),
        );

        tracing::debug!(%url, "SRU query");

        let resp = self.http.get(&url).send().await?;
        let status = resp.status();

        if !status.is_success() {
            return Err(crate::Error::Status(status.as_u16()));
        }

        let body = resp.text().await?;
        let (records, total) = parse::parse_response(&body)?;
        let count = records.len() as u32;

        Ok(SearchResults {
            records,
            total,
            cql: cql.to_string(),
            next_start: start + count,
            page_size: max,
        })
    }
}
