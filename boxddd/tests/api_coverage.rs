use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const FIXTURE: &str = include_str!("fixtures/api_coverage_symbols.txt");

#[derive(Debug, Clone)]
struct CoverageEntry {
    symbol: String,
    status: CoverageStatus,
    area: String,
    header: String,
    note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CoverageStatus {
    Safe,
    Raw,
    Omitted,
    Deferred,
}

impl CoverageStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "safe" => Some(Self::Safe),
            "raw" => Some(Self::Raw),
            "omitted" => Some(Self::Omitted),
            "deferred" => Some(Self::Deferred),
            _ => None,
        }
    }
}

#[test]
fn coverage_fixture_matches_vendored_public_api() {
    let header_symbols = extract_header_symbols();
    let entries = parse_fixture(FIXTURE).expect("coverage fixture is well formed");

    assert!(
        header_symbols.len() > 500,
        "expected a broad Box3D public API surface, got {} symbols",
        header_symbols.len()
    );

    let classified: BTreeMap<_, _> = entries
        .iter()
        .map(|entry| (entry.symbol.as_str(), entry))
        .collect();

    let missing: Vec<_> = header_symbols
        .keys()
        .filter(|symbol| !classified.contains_key(symbol.as_str()))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "unclassified B3_API symbols: {missing:#?}"
    );

    let unknown: Vec<_> = classified
        .keys()
        .filter(|symbol| !header_symbols.contains_key(**symbol))
        .copied()
        .collect();
    assert!(
        unknown.is_empty(),
        "fixture contains symbols not found in vendored headers: {unknown:#?}"
    );

    for entry in &entries {
        let actual_header = header_symbols
            .get(entry.symbol.as_str())
            .expect("checked by unknown-symbol assertion");
        assert_eq!(
            actual_header, &entry.header,
            "{} should be classified under {}",
            entry.symbol, actual_header
        );
        assert!(
            !entry.area.trim().is_empty() && !entry.note.trim().is_empty(),
            "{} must include area and note context",
            entry.symbol
        );
    }
}

#[test]
fn coverage_fixture_has_policy_buckets_and_high_priority_symbols() {
    let entries = parse_fixture(FIXTURE).expect("coverage fixture is well formed");
    let by_symbol: BTreeMap<_, _> = entries
        .iter()
        .map(|entry| (entry.symbol.as_str(), entry))
        .collect();
    let statuses: BTreeSet<_> = entries.iter().map(|entry| entry.status).collect();

    assert!(statuses.contains(&CoverageStatus::Safe));
    assert!(statuses.contains(&CoverageStatus::Raw));
    assert!(statuses.contains(&CoverageStatus::Omitted));
    assert!(statuses.contains(&CoverageStatus::Deferred));

    assert_eq!(by_symbol["b3World_Step"].status, CoverageStatus::Safe);
    assert_eq!(by_symbol["b3SetAllocator"].status, CoverageStatus::Raw);
    assert_eq!(by_symbol["b3GetWorldCount"].status, CoverageStatus::Omitted);

    for symbol in [
        "b3Body_CastRay",
        "b3Shape_RayCast",
        "b3World_Explode",
        "b3DynamicTree_Create",
        "b3ShapeDistance",
    ] {
        assert_eq!(
            by_symbol[symbol].status,
            CoverageStatus::Deferred,
            "{symbol} should remain visible until its implementation unit lands"
        );
    }
}

#[test]
fn fixture_parser_rejects_duplicate_symbols_and_bad_statuses() {
    let duplicate = "\
b3World_Step|safe|world|box3d.h|covered
b3World_Step|safe|world|box3d.h|duplicate
";
    assert!(parse_fixture(duplicate).is_err());

    let bad_status = "b3World_Step|maybe|world|box3d.h|bad status\n";
    assert!(parse_fixture(bad_status).is_err());
}

fn parse_fixture(input: &str) -> Result<Vec<CoverageEntry>, String> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();

    for (index, line) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<_> = line.splitn(5, '|').collect();
        if fields.len() != 5 {
            return Err(format!(
                "line {line_number}: expected 5 pipe-separated fields"
            ));
        }

        let symbol = fields[0].trim();
        if !symbol.starts_with("b3") {
            return Err(format!("line {line_number}: symbol must start with b3"));
        }
        if !seen.insert(symbol.to_owned()) {
            return Err(format!("line {line_number}: duplicate symbol {symbol}"));
        }

        let status = CoverageStatus::parse(fields[1].trim())
            .ok_or_else(|| format!("line {line_number}: invalid status {}", fields[1]))?;

        entries.push(CoverageEntry {
            symbol: symbol.to_owned(),
            status,
            area: fields[2].trim().to_owned(),
            header: fields[3].trim().to_owned(),
            note: fields[4].trim().to_owned(),
        });
    }

    if entries.is_empty() {
        return Err("coverage fixture is empty".to_owned());
    }

    Ok(entries)
}

fn extract_header_symbols() -> BTreeMap<String, String> {
    let header_dir = header_dir();
    let mut symbols = BTreeMap::new();

    for entry in fs::read_dir(&header_dir).expect("Box3D header directory exists") {
        let path = entry.expect("header directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("h") {
            continue;
        }

        let header = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("header has utf-8 name")
            .to_owned();
        let contents = fs::read_to_string(&path).expect("header can be read");
        for symbol in extract_symbols_from_header(&contents) {
            symbols.entry(symbol).or_insert_with(|| header.clone());
        }
    }

    symbols
}

fn header_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../boxddd-sys/third-party/box3d/include/box3d")
}

fn extract_symbols_from_header(contents: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut rest = contents;

    while let Some(index) = rest.find("B3_API") {
        rest = &rest[index + "B3_API".len()..];
        let Some(end) = rest.find(';') else {
            break;
        };
        let declaration = &rest[..end];
        rest = &rest[end + 1..];

        let Some(open_paren) = declaration.find('(') else {
            continue;
        };
        let before_paren = &declaration[..open_paren];
        let Some(symbol) = last_identifier(before_paren) else {
            continue;
        };
        if symbol.starts_with("b3") {
            symbols.push(symbol.to_owned());
        }
    }

    symbols
}

fn last_identifier(input: &str) -> Option<&str> {
    let end = input
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(|(index, ch)| index + ch.len_utf8())?;

    let start = input[..end]
        .char_indices()
        .rev()
        .find(|(_, ch)| !(ch.is_ascii_alphanumeric() || *ch == '_'))
        .map_or(0, |(index, ch)| index + ch.len_utf8());

    Some(&input[start..end])
}
