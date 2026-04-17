use bubblebd::Client;

fn full_album_json() -> serde_json::Value {
    serde_json::json!({
        "objectId": "Xd9C4X5s50ZtAk",
        "permalink": "sakamoto-days-tome-21",
        "title": null,
        "tome": 21,
        "summary": "An action-packed volume.",
        "note": 3.8,
        "numberOfNotes": 4,
        "images": {
            "front": {
                "original": "https://example.com/original.jpg",
                "large": "https://example.com/large.jpg",
                "medium": "https://example.com/medium.jpg"
            }
        },
        "tags": [
            { "objectId": "t1", "name": "Action" },
            { "objectId": "t2", "name": "Shōnen" }
        ],
        "prints": [{
            "objectId": "print1",
            "ean": "9782344067321",
            "isbn": null,
            "publicationDate": "2026-04-08T00:00:00.000Z",
            "numberOfPages": 208,
            "length": 18.1,
            "height": 1.5,
            "width": 11.8,
            "weight": 0.146,
            "type": "album simple N&B",
            "collection": null,
            "images": {
                "front": {
                    "large": "https://example.com/print-large.jpg"
                }
            },
            "publisher": {
                "objectId": "pub1",
                "name": "Glénat"
            },
            "authors": [{
                "objectId": "aut1",
                "permalink": "yuto-suzuki",
                "displayName": "Yuto Suzuki",
                "role": "auteur",
                "firstName": "Yuto",
                "lastName": "Suzuki",
                "imageUrl": null,
                "yearOfBirth": null,
                "yearOfDeath": null,
                "biography": null
            }],
            "sellingInfo": {
                "price": "7.20",
                "discountedPrice": null,
                "online": {
                    "numberOfSellers": 1,
                    "estimatedDeliveryDate": "2026-04-15T00:00:00.000Z",
                    "availability": {
                        "message": "En stock",
                        "code": 100,
                        "color": "#62ca22"
                    }
                },
                "clickAndCollect": {
                    "numberOfSellers": 72,
                    "availability": {
                        "message": "72 librairies",
                        "code": 100,
                        "color": "#62ca22"
                    }
                }
            }
        }],
        "serie": {
            "objectId": "ser1",
            "title": "Sakamoto Days",
            "note": 4.6,
            "numberOfNotes": 53,
            "category": "Mangas",
            "permalink": "sakamoto-days"
        }
    })
}

