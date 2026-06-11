use chrono::{DateTime, NaiveDate, Utc};

pub mod app_trait;
pub mod console_usecase;
pub mod usecase;

// 1970-01-01 00:00:00 UTC
pub const SAFE_MIN_DATETIME: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
    NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap(),
    Utc,
);

// 9999-12-31 23:59:59 UTC
pub const SAFE_MAX_DATETIME: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
    NaiveDate::from_ymd_opt(9999, 12, 31)
        .unwrap()
        .and_hms_opt(23, 59, 59)
        .unwrap(),
    Utc,
);
