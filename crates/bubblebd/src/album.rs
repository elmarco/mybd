use crate::{
    Album, AlbumSerie, Author, Availability, Client, Error, Print, Publisher, Result, SellingInfo,
    Tag,
};

impl Client {
    /// Fetch detailed album info from the Bubble BD REST API.
    ///
    /// Calls `GET /v1.6/albums/{object_id}` and returns an [`Album`] with
    /// all print editions, authors, pricing, and availability data.
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
    /// let album = client.get_album("Xd9C4X5s50ZtAk").await?;
    ///
    /// println!("Tome {:?}: {:?}", album.tome, album.title);
    /// for print in &album.prints {
    ///     if let Some(ean) = &print.ean {
    ///         println!("  EAN: {ean}");
    ///     }
    ///     if let Some(pages) = print.number_of_pages {
    ///         println!("  {pages} pages");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_album(&self, object_id: &str) -> Result<Album> {
        let url = format!("{}/v1.6/albums/{object_id}", self.api_base_url);
        tracing::debug!(%url, "fetching album");

        let resp = self
            .http
            .get(&url)
            .header("User-Agent", "mybd/0.1")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            tracing::debug!(%status, "album fetch failed");
            return Err(Error::Status(status.as_u16()));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| Error::Parse(e.to_string()))?;
        let album = parse_album(object_id, &data);
        tracing::debug!(%status, prints = album.prints.len(), "album fetched");
        Ok(album)
    }
}

fn parse_album(object_id: &str, data: &serde_json::Value) -> Album {
    let title = data["title"].as_str().map(String::from);
    let permalink = data["permalink"].as_str().map(String::from);
    let tome = data["tome"].as_i64();
    let summary = data["summary"].as_str().map(String::from);
    let note = data["note"].as_f64();
    let number_of_notes = data["numberOfNotes"].as_i64();

    let cover_url = data["images"]["front"]["large"]
        .as_str()
        .or_else(|| data["images"]["front"]["medium"].as_str())
        .map(String::from);

    let tags = parse_tags(&data["tags"]);
    let prints = parse_prints(&data["prints"]);
    let serie = parse_serie(&data["serie"]);

    Album {
        object_id: object_id.to_string(),
        permalink,
        title,
        tome,
        summary,
        note,
        number_of_notes,
        cover_url,
        tags,
        prints,
        serie,
    }
}

fn parse_tags(val: &serde_json::Value) -> Vec<Tag> {
    val.as_array()
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
        .unwrap_or_default()
}

fn parse_prints(val: &serde_json::Value) -> Vec<Print> {
    val.as_array()
        .map(|arr| arr.iter().map(parse_single_print).collect())
        .unwrap_or_default()
}

fn parse_single_print(p: &serde_json::Value) -> Print {
    let object_id = p["objectId"].as_str().unwrap_or("").to_string();
    let ean = p["ean"].as_str().map(String::from);
    let isbn = p["isbn"].as_str().map(String::from);
    let publication_date = p["publicationDate"].as_str().map(String::from);
    let number_of_pages = p["numberOfPages"].as_i64();
    let length_cm = p["length"].as_f64();
    let height_cm = p["height"].as_f64();
    let width_cm = p["width"].as_f64();
    let weight_kg = p["weight"].as_f64();
    let print_type = p["type"].as_str().map(String::from);
    let collection = p["collection"].as_str().map(String::from);

    let cover_url = p["images"]["front"]["large"]
        .as_str()
        .or_else(|| p["images"]["front"]["medium"].as_str())
        .map(String::from);

    let publisher = p["publisher"]["objectId"].as_str().map(|id| Publisher {
        object_id: id.to_string(),
        name: p["publisher"]["name"].as_str().unwrap_or("").to_string(),
    });

    let authors = p["authors"]
        .as_array()
        .map(|arr| arr.iter().filter_map(parse_author).collect())
        .unwrap_or_default();

    let selling_info = parse_selling_info(&p["sellingInfo"]);

    Print {
        object_id,
        ean,
        isbn,
        publication_date,
        number_of_pages,
        length_cm,
        height_cm,
        width_cm,
        weight_kg,
        publisher,
        print_type,
        collection,
        cover_url,
        authors,
        selling_info,
    }
}

fn parse_author(a: &serde_json::Value) -> Option<Author> {
    let object_id = a["objectId"].as_str()?.to_string();
    Some(Author {
        object_id,
        permalink: a["permalink"].as_str().map(String::from),
        display_name: a["displayName"].as_str().unwrap_or("").to_string(),
        role: a["role"].as_str().map(String::from),
        first_name: a["firstName"].as_str().map(String::from),
        last_name: a["lastName"].as_str().map(String::from),
        image_url: a["imageUrl"].as_str().map(String::from),
        year_of_birth: a["yearOfBirth"].as_str().map(String::from),
        year_of_death: a["yearOfDeath"].as_str().map(String::from),
        biography: a["biography"].as_str().map(String::from),
    })
}

fn parse_availability(val: &serde_json::Value) -> Option<Availability> {
    Some(Availability {
        message: val["availability"]["message"].as_str()?.to_string(),
        code: val["availability"]["code"].as_i64()?,
        number_of_sellers: val["numberOfSellers"].as_i64().unwrap_or(0),
    })
}

fn parse_selling_info(val: &serde_json::Value) -> Option<SellingInfo> {
    if val.is_null() {
        return None;
    }
    Some(SellingInfo {
        price: val["price"].as_str().map(String::from),
        discounted_price: val["discountedPrice"].as_str().map(String::from),
        online: parse_availability(&val["online"]),
        click_and_collect: parse_availability(&val["clickAndCollect"]),
    })
}

fn parse_serie(val: &serde_json::Value) -> Option<AlbumSerie> {
    let object_id = val["objectId"].as_str()?.to_string();
    Some(AlbumSerie {
        object_id,
        title: val["title"].as_str().unwrap_or("").to_string(),
        note: val["note"].as_f64(),
        number_of_notes: val["numberOfNotes"].as_i64(),
        category: val["category"].as_str().map(String::from),
        permalink: val["permalink"].as_str().map(String::from),
    })
}
