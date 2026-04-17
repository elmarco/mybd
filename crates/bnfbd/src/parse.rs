use crate::{Error, Record, Result, types::Author};

pub fn parse_records(xml: &str) -> Result<Vec<Record>> {
    let (records, _) = parse_response(xml)?;
    Ok(records)
}

pub(crate) fn parse_response(xml: &str) -> Result<(Vec<Record>, u32)> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| Error::Xml(e.to_string()))?;

    let total = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "numberOfRecords")
        .and_then(|n| n.text())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let records = doc
        .descendants()
        .filter(|n| {
            n.is_element()
                && n.tag_name().name() == "record"
                && n.attribute("type") == Some("Bibliographic")
        })
        .filter_map(|n| parse_record(n))
        .collect();

    Ok((records, total))
}

fn parse_record(node: roxmltree::Node) -> Option<Record> {
    let ark = node.attribute("id")?.to_string();
    let title = subfield(node, "200", "a")?.to_string();

    Some(Record {
        ark,
        title,
        authors: parse_authors(node),
        publisher: subfield(node, "210", "c").map(String::from),
        pub_date: subfield(node, "210", "d").map(String::from),
        pages: subfield(node, "215", "a").map(String::from),
        dimensions: subfield(node, "215", "d").map(String::from),
        isbn: subfield(node, "010", "a").map(String::from),
        ean: subfield(node, "073", "a").map(String::from),
        series: subfield(node, "225", "a").map(String::from),
        volume: subfield(node, "225", "v").and_then(parse_volume),
        language: subfield(node, "101", "a").map(String::from),
    })
}

fn parse_authors(record: roxmltree::Node) -> Vec<Author> {
    record
        .children()
        .filter(|n| {
            n.is_element()
                && n.tag_name().name() == "datafield"
                && matches!(n.attribute("tag"), Some("700" | "701" | "702"))
        })
        .filter_map(|df| {
            let name = df_subfield(df, "a")?.to_string();
            Some(Author {
                name,
                first_name: df_subfield(df, "b").map(String::from),
                dates: df_subfield(df, "f").map(String::from),
                role_code: df_subfield(df, "4").map(String::from),
                bnf_id: df_subfield(df, "3").map(String::from),
                isni: df_subfield(df, "o").map(String::from),
            })
        })
        .collect()
}

fn subfield<'a, 'input: 'a>(
    record: roxmltree::Node<'a, 'input>,
    tag: &str,
    code: &str,
) -> Option<&'a str> {
    let df = record.children().find(|n| {
        n.is_element() && n.tag_name().name() == "datafield" && n.attribute("tag") == Some(tag)
    })?;
    df_subfield(df, code)
}

fn df_subfield<'a, 'input: 'a>(
    datafield: roxmltree::Node<'a, 'input>,
    code: &str,
) -> Option<&'a str> {
    datafield
        .children()
        .find(|n| {
            n.is_element() && n.tag_name().name() == "subfield" && n.attribute("code") == Some(code)
        })
        .and_then(|n| n.text())
}

fn parse_volume(v: &str) -> Option<i32> {
    v.trim().parse::<i32>().ok().or_else(|| {
        let digits: String = v
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        digits.parse().ok()
    })
}
