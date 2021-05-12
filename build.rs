use anyhow::Context;
use std::io::Write;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let s = include_str!("events.txt");

    let mut f =
        std::fs::File::create(Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("events.rs"))
            .expect("Failed to create events.js");

    writeln!(
        f,
        "pub(crate) fn events() -> impl IntoIterator<Item=(&'static str, Event)> {{"
    )?;

    writeln!(
        f,
        r#"    let mut events = ::std::collections::BTreeMap::new();"#
    )?;

    for (i, line) in s.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (summary, datespec) = line.rsplit_once(": ").ok_or_else(|| {
            anyhow::anyhow!("Didn't find delimiter `: ` in line {} `{}`", i, line)
        })?;

        let datespec = parse_datespec(datespec)
            .with_context(|| format!("Failed to parse datespec `{}`", datespec))?;
        writeln!(
            f,
            r####"    events.insert(r###"{}"###, {});"####,
            summary, datespec
        )?;
    }

    writeln!(f, "    events")?;
    writeln!(f, "}}")?;

    Ok(())
}

fn parse_datespec(datespec: &str) -> anyhow::Result<String> {
    let parts: Vec<_> = datespec.split(',').collect();
    match parts.as_slice() {
        ["FixedDate", month, day_of_month] => {
            let parsed_day_of_month: u32 = day_of_month
                .trim_start_matches('0')
                .parse()
                .with_context(|| format!("Failed to parse day of month `{}`", day_of_month))?;
            if parsed_day_of_month == 0 || parsed_day_of_month > 31 {
                anyhow::bail!(
                    "Invalid day of month `{}` should be in range 1..=31",
                    parsed_day_of_month
                );
            }
            Ok(format!(
                "Event::FixedDate(Month::{}, {})",
                month, parsed_day_of_month
            ))
        }
        ["FixedDayOfMonth", month, weekday, week_in_month] => Ok(format!(
            "Event::FixedDayOfMonth(Month::{}, Weekday::{}, WeekInMonth::{})",
            month,
            &weekday[0..=2],
            week_in_month
        )),
        _ => {
            anyhow::bail!("Failed to parse datespec: `{}`", datespec)
        }
    }
}
