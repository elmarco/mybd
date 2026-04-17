use bubblebd::{Client, WorkType};

fn full_series_json() -> serde_json::Value {
    serde_json::json!({
        "title": "Les Aventures de Tintin",
        "permalink": "les-aventures-de-tintin",
        "category": "bd",
        "collection": "Collection Tintin",
        "genre": "Aventure",
        "isTerminated": true,
        "note": 4.8,
        "numberOfNotes": 150,
        "numberOfAlbums": 24,
        "descriptionShort": "A classic BD series.",
        "albums": [{
            "objectId": "album1",
            "tome": 1,
            "title": "Tintin au pays des Soviets",
            "prints": [{
                "ean": "9782203001015",
                "publicationDate": "1930-01-10",
                "authors": [
                    { "displayName": "Hergé", "role": "auteur" },
                    { "displayName": "Hergé", "role": "dessinateur" }
                ]
            }],
            "images": {
                "front": {
                    "large": "https://example.com/tintin-cover.jpg"
                }
            }
        }, {
            "objectId": "album2",
            "tome": 2,
            "title": "Tintin au Congo",
            "prints": [{
                "ean": "9782203001022"
            }],
            "images": {
                "front": {
                    "medium": "https://example.com/tintin2-cover.jpg"
                }
            }
        }],
        "tags": [
            { "objectId": "tag1", "name": "Aventure" },
            { "objectId": "tag2", "name": "Classique" }
        ]
    })
}

#[tokio::test]
async fn get_series_full_response() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1.6/series/abc123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(full_series_json().to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, albums) = client.get_series("abc123").await.unwrap();

    assert_eq!(series.object_id, "abc123");
    assert_eq!(series.title, "Les Aventures de Tintin");
    assert_eq!(series.work_type, WorkType::Bd);
    assert_eq!(series.description, Some("A classic BD series.".to_string()));
    assert_eq!(series.isbn, Some("9782203001015".to_string()));
    assert_eq!(
        series.cover_url,
        Some("https://example.com/tintin-cover.jpg".to_string())
    );
    assert_eq!(series.year, Some(1930));

    // New fields
    assert_eq!(
        series.permalink,
        Some("les-aventures-de-tintin".to_string())
    );
    assert_eq!(series.collection, Some("Collection Tintin".to_string()));
    assert_eq!(series.genre, Some("Aventure".to_string()));
    assert_eq!(series.is_terminated, Some(true));
    assert_eq!(series.note, Some(4.8));
    assert_eq!(series.number_of_notes, Some(150));
    assert_eq!(series.number_of_albums, Some(24));
    assert_eq!(series.tags.len(), 2);
    assert_eq!(series.tags[0].name, "Aventure");
    assert_eq!(series.tags[1].object_id, "tag2");

    // Album parsing
    assert_eq!(albums.len(), 2);
    assert_eq!(albums[0].object_id, "album1");
    assert_eq!(albums[0].tome, Some(1));
    assert_eq!(
        albums[0].title,
        Some("Tintin au pays des Soviets".to_string())
    );
    assert_eq!(
        albums[0].cover_url,
        Some("https://example.com/tintin-cover.jpg".to_string())
    );
    assert_eq!(albums[0].ean, Some("9782203001015".to_string()));
    assert_eq!(albums[1].object_id, "album2");
    assert_eq!(albums[1].tome, Some(2));
    assert_eq!(albums[1].ean, Some("9782203001022".to_string()));
    // Falls back to medium when large is missing
    assert_eq!(
        albums[1].cover_url,
        Some("https://example.com/tintin2-cover.jpg".to_string())
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn get_series_minimal_response() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/min1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::json!({ "title": "Minimal" }).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, albums) = client.get_series("min1").await.unwrap();

    assert_eq!(series.title, "Minimal");
    assert_eq!(series.work_type, WorkType::Bd);
    assert_eq!(series.description, None);
    assert_eq!(series.isbn, None);
    assert_eq!(series.cover_url, None);
    assert_eq!(series.year, None);
    assert_eq!(series.permalink, None);
    assert_eq!(series.collection, None);
    assert_eq!(series.genre, None);
    assert_eq!(series.is_terminated, None);
    assert_eq!(series.note, None);
    assert_eq!(series.number_of_notes, None);
    assert_eq!(series.number_of_albums, None);
    assert!(series.tags.is_empty());
    assert!(albums.is_empty());
}

#[tokio::test]
async fn get_series_missing_title_defaults_to_untitled() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/no_title")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::json!({}).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, _) = client.get_series("no_title").await.unwrap();

    assert_eq!(series.title, "Untitled");
}

#[tokio::test]
async fn get_series_manga_category() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/manga1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "title": "One Piece",
                "category": "mangas"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, _) = client.get_series("manga1").await.unwrap();
    assert_eq!(series.work_type, WorkType::Manga);
}

#[tokio::test]
async fn get_series_multi_author_response() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/multi_author")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "title": "Multi Author",
                "albums": [{
                    "objectId": "album1",
                    "tome": 1,
                    "prints": [{
                        "authors": [
                            { "displayName": "Artist First", "role": "dessinateur" },
                            { "displayName": "Writer Second", "role": "scénariste" }
                        ]
                    }]
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, albums) = client.get_series("multi_author").await.unwrap();

    assert_eq!(series.title, "Multi Author");
    assert_eq!(albums.len(), 1);
}

#[tokio::test]
async fn get_series_uses_description_long_as_fallback() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/long_desc")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "title": "Long Desc",
                "descriptionLong": "A long description."
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let (series, _) = client.get_series("long_desc").await.unwrap();
    assert_eq!(series.description, Some("A long description.".to_string()));
}

#[tokio::test]
async fn get_series_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/series/bad")
        .with_status(404)
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let err = client.get_series("bad").await.unwrap_err();
    assert!(err.to_string().contains("404"));
}
