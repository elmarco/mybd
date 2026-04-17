use crate::{AlbumInfo, Client, Error, Result, Series, Tag, WorkType};

impl Client {
    /// Fetch detailed series info from the Bubble BD REST API.
    ///
    /// Calls `GET /v1.6/series/{object_id}` and parses the response into a
    /// [`Series`] struct, extracting top-level metadata plus ISBN/cover
    /// data from the first album's first print.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] for non-2xx responses (e.g. 404 for unknown
    /// IDs) and [`Error::Parse`] if the JSON body cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> bubblebd::Result<()> {
    /// let client = bubblebd::Client::new();
    /// let (series, albums) = client.get_series("0xcxuigcLForBk").await?;
    ///
    /// println!("{} ({})", series.title, series.work_type);
    /// println!("{} albums returned", albums.len());
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_series(&self, object_id: &str) -> Result<(Series, Vec<AlbumInfo>)> {
        let url = format!("{}/v1.6/series/{object_id}", self.api_base_url);
        tracing::debug!(%url, "fetching series");

        let resp = self
            .http
            .get(&url)
            .header("User-Agent", "mybd/0.1")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            tracing::debug!(%status, "series fetch failed");
            return Err(Error::Status(status.as_u16()));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| Error::Parse(e.to_string()))?;
        tracing::debug!(%status, "series fetched");

        let title = data["title"].as_str().unwrap_or("Untitled").to_string();

        let work_type = WorkType::from_category(data["category"].as_str(), data["type"].as_str());

        let description = data["descriptionShort"]
            .as_str()
            .or_else(|| data["descriptionLong"].as_str())
            .map(String::from);

        let first_album = data["albums"].as_array().and_then(|a| a.first());
        let first_print = first_album
            .and_then(|album| album["prints"].as_array())
            .and_then(|prints| prints.first());

        let isbn = first_print
            .and_then(|print| print["ean"].as_str())
            .map(String::from);

        let cover_url = first_album
            .and_then(|album| album["images"]["front"]["large"].as_str())
            .map(String::from);

        let year = first_print
            .and_then(|print| print["publicationDate"].as_str())
            .and_then(|d| d.get(..4))
            .and_then(|y| y.parse().ok());

        let permalink = data["permalink"].as_str().map(String::from);
        let collection = data["collection"].as_str().map(String::from);
        let genre = data["genre"].as_str().map(String::from);
        let is_terminated = data["isTerminated"].as_bool();
        let note = data["note"].as_f64();
        let number_of_notes = data["numberOfNotes"].as_i64();
        let number_of_albums = data["numberOfAlbums"].as_i64();

        let tags = data["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| {
                        Some(Tag {
                            object_id: t["objectId"].as_str()?.to_string(),
                            name: t["name"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse all albums from the response array
        let albums: Vec<AlbumInfo> = data["albums"]
            .as_array()
            .map(|arr| arr.iter().filter_map(parse_album_info).collect())
            .unwrap_or_default();

        let series = Series {
            object_id: object_id.to_string(),
            title,
            work_type,
            description,
            isbn,
            cover_url,
            year,
            permalink,
            collection,
            genre,
            is_terminated,
            note,
            number_of_notes,
            number_of_albums,
            tags,
        };

        tracing::debug!(albums = albums.len(), "parsed series albums");
        Ok((series, albums))
    }
}

fn parse_album_info(album: &serde_json::Value) -> Option<AlbumInfo> {
    let object_id = album["objectId"].as_str()?.to_string();
    let tome = album["tome"].as_i64();
    let title = album["title"].as_str().map(String::from);
    let cover_url = album["images"]["front"]["large"]
        .as_str()
        .or_else(|| album["images"]["front"]["medium"].as_str())
        .map(String::from);
    let ean = album["prints"]
        .as_array()
        .and_then(|prints| prints.first())
        .and_then(|p| p["ean"].as_str())
        .map(String::from);

    Some(AlbumInfo {
        object_id,
        tome,
        title,
        cover_url,
        ean,
    })
}
