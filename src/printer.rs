use crate::{ArchivedHgncRecord, Field, OutputType, QueryResult};
use std::error::Error;

fn print_pretty_helper(query: &str, records: &[&ArchivedHgncRecord], fields: &[Field]) {
    println!("Query: {}", query);
    println!("Found {} match(es)\n", records.len());

    for (idx, record) in records.iter().enumerate() {
        if idx > 0 {
            println!("\n{}", "=".repeat(80));
            println!();
        }

        for (field, value) in record.selected_fields(fields) {
            println!("{}: {}", field, value);
        }
    }
}

fn print_delimited_helper(
    query: &str,
    records: &[&ArchivedHgncRecord],
    fields: &[Field],
    delimiter: u8,
) {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(std::io::stdout());

    for record in records {
        let row =
            std::iter::once(query).chain(fields.iter().map(|field| record.field_value(*field)));
        wtr.write_record(row).expect("Failed to write record");
    }

    wtr.flush().expect("Failed to flush writer");
}

fn print_json_helper(
    query: &str,
    records: &[&ArchivedHgncRecord],
    fields: &[Field],
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let records_json: Vec<_> = records
        .iter()
        .map(|record| {
            let mut obj = serde_json::Map::new();
            for (field, value) in record.selected_fields(fields) {
                obj.insert(
                    field.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let output = serde_json::json!({
        "query": query,
        "count": records.len(),
        "records": records_json
    });

    if pretty {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }

    Ok(())
}

fn print_json_error_helper(query: &str, pretty: bool) -> Result<(), Box<dyn Error>> {
    let error_obj = serde_json::json!({
        "error": "not_found",
        "query": query,
        "message": format!("No record found for query: {}", query)
    });
    let json = serde_json::to_string(&error_obj)?;

    if pretty {
        println!("{}", serde_json::to_string_pretty(&error_obj)?);
    } else {
        println!("{}", json);
    }
    eprintln!("No record found for query: {}", query);

    Ok(())
}

/// Print a query result in the specified format
pub fn print_query_result(
    result: &QueryResult,
    fields: &[Field],
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    match (result, output_type) {
        // Found cases
        (QueryResult::Found(query, records), OutputType::Pretty) => {
            print_pretty_helper(query, records, fields);
        }
        (QueryResult::Found(query, records), OutputType::Json) => {
            print_json_helper(query, records, fields, false)?;
        }
        (QueryResult::Found(query, records), OutputType::JsonPretty) => {
            print_json_helper(query, records, fields, true)?;
        }
        (QueryResult::Found(query, records), OutputType::Csv) => {
            print_delimited_helper(query, records, fields, b',');
        }
        (QueryResult::Found(query, records), OutputType::Tsv) => {
            print_delimited_helper(query, records, fields, b'\t');
        }

        // NotFound cases
        (QueryResult::NotFound(query), OutputType::Json) => {
            print_json_error_helper(query, false)?;
        }
        (QueryResult::NotFound(query), OutputType::JsonPretty) => {
            print_json_error_helper(query, true)?;
        }
        (QueryResult::NotFound(query), OutputType::Pretty | OutputType::Csv | OutputType::Tsv) => {
            eprintln!("No record found for query: {}", query);
        }
    }
    Ok(())
}
