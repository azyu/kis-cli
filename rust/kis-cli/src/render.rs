use unicode_width::UnicodeWidthStr;

pub fn render_pairs(rows: &[(&str, String)]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let width = rows
        .iter()
        .map(|(label, _)| display_width(label))
        .max()
        .unwrap_or(0);
    let mut output = String::new();

    for (index, (label, value)) in rows.iter().enumerate() {
        output.push_str(label);
        let label_width = display_width(label);
        if width > label_width {
            output.push_str(&" ".repeat(width - label_width));
        }
        output.push_str("  ");
        output.push_str(value);
        if index + 1 != rows.len() {
            output.push('\n');
        }
    }

    output
}

pub fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    let mut widths: Vec<usize> = headers.iter().map(|header| display_width(header)).collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate().take(widths.len()) {
            widths[index] = widths[index].max(display_width(cell));
        }
    }

    let mut lines = Vec::with_capacity(rows.len() + 2);
    lines.push(join_row(
        headers.iter().copied().map(str::to_owned).collect(),
        &widths,
    ));
    lines.push(join_row(
        widths.iter().map(|width| "-".repeat(*width)).collect(),
        &widths,
    ));
    lines.extend(rows.iter().map(|row| join_row(row.clone(), &widths)));
    lines.join("\n")
}

fn join_row(columns: Vec<String>, widths: &[usize]) -> String {
    columns
        .into_iter()
        .enumerate()
        .map(|(index, column)| {
            let width = widths.get(index).copied().unwrap_or_default();
            let padding = width.saturating_sub(display_width(&column));
            format!("{column}{}", " ".repeat(padding))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

#[cfg(test)]
mod tests {
    use super::{render_pairs, render_table};

    #[test]
    fn renders_pairs_with_alignment() {
        let output = render_pairs(&[
            ("종목명", "삼성전자".to_string()),
            ("현재가", "70000".to_string()),
        ]);

        assert_eq!(output, "종목명  삼성전자\n현재가  70000");
    }

    #[test]
    fn aligns_pairs_using_display_width_for_korean_labels() {
        let output = render_pairs(&[
            ("계좌번호", "12345678".to_string()),
            ("환경", "virtual".to_string()),
        ]);

        assert_eq!(output, "계좌번호  12345678\n환경      virtual");
    }

    #[test]
    fn renders_table_with_header_separator() {
        let output = render_table(
            &["날짜", "종가"],
            &[
                vec!["20260306".to_string(), "70000".to_string()],
                vec!["20260305".to_string(), "69500".to_string()],
            ],
        );

        let mut lines = output.lines();
        let header = lines.next().unwrap();
        let separator = lines.next().unwrap();

        assert!(header.contains("날짜"));
        assert!(header.contains("종가"));
        assert!(separator.contains("--"));
        assert!(output.contains("20260306  70000"));
    }

    #[test]
    fn aligns_tables_using_display_width_for_korean_cells() {
        let output = render_table(
            &["상태", "값"],
            &[
                vec!["가상".to_string(), "1".to_string()],
                vec!["실전환경".to_string(), "2".to_string()],
            ],
        );

        assert_eq!(
            output,
            "상태      값\n--------  --\n가상      1 \n실전환경  2 "
        );
    }
}
