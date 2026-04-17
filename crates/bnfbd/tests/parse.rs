fn sru_wrap(records: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<srw:searchRetrieveResponse xmlns:srw="http://www.loc.gov/zing/srw/">
  <srw:numberOfRecords>1</srw:numberOfRecords>
  <srw:records>{records}</srw:records>
</srw:searchRetrieveResponse>"#
    )
}

fn make_record(ark: &str, datafields: &str) -> String {
    format!(
        r#"<srw:record xmlns:srw="http://www.loc.gov/zing/srw/">
  <srw:recordData>
    <mxc:record xmlns:mxc="info:lc/xmlns/marcxchange-v2" id="{ark}" type="Bibliographic">
      {datafields}
    </mxc:record>
  </srw:recordData>
</srw:record>"#
    )
}

fn datafield(tag: &str, subfields: &[(&str, &str)]) -> String {
    let subs: String = subfields
        .iter()
        .map(|(code, val)| {
            format!(
                r#"<mxc:subfield xmlns:mxc="info:lc/xmlns/marcxchange-v2" code="{code}">{val}</mxc:subfield>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");
    format!(
        r#"<mxc:datafield xmlns:mxc="info:lc/xmlns/marcxchange-v2" tag="{tag}" ind1=" " ind2=" ">
      {subs}
    </mxc:datafield>"#
    )
}

#[test]
fn parse_full_record() {
    let fields = [
        datafield("010", &[("a", "978-2-01-210235-0")]),
        datafield("073", &[("a", "9782012102350")]),
        datafield("101", &[("a", "fre")]),
        datafield(
            "200",
            &[("a", "Astérix le Gaulois"), ("f", "René Goscinny")],
        ),
        datafield("210", &[("c", "Hachette"), ("d", "2004")]),
        datafield("215", &[("a", "48 p."), ("d", "30 cm")]),
        datafield("225", &[("a", "Les Aventures d'Astérix"), ("v", "1")]),
        datafield(
            "700",
            &[
                ("a", "Goscinny"),
                ("b", "René"),
                ("f", "1926-1977"),
                ("4", "0070"),
            ],
        ),
        datafield(
            "702",
            &[
                ("a", "Uderzo"),
                ("b", "Albert"),
                ("f", "1927-2020"),
                ("4", "0440"),
            ],
        ),
    ]
    .join("\n      ");

    let xml = sru_wrap(&make_record("ark:/12148/cb30000001", &fields));
    let records = bnfbd::parse_records(&xml).unwrap();

    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.ark, "ark:/12148/cb30000001");
    assert_eq!(r.title, "Astérix le Gaulois");
    assert_eq!(r.isbn.as_deref(), Some("978-2-01-210235-0"));
    assert_eq!(r.ean.as_deref(), Some("9782012102350"));
    assert_eq!(r.language.as_deref(), Some("fre"));
    assert_eq!(r.publisher.as_deref(), Some("Hachette"));
    assert_eq!(r.pub_date.as_deref(), Some("2004"));
    assert_eq!(r.pages.as_deref(), Some("48 p."));
    assert_eq!(r.dimensions.as_deref(), Some("30 cm"));
    assert_eq!(r.series.as_deref(), Some("Les Aventures d'Astérix"));
    assert_eq!(r.volume, Some(1));

    assert_eq!(r.authors.len(), 2);
    assert_eq!(r.authors[0].name, "Goscinny");
    assert_eq!(r.authors[0].first_name.as_deref(), Some("René"));
    assert_eq!(r.authors[0].dates.as_deref(), Some("1926-1977"));
    assert_eq!(r.authors[0].role_code.as_deref(), Some("0070"));

    assert_eq!(r.authors[1].name, "Uderzo");
    assert_eq!(r.authors[1].first_name.as_deref(), Some("Albert"));
    assert_eq!(r.authors[1].dates.as_deref(), Some("1927-2020"));
    assert_eq!(r.authors[1].role_code.as_deref(), Some("0440"));
}

#[test]
fn parse_minimal_record() {
    let fields = datafield("200", &[("a", "Titre minimal")]);
    let xml = sru_wrap(&make_record("ark:/12148/cb00000002", &fields));
    let records = bnfbd::parse_records(&xml).unwrap();

    assert_eq!(records.len(), 1);
    let r = &records[0];
    assert_eq!(r.ark, "ark:/12148/cb00000002");
    assert_eq!(r.title, "Titre minimal");
    assert!(r.isbn.is_none());
    assert!(r.ean.is_none());
    assert!(r.language.is_none());
    assert!(r.publisher.is_none());
    assert!(r.pub_date.is_none());
    assert!(r.pages.is_none());
    assert!(r.dimensions.is_none());
    assert!(r.series.is_none());
    assert!(r.volume.is_none());
    assert!(r.authors.is_empty());
}

#[test]
fn parse_multiple_records() {
    let r1 = make_record(
        "ark:/12148/cb00000001",
        &datafield("200", &[("a", "Premier livre")]),
    );
    let r2 = make_record(
        "ark:/12148/cb00000002",
        &datafield("200", &[("a", "Deuxième livre")]),
    );
    let xml = sru_wrap(&format!("{r1}\n{r2}"));
    let records = bnfbd::parse_records(&xml).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].title, "Premier livre");
    assert_eq!(records[1].title, "Deuxième livre");
}

#[test]
fn parse_empty_response() {
    let xml = sru_wrap("");
    let records = bnfbd::parse_records(&xml).unwrap();
    assert!(records.is_empty());
}

#[test]
fn parse_volume_non_numeric_prefix() {
    let fields = [
        datafield("200", &[("a", "Un titre")]),
        datafield("225", &[("a", "Ma série"), ("v", "T.14")]),
    ]
    .join("\n      ");
    let xml = sru_wrap(&make_record("ark:/12148/cb00000003", &fields));
    let records = bnfbd::parse_records(&xml).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].volume, Some(14));
}

#[test]
fn parse_invalid_xml() {
    let result = bnfbd::parse_records("<<<not xml at all>>>");
    assert!(result.is_err());
}
