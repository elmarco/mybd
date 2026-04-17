use crate::{AlbumHit, AuthorHit, Client, CollectionHit, PublisherHit, Result, SeriesHit, TagHit};

impl Client {
    /// Search the Algolia **Series** index by title.
    ///
    /// Returns up to 20 hits. The Algolia index does not include work-type
    /// metadata (manga / comic / BD) — use [`Client::get_series`] on a hit's
    /// `object_id` to obtain the full [`Series`](crate::Series) with its
    /// [`WorkType`](crate::WorkType).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let hits = client.search_series("Tintin").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_series(&self, query: &str) -> Result<Vec<SeriesHit>> {
        let hits = self.algolia_query("Series", query, 20).await?;
        let results = Self::parse_series_hits(&hits);
        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    /// Like [`search_series`](Self::search_series) but fetches **all** pages
    /// from Algolia (up to 100 hits per page).
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_series_all(&self, query: &str) -> Result<Vec<SeriesHit>> {
        let hits = self.algolia_query_all("Series", query, 100).await?;
        let results = Self::parse_series_hits(&hits);
        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    fn parse_series_hits(hits: &[serde_json::Value]) -> Vec<SeriesHit> {
        hits.iter()
            .filter_map(|hit| {
                let title = hit["title"].as_str()?.to_string();
                let object_id = hit["objectId"].as_str()?.to_string();
                let cover_url = hit["imageUrl"].as_str().map(String::from);
                let note = hit["note"].as_f64();

                let permalink = hit["permalink"].as_str().map(String::from);
                let collection = hit["collection"].as_str().map(String::from);
                let is_terminated = hit["isTerminated"].as_bool();
                let series_type = hit["type"].as_str().map(String::from);
                let has_sexual_content = hit["hasSexualContent"].as_bool();

                Some(SeriesHit {
                    object_id,
                    title,
                    cover_url,
                    note,
                    permalink,
                    collection,
                    is_terminated,
                    series_type,
                    has_sexual_content,
                })
            })
            .collect()
    }

    /// Search the Algolia **Albums** index by EAN barcode.
    ///
    /// Returns up to 5 hits, filtered to only include albums whose `eans`
    /// field contains the queried barcode.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let hits = client.search_albums_by_ean("9782203001015").await?;
    ///
    /// for hit in &hits {
    ///     println!("{}: {}", hit.title, hit.series_object_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_albums_by_ean(&self, ean: &str) -> Result<Vec<AlbumHit>> {
        let hits = self.algolia_query("Albums", ean, 5).await?;

        let results: Vec<_> = hits
            .iter()
            .filter(|hit| hit["eans"].as_str().is_some_and(|eans| eans.contains(ean)))
            .filter_map(|hit| {
                let object_id = hit["objectId"].as_str()?.to_string();
                let title = hit["title"].as_str().unwrap_or("").to_string();
                let serie_title = hit["serieTitle"].as_str().map(String::from);
                let tome = hit["tome"].as_i64();
                let display_title = match &serie_title {
                    Some(st) if !st.is_empty() => match tome {
                        Some(t) => format!("{st} - T{t}"),
                        None => st.clone(),
                    },
                    _ => title,
                };

                let series_object_id = hit["serieObjectId"]
                    .as_str()
                    .or_else(|| hit["objectId"].as_str())?
                    .to_string();

                let cover_url = hit["imageUrl"].as_str().map(String::from);
                let note = hit["note"].as_f64();
                let number_of_notes = hit["numberOfNotes"].as_i64();
                let ean_value = hit["eans"]
                    .as_str()
                    .and_then(|e| e.split(';').next())
                    .map(String::from);
                let permalink = hit["permalink"].as_str().map(String::from);
                let price = hit["price"].as_str().map(String::from);
                let serie_permalink = hit["seriePermalink"].as_str().map(String::from);
                let default_selling_print_object_id = hit["defaultSellingPrintObjectId"]
                    .as_str()
                    .map(String::from);
                let has_sexual_content = hit["hasSexualContent"].as_bool();

                Some(AlbumHit {
                    object_id,
                    series_object_id,
                    title: display_title,
                    cover_url,
                    note,
                    number_of_notes,
                    ean: ean_value,
                    tome,
                    permalink,
                    price,
                    serie_title,
                    serie_permalink,
                    default_selling_print_object_id,
                    has_sexual_content,
                })
            })
            .collect();

        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    /// Search the Algolia **Authors** index by name.
    ///
    /// Returns up to 20 author hits. For full biographical details, fetch an
    /// album containing the author — author data is embedded in [`crate::Print`]
    /// objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let authors = client.search_authors("Hergé").await?;
    ///
    /// for author in &authors {
    ///     println!("{} ({})", author.display_name, author.object_id);
    ///     if let Some(year) = &author.year_of_birth {
    ///         println!("  Born: {year}");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_authors(&self, query: &str) -> Result<Vec<AuthorHit>> {
        let hits = self.algolia_query("Authors", query, 20).await?;

        let results: Vec<_> = hits
            .iter()
            .filter_map(|hit| {
                let object_id = hit["objectId"].as_str()?.to_string();
                Some(AuthorHit {
                    object_id,
                    permalink: hit["permalink"].as_str().map(String::from),
                    display_name: hit["displayName"].as_str().unwrap_or("").to_string(),
                    image_url: hit["imageUrl"].as_str().map(String::from),
                    year_of_birth: hit["yearOfBirth"].as_str().map(String::from),
                    year_of_death: hit["yearOfDeath"].as_str().map(String::from),
                })
            })
            .collect();

        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    /// Search the Algolia **Publishers** index by name.
    ///
    /// Returns up to 20 publisher hits.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let publishers = client.search_publishers("Glénat").await?;
    ///
    /// for pub_hit in &publishers {
    ///     println!("{}: {}", pub_hit.name, pub_hit.object_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_publishers(&self, query: &str) -> Result<Vec<PublisherHit>> {
        let hits = self.algolia_query("Publishers", query, 20).await?;

        let results: Vec<_> = hits
            .iter()
            .filter_map(|hit| {
                let object_id = hit["objectId"].as_str()?.to_string();
                // The Publishers index uses "name" or "name_raw" for the display name.
                let name = hit["name"]
                    .as_str()
                    .or_else(|| hit["name_raw"].as_str())?
                    .to_string();
                Some(PublisherHit { object_id, name })
            })
            .collect();

        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    /// Search the Algolia **Tags** index by name.
    ///
    /// Returns up to 20 tag hits. Tags represent genres and themes (e.g.
    /// "Action", "Humour", "Science-Fiction"). The `weight` field indicates
    /// popularity — higher values mean the tag is used on more works.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let tags = client.search_tags("action").await?;
    ///
    /// for tag in &tags {
    ///     println!("{} (weight: {})", tag.name, tag.weight);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_tags(&self, query: &str) -> Result<Vec<TagHit>> {
        let hits = self.algolia_query("Tags", query, 20).await?;

        let results: Vec<_> = hits
            .iter()
            .filter_map(|hit| {
                let object_id = hit["objectId"].as_str()?.to_string();
                let name = hit["name"].as_str()?.to_string();
                let weight = hit["weight"].as_i64().unwrap_or(0);
                Some(TagHit {
                    object_id,
                    name,
                    weight,
                })
            })
            .collect();

        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    /// Search the Algolia **Collections** index by name.
    ///
    /// Collections are publisher imprint lines (e.g. "Panini Manga",
    /// "Soleil Manga Seinen", "Glénat Shonen Manga").
    ///
    /// Returns up to 20 collection hits.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be
    /// parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let collections = client.search_collections("manga").await?;
    ///
    /// for coll in &collections {
    ///     println!("{}: {}", coll.name, coll.object_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(hits))]
    pub async fn search_collections(&self, query: &str) -> Result<Vec<CollectionHit>> {
        let hits = self.algolia_query("Collections", query, 20).await?;

        let results: Vec<_> = hits
            .iter()
            .filter_map(|hit| {
                let object_id = hit["objectId"].as_str()?.to_string();
                let name = hit["nameFrench"]
                    .as_str()
                    .or_else(|| hit["name"].as_str())?
                    .to_string();
                Some(CollectionHit { object_id, name })
            })
            .collect();

        tracing::Span::current().record("hits", results.len());
        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Send a query to the Algolia multi-index endpoint and return the raw hits
    /// array from the first result.
    #[tracing::instrument(skip(self))]
    async fn algolia_query(
        &self,
        index_name: &str,
        query: &str,
        hits_per_page: u32,
    ) -> Result<Vec<serde_json::Value>> {
        let (hits, _) = self
            .algolia_query_page(index_name, query, hits_per_page, 0)
            .await?;
        Ok(hits)
    }

    /// Send a paginated query. Returns `(hits, nb_pages)`.
    #[tracing::instrument(skip(self))]
    async fn algolia_query_page(
        &self,
        index_name: &str,
        query: &str,
        hits_per_page: u32,
        page: u32,
    ) -> Result<(Vec<serde_json::Value>, u32)> {
        let url = format!("{}/1/indexes/*/queries", self.algolia_base_url);

        let body = serde_json::json!({
            "requests": [{
                "indexName": index_name,
                "params": format!(
                    "query={}&hitsPerPage={hits_per_page}&page={page}&attributesToHighlight=[]",
                    urlencoding::encode(query)
                )
            }]
        });

        tracing::debug!(url = %url, %index_name, %query, %page, "sending Algolia query");

        let resp = self
            .http
            .post(&url)
            .header("x-algolia-application-id", &self.algolia_app_id)
            .header("x-algolia-api-key", &self.algolia_api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let data: serde_json::Value = resp.json().await?;

        let result = &data["results"][0];
        let hits = result["hits"].as_array().cloned().unwrap_or_default();
        let nb_pages = result["nbPages"].as_u64().unwrap_or(1) as u32;

        tracing::debug!(%status, hits = hits.len(), %nb_pages, "Algolia query complete");

        Ok((hits, nb_pages))
    }

    /// Fetch all pages for a given index/query and return all raw hits.
    async fn algolia_query_all(
        &self,
        index_name: &str,
        query: &str,
        hits_per_page: u32,
    ) -> Result<Vec<serde_json::Value>> {
        let (mut all_hits, nb_pages) = self
            .algolia_query_page(index_name, query, hits_per_page, 0)
            .await?;

        for page in 1..nb_pages {
            let (hits, _) = self
                .algolia_query_page(index_name, query, hits_per_page, page)
                .await?;
            if hits.is_empty() {
                break;
            }
            all_hits.extend(hits);
        }

        Ok(all_hits)
    }
}
