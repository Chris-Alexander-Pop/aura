//! RFC 5545 iCalendar text parser and serializer for `.ics` files.
//!
//! Standard `.ics` files are line-oriented text, not XML; `quick-xml` is reserved for
//! future xCal or other XML calendar formats.

use crate::services::calendar::CalendarEvent;
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVevent {
    pub uid: String,
    pub summary: String,
    pub description: String,
    pub dtstart: i64,
    pub dtend: i64,
}

/// Parse iCalendar text into VEVENT components.
pub fn parse_ics(content: &str) -> Result<Vec<ParsedVevent>> {
    let lines = unfold_lines(content);
    let mut events = Vec::new();
    let mut in_event = false;
    let mut props: HashMap<String, String> = HashMap::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "BEGIN:VEVENT" {
            in_event = true;
            props.clear();
            continue;
        }
        if line == "END:VEVENT" {
            if in_event {
                if let Some(ev) = vevent_from_props(&props) {
                    events.push(ev);
                }
            }
            in_event = false;
            props.clear();
            continue;
        }
        if !in_event {
            continue;
        }
        if let Some((key, value)) = split_property(line) {
            props.insert(key, value);
        }
    }

    Ok(events)
}

/// Serialize calendar events as a VCALENDAR document.
pub fn export_ics(events: &[CalendarEvent], calendar_name: &str) -> String {
    let mut out = String::from("BEGIN:VCALENDAR\r\n");
    out.push_str("VERSION:2.0\r\n");
    out.push_str("PRODID:-//Aura AGS//EN\r\n");
    out.push_str(&format!("X-WR-CALNAME:{}\r\n", escape_text(calendar_name)));

    for event in events {
        out.push_str("BEGIN:VEVENT\r\n");
        out.push_str(&format!("UID:{}\r\n", escape_text(&event.id)));
        out.push_str(&format!(
            "DTSTART:{}\r\n",
            format_ics_utc(event.start)
        ));
        out.push_str(&format!("DTEND:{}\r\n", format_ics_utc(event.end)));
        out.push_str(&format!("SUMMARY:{}\r\n", escape_text(&event.title)));
        if !event.description.is_empty() {
            out.push_str(&format!(
                "DESCRIPTION:{}\r\n",
                escape_text(&event.description)
            ));
        }
        out.push_str("END:VEVENT\r\n");
    }

    out.push_str("END:VCALENDAR\r\n");
    out
}

pub fn parsed_to_calendar_event(parsed: &ParsedVevent, calendar_id: Option<String>) -> CalendarEvent {
    CalendarEvent {
        id: parsed.uid.clone(),
        title: parsed.summary.clone(),
        start: parsed.dtstart,
        end: parsed.dtend,
        description: parsed.description.clone(),
        calendar_id,
        reminder_minutes: None,
    }
}

fn vevent_from_props(props: &HashMap<String, String>) -> Option<ParsedVevent> {
    let dtstart = props.get("DTSTART").and_then(|v| parse_ics_datetime(v))?;
    let dtend = props
        .get("DTEND")
        .and_then(|v| parse_ics_datetime(v))
        .unwrap_or(dtstart + 3600);
    let uid = props
        .get("UID")
        .cloned()
        .unwrap_or_else(|| format!("aura-{}-{}", dtstart, dtend));
    let summary = props
        .get("SUMMARY")
        .cloned()
        .unwrap_or_else(|| "Untitled".to_string());
    let description = props.get("DESCRIPTION").cloned().unwrap_or_default();
    Some(ParsedVevent {
        uid,
        summary,
        description,
        dtstart,
        dtend,
    })
}

fn unfold_lines(content: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in content.lines() {
        if (line.starts_with(' ') || line.starts_with('\t')) && !out.is_empty() {
            let cont = line.trim_start_matches([' ', '\t']);
            if let Some(last) = out.last_mut() {
                last.push_str(cont);
            }
        } else {
            out.push(line.to_string());
        }
    }
    out
}

fn split_property(line: &str) -> Option<(String, String)> {
    let (raw_key, value) = line.split_once(':')?;
    let key = raw_key
        .split(';')
        .next()
        .unwrap_or(raw_key)
        .trim()
        .to_ascii_uppercase();
    Some((key, unescape_text(value.trim())))
}

