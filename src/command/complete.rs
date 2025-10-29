/// Completion item with text and optional description
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub text: String,
    pub description: Option<String>,
}

impl CompletionItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Trait for providing completions
///
/// Commands can implement this to provide argument completions.
pub trait Completer: Send + Sync {
    /// Get completions for the given prefix
    ///
    /// `args` contains already-typed arguments
    /// `current` is the partial argument being completed
    fn complete(&self, args: &[String], current: &str) -> Vec<CompletionItem>;
}

/// Simple completion from a fixed list
pub struct ListCompleter {
    items: Vec<String>,
}

impl ListCompleter {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl Completer for ListCompleter {
    fn complete(&self, _args: &[String], current: &str) -> Vec<CompletionItem> {
        self.items
            .iter()
            .filter(|item| item.starts_with(current))
            .map(|item| CompletionItem::new(item.clone()))
            .collect()
    }
}

/// File path completer (placeholder - would need actual FS access)
pub struct FileCompleter;

impl Completer for FileCompleter {
    fn complete(&self, _args: &[String], current: &str) -> Vec<CompletionItem> {
        use std::fs;
        use std::path::Path;

        // Parse the current path
        let path = Path::new(current);
        let (dir, prefix) = if current.ends_with('/') || current.ends_with('\\') {
            // User is completing within a directory
            (path, "")
        } else {
            // User is typing a file/dir name
            let dir = path.parent().unwrap_or(Path::new("."));
            let prefix = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            (dir, prefix)
        };

        // List directory contents
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };

        let mut completions = Vec::new();

        for entry in entries.flatten() {
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };

            // Filter by prefix
            if !name.starts_with(prefix) {
                continue;
            }

            // Check if it's a directory
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            // Build full path
            let full_path = if current.is_empty() {
                name.clone()
            } else {
                let parent = dir.to_string_lossy();
                if parent == "." {
                    name.clone()
                } else {
                    format!("{}/{}", parent, name)
                }
            };

            let description = if is_dir {
                "directory".to_string()
            } else {
                "file".to_string()
            };

            completions.push(CompletionItem {
                text: full_path,
                description: Some(description),
            });
        }

        // Sort completions
        completions.sort_by(|a, b| a.text.cmp(&b.text));

        completions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("test").with_description("A test item");

        assert_eq!(item.text, "test");
        assert_eq!(item.description, Some("A test item".to_string()));
    }

    #[test]
    fn test_list_completer() {
        let completer = ListCompleter::new(vec![
            "hello".to_string(),
            "help".to_string(),
            "world".to_string(),
        ]);

        let results = completer.complete(&[], "hel");
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|c| c.text == "hello"));
        assert!(results.iter().any(|c| c.text == "help"));
    }

    #[test]
    fn test_list_completer_no_match() {
        let completer = ListCompleter::new(vec!["hello".to_string(), "world".to_string()]);

        let results = completer.complete(&[], "xyz");
        assert_eq!(results.len(), 0);
    }
}
