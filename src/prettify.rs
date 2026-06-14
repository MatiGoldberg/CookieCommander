use anyhow::Result;

pub trait Prettifier: Send + Sync {
    /// Checks if the content is in a state that can be formatted by this prettifier.
    fn can_prettify(&self, content: &str) -> bool;
    
    /// Reformats the content.
    fn prettify(&self, content: &str) -> Result<String>;
}

pub struct JsonPrettifier;

impl Prettifier for JsonPrettifier {
    fn can_prettify(&self, content: &str) -> bool {
        let trimmed = content.trim_end();
        trimmed.lines().count() <= 1 && !trimmed.is_empty()
    }

    fn prettify(&self, content: &str) -> Result<String> {
        let val: serde_json::Value = serde_json::from_str(content)?;
        let pretty = serde_json::to_string_pretty(&val)?;
        Ok(pretty)
    }
}

pub fn get_prettifier(extension: &str) -> Option<Box<dyn Prettifier>> {
    match extension.to_lowercase().as_str() {
        "json" => Some(Box::new(JsonPrettifier)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_prettifier_can_prettify() {
        let prettifier = JsonPrettifier;
        
        // Single-line JSON
        assert!(prettifier.can_prettify(r#"{"a": 1, "b": "hello"}"#));
        
        // Empty
        assert!(!prettifier.can_prettify(""));
        
        // Multi-line JSON
        assert!(!prettifier.can_prettify(r#"{
  "a": 1
}"#));
    }

    #[test]
    fn test_json_prettifier_format() {
        let prettifier = JsonPrettifier;
        let minified = r#"{"a":1,"b":[true,false]}"#;
        let prettified = prettifier.prettify(minified).unwrap();
        
        // Verify it parsed and formatted
        let expected = serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(minified).unwrap()).unwrap();
        assert_eq!(prettified, expected);
        
        // Test invalid JSON
        assert!(prettifier.prettify(r#"{"a":1"#).is_err());
    }

    #[test]
    fn test_get_prettifier() {
        assert!(get_prettifier("json").is_some());
        assert!(get_prettifier("JSON").is_some());
        assert!(get_prettifier("txt").is_none());
    }
}
