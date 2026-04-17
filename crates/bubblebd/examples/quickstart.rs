#[tokio::main]
async fn main() -> bubblebd::Result<()> {
    tracing_subscriber::fmt::init();

    let client = bubblebd::Client::new();

    // Search for series
    let hits = client.search_series("One Piece").await?;

    // Get full series details
    if let Some(hit) = hits.first() {
        let (series, albums) = client.get_series(&hit.object_id).await?;
        println!(
            "{} ({} volumes, {} returned)",
            series.title,
            series.number_of_albums.unwrap_or(0),
            albums.len(),
        );
    }

    // Look up an album by EAN barcode
    let _albums = client.search_albums_by_ean("9782203001015").await?;

    // Search authors, publishers, tags, collections
    let _authors = client.search_authors("Hergé").await?;
    let _publishers = client.search_publishers("Glénat").await?;
    let _tags = client.search_tags("action").await?;
    let _collections = client.search_collections("manga").await?;

    // Get full album details (prints, pricing, dimensions, …)
    let album = client.get_album("Xd9C4X5s50ZtAk").await?;
    dbg!(&album);
    for print in &album.prints {
        println!(
            "  EAN {:?}, {} pages, publisher: {:?}",
            print.ean,
            print.number_of_pages.unwrap_or(0),
            print.publisher.as_ref().map(|p| &p.name),
        );
    }

    Ok(())
}
