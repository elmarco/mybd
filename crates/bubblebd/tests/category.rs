use bubblebd::WorkType;

#[test]
fn manga_category() {
    assert_eq!(
        WorkType::from_category(Some("mangas"), None),
        WorkType::Manga
    );
    assert_eq!(
        WorkType::from_category(Some("Manga"), None),
        WorkType::Manga
    );
    assert_eq!(
        WorkType::from_category(Some("MANGAS"), None),
        WorkType::Manga
    );
}

#[test]
fn comics_category() {
    assert_eq!(
        WorkType::from_category(Some("comics"), None),
        WorkType::Comic
    );
    assert_eq!(
        WorkType::from_category(Some("Comics"), None),
        WorkType::Comic
    );
}

#[test]
fn bd_category() {
    assert_eq!(WorkType::from_category(Some("bd"), None), WorkType::Bd);
    assert_eq!(
        WorkType::from_category(Some("jeunesse"), None),
        WorkType::Bd
    );
    assert_eq!(WorkType::from_category(Some("other"), None), WorkType::Bd);
}

#[test]
fn no_category_falls_back_to_type_field() {
    assert_eq!(
        WorkType::from_category(None, Some("manga series")),
        WorkType::Manga
    );
    assert_eq!(
        WorkType::from_category(None, Some("American Comic")),
        WorkType::Comic
    );
    assert_eq!(WorkType::from_category(None, Some("album")), WorkType::Bd);
    assert_eq!(WorkType::from_category(None, None), WorkType::Bd);
}

#[test]
fn category_takes_precedence_over_type_field() {
    // Even if type says "manga", category "comics" wins
    assert_eq!(
        WorkType::from_category(Some("comics"), Some("manga")),
        WorkType::Comic
    );
}

#[test]
fn display_roundtrip() {
    assert_eq!(WorkType::Bd.to_string(), "bd");
    assert_eq!(WorkType::Manga.to_string(), "manga");
    assert_eq!(WorkType::Comic.to_string(), "comic");
}
