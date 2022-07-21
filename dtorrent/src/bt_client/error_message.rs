use std::fmt::Debug;

/**
Represents the inside message of an error, and can be nicely printed using Debug.

```rust
use dtorrent::bt_client::error_message::ErrorMessage;

#[derive(Debug)]
enum Errors {
   AnError(ErrorMessage),
}

let message = ErrorMessage::new("Something bad happened...".to_string());
let err = Errors::AnError(message);

eprintln!("{:?}", err);
```
*/
#[derive(PartialEq)]
pub struct ErrorMessage {
    pub message: String,
}

impl Debug for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl ErrorMessage {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
