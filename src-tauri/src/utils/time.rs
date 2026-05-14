use chrono::{DateTime, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn iso_now() -> String {
    now().to_rfc3339()
}