#[tokio::test]
async fn get_album_full_response() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1.6/albums/Xd9C4X5s50ZtAk")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(full_album_json().to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("Xd9C4X5s50ZtAk").await.unwrap();

    assert_eq!(album.object_id, "Xd9C4X5s50ZtAk");
    assert_eq!(album.permalink, Some("sakamoto-days-tome-21".to_string()));
    assert_eq!(album.title, None);
    assert_eq!(album.tome, Some(21));
    assert_eq!(album.summary, Some("An action-packed volume.".to_string()));
    assert_eq!(album.note, Some(3.8));
    assert_eq!(album.number_of_notes, Some(4));
    assert_eq!(
        album.cover_url,
        Some("https://example.com/large.jpg".to_string())
    );

    // Tags
    assert_eq!(album.tags.len(), 2);
    assert_eq!(album.tags[0].name, "Action");
    assert_eq!(album.tags[1].name, "Shōnen");

    // Serie back-reference
    let serie = album.serie.as_ref().unwrap();
    assert_eq!(serie.object_id, "ser1");
    assert_eq!(serie.title, "Sakamoto Days");
    assert_eq!(serie.note, Some(4.6));
    assert_eq!(serie.number_of_notes, Some(53));
    assert_eq!(serie.category, Some("Mangas".to_string()));
    assert_eq!(serie.permalink, Some("sakamoto-days".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn get_album_print_details() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/Xd9C4X5s50ZtAk")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(full_album_json().to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("Xd9C4X5s50ZtAk").await.unwrap();

    assert_eq!(album.prints.len(), 1);
    let print = &album.prints[0];

    assert_eq!(print.object_id, "print1");
    assert_eq!(print.ean, Some("9782344067321".to_string()));
    assert_eq!(print.isbn, None);
    assert_eq!(
        print.publication_date,
        Some("2026-04-08T00:00:00.000Z".to_string())
    );
    assert_eq!(print.number_of_pages, Some(208));
    assert_eq!(print.length_cm, Some(18.1));
    assert_eq!(print.height_cm, Some(1.5));
    assert_eq!(print.width_cm, Some(11.8));
    assert_eq!(print.weight_kg, Some(0.146));
    assert_eq!(print.print_type, Some("album simple N&B".to_string()));
    assert_eq!(print.collection, None);
    assert_eq!(
        print.cover_url,
        Some("https://example.com/print-large.jpg".to_string())
    );

    // Publisher
    let publisher = print.publisher.as_ref().unwrap();
    assert_eq!(publisher.object_id, "pub1");
    assert_eq!(publisher.name, "Glénat");

    // Authors
    assert_eq!(print.authors.len(), 1);
    assert_eq!(print.authors[0].object_id, "aut1");
    assert_eq!(print.authors[0].display_name, "Yuto Suzuki");
    assert_eq!(print.authors[0].role, Some("auteur".to_string()));
    assert_eq!(print.authors[0].permalink, Some("yuto-suzuki".to_string()));
    assert_eq!(print.authors[0].first_name, Some("Yuto".to_string()));
    assert_eq!(print.authors[0].last_name, Some("Suzuki".to_string()));
}

#[tokio::test]
async fn get_album_selling_info() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/Xd9C4X5s50ZtAk")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(full_album_json().to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("Xd9C4X5s50ZtAk").await.unwrap();
    let selling = album.prints[0].selling_info.as_ref().unwrap();

    assert_eq!(selling.price, Some("7.20".to_string()));
    assert_eq!(selling.discounted_price, None);

    let online = selling.online.as_ref().unwrap();
    assert_eq!(online.message, "En stock");
    assert_eq!(online.code, 100);
    assert_eq!(online.number_of_sellers, 1);

    let cc = selling.click_and_collect.as_ref().unwrap();
    assert_eq!(cc.message, "72 librairies");
    assert_eq!(cc.number_of_sellers, 72);
}

#[tokio::test]
async fn get_album_minimal_response() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/min1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::json!({}).to_string())
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("min1").await.unwrap();

    assert_eq!(album.object_id, "min1");
    assert_eq!(album.title, None);
    assert_eq!(album.tome, None);
    assert_eq!(album.summary, None);
    assert_eq!(album.note, None);
    assert_eq!(album.cover_url, None);
    assert!(album.tags.is_empty());
    assert!(album.prints.is_empty());
    assert!(album.serie.is_none());
}

#[tokio::test]
async fn get_album_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/bad")
        .with_status(400)
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let err = client.get_album("bad").await.unwrap_err();
    assert!(err.to_string().contains("400"));
}

#[tokio::test]
async fn get_album_cover_falls_back_to_medium() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/fb1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "images": {
                    "front": {
                        "medium": "https://example.com/medium.jpg"
                    }
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("fb1").await.unwrap();
    assert_eq!(
        album.cover_url,
        Some("https://example.com/medium.jpg".to_string())
    );
}

#[tokio::test]
async fn get_album_multiple_prints() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1.6/albums/multi")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "prints": [
                    { "objectId": "p1", "ean": "1111111111111" },
                    { "objectId": "p2", "ean": "2222222222222", "numberOfPages": 300 }
                ]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = Client::with_base_urls("", &server.url());
    let album = client.get_album("multi").await.unwrap();

    assert_eq!(album.prints.len(), 2);
    assert_eq!(album.prints[0].ean, Some("1111111111111".to_string()));
    assert_eq!(album.prints[1].ean, Some("2222222222222".to_string()));
    assert_eq!(album.prints[1].number_of_pages, Some(300));
}
