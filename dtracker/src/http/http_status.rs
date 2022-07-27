use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum HttpStatus {
    Ok,
    NotFound,
}

impl FromStr for HttpStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "200 OK" => Ok(HttpStatus::Ok),
            "404 NOT FOUND" => Ok(HttpStatus::NotFound),
            _ => Err(()),
        }
    }
}

impl ToString for HttpStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Ok => "200 OK".to_string(),
            Self::NotFound => "404 NOT FOUND".to_string(),
        }
    }
}
