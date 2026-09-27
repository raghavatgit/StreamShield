#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_affinity_insertion() {
        let mut set = HashSet::new();
        set.insert(12345isize);
        assert!(set.contains(&12345isize));
        set.remove(&12345isize);
        assert!(!set.contains(&12345isize));
    }
}
