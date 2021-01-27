use anyhow::{bail, Context, Error, Result};
use chrono::prelude::*;
use std::fs::File;
use std::io::BufRead;

trait OptionExt<'a, T: ?Sized> {
    fn named(self, name: &str) -> Result<&'a T>;
}

impl<'a> OptionExt<'a, str> for Option<&'a str> {
    fn named(self, name: &str) -> Result<&'a str> {
        Ok(self.ok_or_else(|| anyhow::anyhow!("missing field {}", name))?)
    }
}

#[derive(Debug)]
enum GpsRecord {
    User(String),
    Version(String),
    AppVersion(String),
    Device(Vec<String>),
    Coords {
        timestamp: DateTime<FixedOffset>,
        lat: f64,
        lon: f64,
        ele: f64, // meters
        // also has a local timestamp in millis, and string representation of local and UTC times
    },
    Delta {
        millis: u64,
        f2: i64,
        f3: i64,
        ele_change: f64, // meters change since last Coords record
        speed: f64, // meters per second
        heading: f64, // degrees
    },
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("need a file path");
    let file = File::open(path).context("failed to open file")?;
    let mut z = zip::ZipArchive::new(file).context("failed to read zip file")?;
    let (gps_path, acc_path) = {
        let mut gps = None;
        let mut acc = None;
        for path in z.file_names() {
            println!("found file {}", path);
            if path.ends_with(".gps") {
                gps = Some(path);
            } else if path.ends_with(".acc") {
                acc = Some(path);
            } else {
                bail!("unrecognized filename {} in input archive", path);
            }
        }
        if gps.is_none() {
            bail!("missing a .gps file in archive");
        }
        if acc.is_none() {
            bail!("missing a .acc file in archive");
        }
        (gps.unwrap().to_owned(), acc.unwrap().to_owned())
    };
    let gps_file = z.by_name(&gps_path).context("failed to get .gps file from archive")?;
    let mut records = vec![];
    for line in std::io::BufReader::new(gps_file).lines() {
        let line = line.context("read error")?;
        let mut fields = line.split(',');
        let tag = fields.next().ok_or_else(|| Error::msg("missing tag field"))?;
        macro_rules! parse {
            ($name:expr) => {
                fields.next().named($name)?.parse().context(concat!("invalid ", $name))?
            }
        }
        let record = match tag {
            "U" => GpsRecord::User(fields.next().named("username")?.to_owned()),
            "V" => GpsRecord::Version(fields.next().named("version")?.to_owned()),
            "A" => GpsRecord::AppVersion(fields.next().named("app version")?.to_owned()),
            "I" => GpsRecord::Device(fields.map(|s| s.to_owned()).collect()),
            "H" => {
                let utc_timestamp: i64 = parse!("timestamp"); // UTC
                let lat = parse!("latitude");
                let lon = parse!("longitude");
                let ele = parse!("elevation");
                let local_timestamp: i64 = parse!("local timestamp");
                let datetime_utc = fields.next().named("first datetime")?;
                let datetime_local = fields.next().named("second datetime")?;

                let tz_off_secs = (local_timestamp - utc_timestamp) / 1000;
                let tz = chrono::FixedOffset::east(tz_off_secs as i32);
                let timestamp = tz.timestamp(
                    utc_timestamp / 1000,
                    (utc_timestamp % 1000) as u32 * 1_000_000);

                // just make sure they parse correctly
                let _date1 = NaiveDateTime::parse_from_str(datetime_utc, "%Y-%m-%dT%H:%M:%S%.f")
                    .context("invalid first datetime")?;
                let _date2 = NaiveDateTime::parse_from_str(datetime_local, "%Y-%m-%dT%H:%M:%S%.f")
                    .context("invalid second datetime")?;

                GpsRecord::Coords {
                    timestamp,
                    lat, lon, ele,
                }
            }
            "D" => {
                let millis = parse!("milliseconds");
                let f2 = parse!("field 2");
                let f3 = parse!("field 3");
                let ele_change_mm: i64 = parse!("elevation change");
                let speed = parse!("speed");
                let heading = parse!("heading");

                GpsRecord::Delta {
                    millis,
                    f2,
                    f3,
                    ele_change: ele_change_mm as f64 / 100.,
                    speed,
                    heading,
                }
            }
            _ => bail!("unrecognized tag {}", tag),
        };
        println!("{:?}", record);
        records.push(record);
    }
    let max_speed = records
        .iter()
        .filter_map(|r| match r {
            GpsRecord::Delta { speed, .. } => Some(speed),
            _ => None,
        })
        .max_by(|a, b| if a > b { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less })
        .unwrap();
    println!("max speed: {} m/s, {:.1} MPH", max_speed, max_speed * 2.2369363);
    Ok(())
}
