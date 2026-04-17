use bubblebd::Client;

fn algolia_response(hits: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "results": [{
            "hits": hits,
            "nbHits": hits.as_array().map(|a| a.len()).unwrap_or(0)
        }]
    })
}

// ---------------------------------------------------------------------------
// Series search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_series_returns_hits() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                {
                    "objectId": "abc123",
                    "title": "Tintin",
                    "imageUrl": "https://example.com/tintin.jpg",
                    "note": 4.5
                },
                {
                    "objectId": "def456",
                    "title": "One Piece",
                    "imageUrl": "https://example.com/onepiece.jpg",
                    "note": 4.8
                }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_series("test").await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Tintin");
    assert_eq!(results[0].object_id, "abc123");
    assert_eq!(results[0].note, Some(4.5));
    assert_eq!(results[1].title, "One Piece");
    mock.assert_async().await;
}

#[tokio::test]
async fn search_series_empty_results() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(algolia_response(serde_json::json!([])).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_series("nonexistent").await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_series_skips_hits_without_title() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "a" },
                { "objectId": "b", "title": "Valid" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_series("test").await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Valid");
}

// ---------------------------------------------------------------------------
// Albums EAN search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_albums_by_ean_returns_matching() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                {
                    "objectId": "alb1",
                    "serieObjectId": "ser1",
                    "title": "Album 1",
                    "serieTitle": "Series One",
                    "tome": 3,
                    "eans": "9782203001015",
                    "imageUrl": "https://example.com/cover.jpg",
                    "note": 4.0
                },
                {
                    "objectId": "alb2",
                    "title": "Album 2",
                    "eans": "9999999999999",
                    "note": 3.0
                }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_albums_by_ean("9782203001015").await.unwrap();

    // Only the first hit has matching EAN
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].series_object_id, "ser1");
    assert_eq!(results[0].title, "Series One - T3");
    assert_eq!(results[0].ean, Some("9782203001015".to_string()));
}

#[tokio::test]
async fn search_albums_by_ean_no_matches() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(algolia_response(serde_json::json!([])).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_albums_by_ean("0000000000000").await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_albums_uses_serie_title_without_tome() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([{
                "objectId": "alb1",
                "serieObjectId": "ser1",
                "title": "Album Title",
                "serieTitle": "Series Name",
                "eans": "1234567890123"
            }]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_albums_by_ean("1234567890123").await.unwrap();

    assert_eq!(results[0].title, "Series Name");
}

#[tokio::test]
async fn search_albums_falls_back_to_object_id() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([{
                "objectId": "fallback_id",
                "title": "Standalone",
                "eans": "1111111111111"
            }]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_albums_by_ean("1111111111111").await.unwrap();

    assert_eq!(results[0].series_object_id, "fallback_id");
    // No serieTitle, so uses album title
    assert_eq!(results[0].title, "Standalone");
}

// ---------------------------------------------------------------------------
// Authors search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_authors_returns_hits() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                {
                    "objectId": "aut1",
                    "permalink": "herge",
                    "displayName": "Hergé",
                    "imageUrl": "https://example.com/herge.jpg",
                    "yearOfBirth": "1907",
                    "yearOfDeath": "1983"
                },
                {
                    "objectId": "aut2",
                    "permalink": "studios-herge",
                    "displayName": "Studios Hergé",
                    "imageUrl": null,
                    "yearOfBirth": null,
                    "yearOfDeath": null
                }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_authors("Hergé").await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].object_id, "aut1");
    assert_eq!(results[0].display_name, "Hergé");
    assert_eq!(results[0].permalink, Some("herge".to_string()));
    assert_eq!(
        results[0].image_url,
        Some("https://example.com/herge.jpg".to_string())
    );
    assert_eq!(results[0].year_of_birth, Some("1907".to_string()));
    assert_eq!(results[0].year_of_death, Some("1983".to_string()));
    assert_eq!(results[1].display_name, "Studios Hergé");
    assert_eq!(results[1].image_url, None);
    assert_eq!(results[1].year_of_birth, None);

    mock.assert_async().await;
}

#[tokio::test]
async fn search_authors_skips_hits_without_object_id() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "displayName": "No ID" },
                { "objectId": "valid", "displayName": "Has ID" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_authors("test").await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].object_id, "valid");
}

// ---------------------------------------------------------------------------
// Publishers search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_publishers_returns_hits() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "pub1", "name": "Glénat", "name_raw": "GLENAT" },
                { "objectId": "pub2", "name": null, "name_raw": "Dargaud" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_publishers("Glénat").await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].object_id, "pub1");
    assert_eq!(results[0].name, "Glénat");
    // Second hit falls back to name_raw since name is null
    assert_eq!(results[1].name, "Dargaud");

    mock.assert_async().await;
}

#[tokio::test]
async fn search_publishers_skips_hits_without_any_name() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "pub1" },
                { "objectId": "pub2", "name": "Valid" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_publishers("test").await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Valid");
}

// ---------------------------------------------------------------------------
// Tags search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_tags_returns_hits() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "t1", "name": "Action", "weight": 1496 },
                { "objectId": "t2", "name": "Action Et Aventure", "weight": 676 }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_tags("action").await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].object_id, "t1");
    assert_eq!(results[0].name, "Action");
    assert_eq!(results[0].weight, 1496);
    assert_eq!(results[1].weight, 676);

    mock.assert_async().await;
}

#[tokio::test]
async fn search_tags_defaults_weight_to_zero() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "t1", "name": "NoWeight" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_tags("test").await.unwrap();

    assert_eq!(results[0].weight, 0);
}

// ---------------------------------------------------------------------------
// Collections search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_collections_returns_hits() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "c1", "nameFrench": "Panini Manga" },
                { "objectId": "c2", "nameFrench": "Soleil Manga Seinen" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_collections("manga").await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].object_id, "c1");
    assert_eq!(results[0].name, "Panini Manga");
    assert_eq!(results[1].name, "Soleil Manga Seinen");

    mock.assert_async().await;
}

#[tokio::test]
async fn search_collections_falls_back_to_name() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            algolia_response(serde_json::json!([
                { "objectId": "c1", "name": "Fallback Name" }
            ]))
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_collections("test").await.unwrap();

    assert_eq!(results[0].name, "Fallback Name");
}

#[tokio::test]
async fn search_collections_empty_results() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/1/indexes/*/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(algolia_response(serde_json::json!([])).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls(&server.url(), "");
    let results = client.search_collections("zzz").await.unwrap();
    assert!(results.is_empty());
}
