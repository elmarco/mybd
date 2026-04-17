use bnfbd::Client;

fn minimal_sru_response(ark: &str, title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<srw:searchRetrieveResponse xmlns:srw="http://www.loc.gov/zing/srw/">
  <srw:numberOfRecords>1</srw:numberOfRecords>
  <srw:records>
    <srw:record>
      <srw:recordData>
        <mxc:record xmlns:mxc="info:lc/xmlns/marcxchange-v2" id="{ark}" type="Bibliographic">
          <mxc:datafield tag="200" ind1="1" ind2=" ">
            <mxc:subfield code="a">{title}</mxc:subfield>
          </mxc:datafield>
        </mxc:record>
      </srw:recordData>
    </srw:record>
  </srw:records>
</srw:searchRetrieveResponse>"#
    )
}

fn empty_sru_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<srw:searchRetrieveResponse xmlns:srw="http://www.loc.gov/zing/srw/">
  <srw:numberOfRecords>0</srw:numberOfRecords>
  <srw:records></srw:records>
</srw:searchRetrieveResponse>"#
        .to_string()
}

#[tokio::test]
async fn search_by_isbn_returns_records() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("version".into(), "1.2".into()),
            mockito::Matcher::UrlEncoded("operation".into(), "searchRetrieve".into()),
            mockito::Matcher::UrlEncoded("query".into(), r#"bib.isbn all "9782012101333""#.into()),
            mockito::Matcher::UrlEncoded("recordSchema".into(), "unimarcXchange".into()),
            mockito::Matcher::UrlEncoded("maximumRecords".into(), "10".into()),
            mockito::Matcher::UrlEncoded("startRecord".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(minimal_sru_response(
            "ark:/12148/cb43404620t",
            "Astérix le Gaulois",
        ))
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client.search_by_isbn("9782012101333").await.unwrap();

    assert_eq!(results.records.len(), 1);
    assert_eq!(results.records[0].ark, "ark:/12148/cb43404620t");
    assert_eq!(results.records[0].title, "Astérix le Gaulois");
    assert_eq!(results.total, 1);

    mock.assert_async().await;
}

#[tokio::test]
async fn search_by_title_sends_correct_cql() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("version".into(), "1.2".into()),
            mockito::Matcher::UrlEncoded("operation".into(), "searchRetrieve".into()),
            mockito::Matcher::UrlEncoded(
                "query".into(),
                r#"bib.title all "astérix" and bib.doctype all "a""#.into(),
            ),
            mockito::Matcher::UrlEncoded("recordSchema".into(), "unimarcXchange".into()),
            mockito::Matcher::UrlEncoded("maximumRecords".into(), "20".into()),
            mockito::Matcher::UrlEncoded("startRecord".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(minimal_sru_response(
            "ark:/12148/cb43404620t",
            "Astérix le Gaulois",
        ))
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client.search_by_title("astérix", 20).await.unwrap();

    assert_eq!(results.records.len(), 1);

    mock.assert_async().await;
}

#[tokio::test]
async fn search_by_author_sends_correct_cql() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("version".into(), "1.2".into()),
            mockito::Matcher::UrlEncoded("operation".into(), "searchRetrieve".into()),
            mockito::Matcher::UrlEncoded(
                "query".into(),
                r#"bib.author all "goscinny" and bib.doctype all "a""#.into(),
            ),
            mockito::Matcher::UrlEncoded("recordSchema".into(), "unimarcXchange".into()),
            mockito::Matcher::UrlEncoded("maximumRecords".into(), "10".into()),
            mockito::Matcher::UrlEncoded("startRecord".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(minimal_sru_response(
            "ark:/12148/cb43404620t",
            "Astérix le Gaulois",
        ))
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client.search_by_author("goscinny", 10).await.unwrap();

    assert_eq!(results.records.len(), 1);

    mock.assert_async().await;
}

#[tokio::test]
async fn search_by_series_sends_correct_cql() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("version".into(), "1.2".into()),
            mockito::Matcher::UrlEncoded("operation".into(), "searchRetrieve".into()),
            mockito::Matcher::UrlEncoded(
                "query".into(),
                r#"bib.title all "une aventure d'astérix" and bib.doctype all "a""#.into(),
            ),
            mockito::Matcher::UrlEncoded("recordSchema".into(), "unimarcXchange".into()),
            mockito::Matcher::UrlEncoded("maximumRecords".into(), "50".into()),
            mockito::Matcher::UrlEncoded("startRecord".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(empty_sru_response())
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client
        .search_by_series("une aventure d'astérix", 10)
        .await
        .unwrap();

    assert!(results.records.is_empty());

    mock.assert_async().await;
}

#[tokio::test]
async fn search_empty_results() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(empty_sru_response())
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client.search_by_isbn("0000000000000").await.unwrap();

    assert!(results.records.is_empty());
}

#[tokio::test]
async fn search_http_error() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let result = client.search_by_isbn("9782012101333").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn get_by_ark_returns_record() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("version".into(), "1.2".into()),
            mockito::Matcher::UrlEncoded("operation".into(), "searchRetrieve".into()),
            mockito::Matcher::UrlEncoded(
                "query".into(),
                r#"bib.persistentid all "ark:/12148/cb43404620t""#.into(),
            ),
            mockito::Matcher::UrlEncoded("recordSchema".into(), "unimarcXchange".into()),
            mockito::Matcher::UrlEncoded("maximumRecords".into(), "1".into()),
            mockito::Matcher::UrlEncoded("startRecord".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(minimal_sru_response(
            "ark:/12148/cb43404620t",
            "Astérix le Gaulois",
        ))
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let record = client.get_by_ark("ark:/12148/cb43404620t").await.unwrap();

    assert!(record.is_some());
    assert_eq!(record.unwrap().title, "Astérix le Gaulois");

    mock.assert_async().await;
}

#[tokio::test]
async fn get_by_ark_returns_none_when_not_found() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(empty_sru_response())
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let record = client.get_by_ark("ark:/12148/cb00000000x").await.unwrap();

    assert!(record.is_none());
}

#[tokio::test]
async fn search_has_more() {
    let mut server = mockito::Server::new_async().await;
    let response = r#"<?xml version="1.0" encoding="UTF-8"?>
<srw:searchRetrieveResponse xmlns:srw="http://www.loc.gov/zing/srw/">
  <srw:numberOfRecords>100</srw:numberOfRecords>
  <srw:records>
    <srw:record>
      <srw:recordData>
        <mxc:record xmlns:mxc="info:lc/xmlns/marcxchange-v2" id="ark:/12148/cb00000001" type="Bibliographic">
          <mxc:datafield tag="200" ind1="1" ind2=" ">
            <mxc:subfield code="a">Premier</mxc:subfield>
          </mxc:datafield>
        </mxc:record>
      </srw:recordData>
    </srw:record>
  </srw:records>
</srw:searchRetrieveResponse>"#;

    let _mock = server
        .mock("GET", "/api/SRU")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(response)
        .create_async()
        .await;

    let client = Client::with_base_url(&server.url());
    let results = client.search_by_title("test", 1).await.unwrap();

    assert_eq!(results.total, 100);
    assert_eq!(results.records.len(), 1);
    assert!(results.has_more());
}
