use chrono::prelude::*;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub time: DateTime<FixedOffset>,
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
    pub speed: f64,
    pub course: f64,
}

pub fn write_gpx(mut w: impl Write, segments: &[&[Point]]) -> io::Result<()> {
    writeln!(w, r#"<?xml version="1.0" encoding="utf-8"?>
<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1" creator="wfraser/arz/1">"#)?;
    for &seg in segments {
        writeln!(w, "<trk><trkseg>")?;
        for point in seg {
            writeln!(w, r#"<trkpt lat="{}" lon="{}"><ele>{}</ele><time>{}</time><speed>{}</speed><course>{}</course></trkpt>"#,
                point.lat,
                point.lon,
                point.ele,
                point.time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                point.speed,
                point.course)?;
        }
        writeln!(w, "</trkseg></trk>")?;
    }
    writeln!(w, "</gpx>")?;
    Ok(())
}
