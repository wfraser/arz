# ARZ

An attempt to reverse-engineer the internal file format used by Trace Snow.

The backend service is dead, and uploads from the app fail, which means any pending records are
trapped in their weird ".ARZ" file format, unless I can reconstruct them from the files pulled off
the phone.

.ARZ is a zip container, and contains two files:
    `data-YYYY-MM-DD-hh-mm-ss.gps`: GPS coordinates, elevation, speed.
    `data-YYYY-MM-DD-hh-mm-ss.acc`: accelerometer data? Haven't figured this one out yet.

These files are basically CSV files, each line is a different type of record denoted by the first
field.

GPS files have some header records "U" (username), "V" (file version), "A" (app version), and "I"
(device identifiers), and then alternate between an "H" record every minute and some "D" records in
between them.

"H" record fields:
    1. unix timestamp in UTC
    2. latitude
    3. longitude
    4. elevation, in meters
    5. unix timestamp in local timezone
    6. RFC 3339 string of UTC date time
    7. RFC 3339 string of local date time

    The date/time records are super redundant.

"D" record fields:
    1. time delta in milliseconds from the last "H"
    2. ?
    3. ?
    4. change in elevation since the last "H", in millimeters
    5. current speed in meters / sec
    6. heading in degrees? not sure.

ACC files have the same header records (except "A" is omitted), then "H" records (which are
different from .GPS files) every minute, then "D" records (which are also different) approximately
every 10 milliseconds in between.

"H" record fields:
    1. not sure, but it goes up by ~60,000 each time, so milliseconds offset from some unknown
       starting point. Maybe phone boot time?
    2. unix timestamp in local timezone
    3. RFC 3339 string of local date time

"D" record fields:
    1. milliseconds since last "H" record
    2. accelerometer data, unknown axis or units
    3. accelerometer data, Y-axis (up/down), meters/sec?
    4. accelerometer data, unknown axis or units

I'm guessing on the accelerometer data bit, based on the file name, and the presence of what looks
like a 3-vector, and the middle coordinate being close to 9.8 an awful lot of the time.
