#[cfg(test)]
mod tests {
    #[test]
    fn test_filter_matching() {
        let targets = vec!["discord.exe", "slack.exe"];
        assert!(targets.iter().any(|t| "discord.exe".contains(t)));
        assert!(!targets.iter().any(|t| "notepad.exe".contains(t)));
    }
}