fn parse_ics_datetime(raw: &str) -> Option<i64> {
    let value = raw.trim();
    if value.len() == 8 && value.chars().all(|c| c.is_ascii_digit()) {
        let date = NaiveDate::parse_from_str(value, "%Y%m%d").ok()?;
        return Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0)?).timestamp().into();
    }
    if value.ends_with('Z') && value.len() >= 16 {
        let dt = NaiveDateTime::parse_from_str(value.trim_end_matches('Z'), "%Y%m%dT%H%M%S").ok()?;
        return Some(Utc.from_utc_datetime(&dt).timestamp());
    }
    if value.contains('T') && value.len() >= 15 {
        let dt = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S").ok()?;
        return Some(Utc.from_utc_datetime(&dt).timestamp());
    }
    None
}

fn format_ics_utc(ts: i64) -> String {
    let dt = Utc
        .timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(Utc::now);
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

fn escape_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

fn unescape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Import parsed events into SQLite `calendar_events` namespace.
pub async fn import_events_to_storage(
    parsed: &[ParsedVevent],
    calendar_id: Option<String>,
) -> Result<usize> {
    use crate::utils::storage;

    storage::init().await?;
    let mut count = 0usize;
    for p in parsed {
        let event = parsed_to_calendar_event(p, calendar_id.clone());
        storage::set_kv(
            "calendar_events",
            &event.id,
            &serde_json::to_value(&event)?,
        )
        .await?;
        count += 1;
    }
    Ok(count)
}

/// Load events from storage, optionally filtered by `calendar_id`.
pub async fn load_events_for_export(calendar_id: Option<&str>) -> Result<Vec<CalendarEvent>> {
    use crate::utils::storage;

    storage::init().await?;
    let mut events =
        crate::services::calendar::events_from_storage_values(
            storage::scan_namespace("calendar_events").await?,
        );
    if let Some(cid) = calendar_id {
        events.retain(|e| e.calendar_id.as_deref() == Some(cid));
    }
    Ok(events)
}

pub async fn read_ics_file(path: &std::path::Path) -> Result<Vec<ParsedVevent>> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("read ICS file {}", path.display()))?;
    parse_ics(&content)
}

pub async fn write_ics_file(path: &std::path::Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }
    tokio::fs::write(path, body)
        .await
        .with_context(|| format!("write ICS file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/calendar/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    #[test]
    fn parse_fixture_single_event() {
        let events = parse_ics(&fixture("single_event.ics")).expect("parse");
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev.uid, "fixture-meeting@aura");
        assert_eq!(ev.summary, "Fixture Meeting");
        assert_eq!(ev.description, "From integration fixture");
        assert!(ev.dtstart < ev.dtend);
    }

    #[test]
    fn parse_unfolded_description() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:u1\r\nDTSTART:20260701T120000Z\r\n\
                   DTEND:20260701T130000Z\r\nSUMMARY:T\r\nDESCRIPTION:line one\r\n line two\r\n\
                   END:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = parse_ics(ics).unwrap();
        assert_eq!(events[0].description, "line oneline two");
    }

    #[test]
    fn export_import_round_trip() {
        let original = CalendarEvent {
            id: "roundtrip@aura".into(),
            title: "Round Trip".into(),
            start: 1_718_000_000,
            end: 1_718_003_600,
            description: "notes".into(),
            calendar_id: Some("work".into()),
            reminder_minutes: Some(15),
        };
        let body = export_ics(&[original.clone()], "Work");
        let parsed = parse_ics(&body).expect("re-parse");
        assert_eq!(parsed.len(), 1);
        let back = parsed_to_calendar_event(&parsed[0], original.calendar_id.clone());
        assert_eq!(back.id, original.id);
        assert_eq!(back.title, original.title);
        assert_eq!(back.start, original.start);
        assert_eq!(back.end, original.end);
        assert_eq!(back.description, original.description);
    }

    #[test]
    fn parse_rejects_empty_calendar() {
        let events = parse_ics("BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n").unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn parse_date_only_dtstart() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:d1\r\nDTSTART:20260115\r\n\
                   SUMMARY:All day\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = parse_ics(ics).unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].dtend >= events[0].dtstart);
    }
}
