/// Placeholder greeting command — will be replaced in M1 with real API commands.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Tweak.", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_formatted_string() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! Welcome to Tweak.");
    }

    #[test]
    fn greet_handles_empty_name() {
        let result = greet("");
        assert_eq!(result, "Hello, ! Welcome to Tweak.");
    }
}

