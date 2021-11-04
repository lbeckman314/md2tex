/// Used to keep track of current pulldown_cmark "event".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Code,
    Emphasis,
    Header,
    Strong,
    Table,
    TableHead,
    Text,
    BlockQuote,
}

impl Default for EventType {
    fn default() -> Self {
        Self::Text
    }
}
