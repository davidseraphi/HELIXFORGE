//! GenBank flat-file and FASTA parsing for the SynthBio registry.
//!
//! Real-world tolerant: multi-record files, CRLF/mixed endings, wrapped
//! qualifiers, unknown feature keys (preserved as misc, never dropped), and
//! the location operators found in actual exports — simple ranges, 5'/3'
//! partials, complement(), join(), single bases, and between-base (^)
//! positions. Malformed records produce per-record errors with line numbers;
//! the valid remainder still imports.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureLoc {
    /// 1-based inclusive start (smallest base position in the location).
    pub start: usize,
    /// 1-based inclusive end (largest base position in the location).
    pub end: usize,
    /// -1 for complement locations, +1 otherwise.
    pub strand: i8,
}

#[derive(Debug, Clone)]
pub struct ParsedFeature {
    pub key: String,
    pub loc: FeatureLoc,
    pub product: String,
    pub gene: String,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct ParsedRecord {
    /// LOCUS name or FASTA header first token.
    pub name: String,
    /// DEFINITION or the full FASTA header.
    pub definition: String,
    /// ACCESSION.VERSION when present.
    pub accession: String,
    /// `linear` or `circular`.
    pub topology: String,
    /// `dna`, `rna`, or `protein`.
    pub alphabet: String,
    /// Normalized uppercase sequence, whitespace stripped.
    pub sequence: String,
    pub features: Vec<ParsedFeature>,
    /// 1-based line where the record starts in the source file.
    pub source_line: usize,
}

#[derive(Debug, Clone)]
pub struct RecordError {
    pub record: String,
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct ParseOutcome {
    pub records: Vec<ParsedRecord>,
    pub errors: Vec<RecordError>,
}

/// Parse an import body. `format_hint` is `auto`, `genbank`, or `fasta`.
pub fn parse_import(format_hint: &str, content: &str) -> ParseOutcome {
    let format = if format_hint == "auto" {
        sniff_format(content)
    } else {
        format_hint.to_string()
    };
    match format.as_str() {
        "fasta" => parse_fasta(content),
        _ => parse_genbank(content),
    }
}

fn sniff_format(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('>') {
            return "fasta".into();
        }
        if t.to_uppercase().starts_with("LOCUS") {
            return "genbank".into();
        }
        return "genbank".into();
    }
    "genbank".into()
}

// ——— FASTA ———

fn parse_fasta(content: &str) -> ParseOutcome {
    let mut out = ParseOutcome::default();
    let mut current: Option<(String, String, usize, String)> = None; // header, name, line, seq

    let flush = |cur: Option<(String, String, usize, String)>, out: &mut ParseOutcome| {
        if let Some((header, name, line, seq)) = cur {
            let sequence: String = seq
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .map(|c| c.to_ascii_uppercase())
                .collect();
            if sequence.is_empty() {
                out.errors.push(RecordError {
                    record: name,
                    line,
                    reason: "record has no sequence".into(),
                });
                return;
            }
            let alphabet = alphabet_from_sequence(&sequence);
            out.records.push(ParsedRecord {
                definition: header,
                name,
                accession: String::new(),
                topology: "linear".into(),
                alphabet,
                sequence,
                features: Vec::new(),
                source_line: line,
            });
        }
    };

    for (idx, line) in content.lines().enumerate() {
        let lineno = idx + 1;
        if let Some(rest) = line.strip_prefix('>') {
            flush(current.take(), &mut out);
            let header = rest.trim().to_string();
            let name = header
                .split_whitespace()
                .next()
                .unwrap_or("unnamed")
                .to_string();
            current = Some((header, name, lineno, String::new()));
        } else if let Some((_, _, _, ref mut seq)) = current {
            seq.push_str(line.trim());
        } else if !line.trim().is_empty() {
            out.errors.push(RecordError {
                record: "<file>".into(),
                line: lineno,
                reason: "content before first FASTA header".into(),
            });
        }
    }
    flush(current.take(), &mut out);
    out
}

fn alphabet_from_sequence(seq: &str) -> String {
    let nucleic = seq.chars().all(|c| "ACGTURYSWKMBDHVN".contains(c));
    if !nucleic {
        return "protein".into();
    }
    if seq.contains('U') {
        "rna".into()
    } else {
        "dna".into()
    }
}

// ——— GenBank ———

fn parse_genbank(content: &str) -> ParseOutcome {
    let mut out = ParseOutcome::default();
    let mut start = 0usize;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0usize;
    while i <= lines.len() {
        let record_end = if i == lines.len() || lines[i].trim() == "//" {
            i
        } else {
            i += 1;
            continue;
        };
        if record_end > start {
            let slice = &lines[start..record_end];
            match parse_genbank_record(slice, start + 1) {
                Ok((rec, errs)) => {
                    out.records.push(rec);
                    out.errors.extend(errs);
                }
                Err(e) => out.errors.push(e),
            }
        }
        start = i + 1;
        i += 1;
    }
    if out.records.is_empty() && out.errors.is_empty() {
        out.errors.push(RecordError {
            record: "<file>".into(),
            line: 1,
            reason: "no GenBank records found".into(),
        });
    }
    out
}

fn parse_genbank_record(
    lines: &[&str],
    source_line: usize,
) -> Result<(ParsedRecord, Vec<RecordError>), RecordError> {
    let locus_line = lines.first().copied().unwrap_or("").trim_end().to_string();
    if !locus_line.to_uppercase().starts_with("LOCUS") {
        return Err(RecordError {
            record: "<unknown>".into(),
            line: source_line,
            reason: "record does not start with LOCUS".into(),
        });
    }
    let parts: Vec<&str> = locus_line.split_whitespace().collect();
    let name = parts.get(1).unwrap_or(&"unnamed").to_string();
    let declared_len: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let topology = if locus_line.to_lowercase().contains("circular") {
        "circular"
    } else {
        "linear"
    }
    .to_string();
    let locus_lower = locus_line.to_lowercase();
    let alphabet = if locus_lower.contains("rna") {
        "rna"
    } else if locus_lower.contains("protein") || locus_lower.contains(" aa") {
        "protein"
    } else {
        "dna"
    }
    .to_string();

    let mut definition = String::new();
    let mut accession = String::new();
    let mut features: Vec<ParsedFeature> = Vec::new();
    let mut sequence = String::new();

    #[derive(PartialEq)]
    enum Section {
        Header,
        Features,
        Origin,
    }
    let mut section = Section::Header;

    let mut cur: Option<(String, String, String, String, String)> = None; // key, loc, product, gene, note
    let mut cur_qual: Option<String> = None;
    let mut rec_errors: Vec<RecordError> = Vec::new();

    {
        let name_ref = &name;
        let mut flush_feature =
            |cur: &mut Option<(String, String, String, String, String)>,
             features: &mut Vec<ParsedFeature>| {
                if let Some((key, loc_raw, product, gene, note)) = cur.take() {
                    match parse_location(&loc_raw) {
                        Ok(loc) => features.push(ParsedFeature {
                            key,
                            loc,
                            product,
                            gene,
                            note,
                        }),
                        Err(reason) => rec_errors.push(RecordError {
                            record: name_ref.clone(),
                            line: source_line,
                            reason: format!("feature `{key}` location `{loc_raw}`: {reason}"),
                        }),
                    }
                }
            };

        for raw in lines.iter().skip(1) {
            let line = raw.trim_end();
            let upper = line.to_uppercase();
            if upper.starts_with("FEATURES") {
                section = Section::Features;
                continue;
            }
            if upper.starts_with("ORIGIN") {
                flush_feature(&mut cur, &mut features);
                section = Section::Origin;
                continue;
            }
            match section {
                Section::Header => {
                    if upper.starts_with("DEFINITION") {
                        definition = line
                            .split_once(|c: char| c.is_whitespace())
                            .map(|x| x.1)
                            .unwrap_or("")
                            .trim()
                            .to_string();
                    } else if upper.starts_with("ACCESSION") {
                        accession = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    } else if upper.starts_with("VERSION") {
                        let v = line.split_whitespace().nth(1).unwrap_or("");
                        if !v.is_empty() {
                            accession = v.to_string();
                        }
                    } else if line.starts_with(' ')
                        && !definition.is_empty()
                        && accession.is_empty()
                    {
                        definition.push(' ');
                        definition.push_str(line.trim());
                    }
                }
                Section::Features => {
                    let is_feature_line =
                        line.len() > 21 && !line.starts_with("                     ");
                    if is_feature_line {
                        flush_feature(&mut cur, &mut features);
                        let key = line[5..21.min(line.len())].trim().to_string();
                        let loc = line[21.min(line.len())..].trim().to_string();
                        cur = Some((key, loc, String::new(), String::new(), String::new()));
                        cur_qual = None;
                    } else if let Some((
                        _,
                        ref mut loc,
                        ref mut product,
                        ref mut gene,
                        ref mut note,
                    )) = cur
                    {
                        let t = line.trim();
                        if let Some(q) = t.strip_prefix('/') {
                            let (qk, qv) = match q.split_once('=') {
                                Some((k, v)) => (k.to_string(), v.trim_matches('"').to_string()),
                                None => (q.to_string(), String::new()),
                            };
                            match qk.as_str() {
                                "product" => *product = qv,
                                "gene" => *gene = qv,
                                "note" => *note = qv,
                                _ => {}
                            }
                            cur_qual = Some(qk);
                        } else if cur_qual.is_none() {
                            // location continuation line
                            loc.push_str(t);
                        } else {
                            // qualifier continuation line
                            let cont = t.trim_matches('"').to_string();
                            match cur_qual.as_deref() {
                                Some("product") => {
                                    product.push(' ');
                                    product.push_str(&cont);
                                }
                                Some("note") => {
                                    note.push(' ');
                                    note.push_str(&cont);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Section::Origin => {
                    for c in line.chars() {
                        if c.is_ascii_alphabetic() {
                            sequence.push(c.to_ascii_uppercase());
                        }
                    }
                }
            }
        }
        flush_feature(&mut cur, &mut features);
    }

    if sequence.is_empty() {
        return Err(RecordError {
            record: name,
            line: source_line,
            reason: "record has no ORIGIN sequence".into(),
        });
    }
    if declared_len > 0 && declared_len != sequence.len() {
        return Err(RecordError {
            record: name,
            line: source_line,
            reason: format!(
                "LOCUS declares {declared_len} bp but ORIGIN has {}",
                sequence.len()
            ),
        });
    }

    Ok((
        ParsedRecord {
            name,
            definition,
            accession,
            topology,
            alphabet,
            sequence,
            features,
            source_line,
        },
        rec_errors,
    ))
}

/// Parse a GenBank location string into a bounding range + strand.
/// Handles: `687..3158`, `<1..>5028`, `123`, `123^124`,
/// `complement(...)`, `join(...)`, and nested `complement(join(...))`.
pub fn parse_location(raw: &str) -> Result<FeatureLoc, String> {
    let mut s = raw.trim().to_string();
    let mut strand: i8 = 1;
    // Unwrap complement( ... ) (possibly nested inside join and vice versa).
    loop {
        let inner = s.trim();
        if let Some(rest) = inner
            .strip_prefix("complement(")
            .and_then(|r| r.strip_suffix(')'))
        {
            strand = -strand;
            s = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = inner
            .strip_prefix("join(")
            .and_then(|r| r.strip_suffix(')'))
        {
            s = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = inner
            .strip_prefix("order(")
            .and_then(|r| r.strip_suffix(')'))
        {
            s = rest.trim().to_string();
            continue;
        }
        break;
    }

    let mut starts: Vec<usize> = Vec::new();
    let mut ends: Vec<usize> = Vec::new();
    for part in s.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        // strip remote-ref prefix like `J00194.1:`
        let p = p.rsplit(':').next().unwrap_or(p);
        if let Some((a, b)) = p.split_once("..") {
            let a = parse_base(a)?;
            let b = parse_base(b)?;
            starts.push(a);
            ends.push(b);
        } else if let Some((a, b)) = p.split_once('^') {
            starts.push(parse_base(a)?);
            ends.push(parse_base(b)?);
        } else {
            let b = parse_base(p)?;
            starts.push(b);
            ends.push(b);
        }
    }
    if starts.is_empty() {
        return Err(format!("no parsable location in `{raw}`"));
    }
    Ok(FeatureLoc {
        start: *starts.iter().min().unwrap(),
        end: *ends.iter().max().unwrap(),
        strand,
    })
}

fn parse_base(s: &str) -> Result<usize, String> {
    s.trim()
        .trim_start_matches(['<', '>'])
        .parse::<usize>()
        .map_err(|e| format!("bad base position `{s}`: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: &str = r#"LOCUS       pDEMO-001              120 bp    DNA     circular SYN 19-JUL-2026
DEFINITION  Demo plasmid with one CDS and a promoter.
ACCESSION   pDEMO-001
VERSION     pDEMO-001.1
FEATURES             Location/Qualifiers
     source          1..1200
     promoter        complement(10..80)
                     /gene="prA"
     CDS             join(100..300,500..900)
                     /codon_start=1
                     /product="demo enzyme"
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
//
LOCUS       pSECOND                120 bp    DNA     linear   SYN 19-JUL-2026
DEFINITION  Second record.
ACCESSION   pSECOND
FEATURES             Location/Qualifiers
     source          1..120
     misc_feature    5^6
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
//
"#;

    #[test]
    fn parses_multi_record_genbank() {
        let out = parse_import("genbank", GB);
        assert_eq!(out.errors.len(), 0, "errors: {:?}", out.errors);
        assert_eq!(out.records.len(), 2);
        let r = &out.records[0];
        assert_eq!(r.name, "pDEMO-001");
        assert_eq!(r.topology, "circular");
        assert_eq!(r.alphabet, "dna");
        assert_eq!(r.sequence.len(), 120);
        assert_eq!(r.features.len(), 3);
        assert_eq!(r.features[1].loc.strand, -1);
        assert_eq!(r.features[1].loc.start, 10);
        assert_eq!(r.features[2].loc.start, 100);
        assert_eq!(r.features[2].loc.end, 900);
        assert_eq!(r.features[2].product, "demo enzyme");
        assert_eq!(out.records[1].features[1].loc.start, 5);
        assert_eq!(out.records[1].features[1].loc.end, 6);
    }

    #[test]
    fn rejects_length_mismatch_with_line_number() {
        let bad = GB.replace(
            "pDEMO-001              120 bp",
            "pDEMO-001              999 bp",
        );
        let out = parse_import("genbank", &bad);
        assert_eq!(out.records.len(), 1);
        assert_eq!(out.errors.len(), 1);
        assert!(out.errors[0].reason.contains("999"));
        assert_eq!(out.errors[0].line, 1);
    }

    #[test]
    fn parses_fasta_multi() {
        let fa = ">seq1 first record\nACGTNNN\nacgt\n>seq2 protein?\nMKVLGFDXW\n";
        let out = parse_import("auto", fa);
        assert_eq!(out.records.len(), 2);
        assert_eq!(out.records[0].alphabet, "dna");
        assert_eq!(out.records[1].alphabet, "protein");
        assert_eq!(out.records[0].sequence.len(), 11);
    }

    #[test]
    fn location_operators() {
        let l = parse_location("complement(3300..4037)").unwrap();
        assert_eq!(
            l,
            FeatureLoc {
                start: 3300,
                end: 4037,
                strand: -1
            }
        );
        let l = parse_location("join(1..100,200..300)").unwrap();
        assert_eq!(
            l,
            FeatureLoc {
                start: 1,
                end: 300,
                strand: 1
            }
        );
        let l = parse_location("complement(join(10..20,40..50))").unwrap();
        assert_eq!(
            l,
            FeatureLoc {
                start: 10,
                end: 50,
                strand: -1
            }
        );
        let l = parse_location("<1..>900").unwrap();
        assert_eq!(
            l,
            FeatureLoc {
                start: 1,
                end: 900,
                strand: 1
            }
        );
        assert!(parse_location("nonsense").is_err());
    }
}
