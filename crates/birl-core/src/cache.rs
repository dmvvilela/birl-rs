use crate::models::{LayerParam, View};
use xxhash_rust::xxh64::xxh64;

/// Generate a cache key using xxHash64
/// This matches the TypeScript implementation using Bun.hash.xxHash64
pub fn generate_cache_key(params: &[LayerParam], view: View, plate_value: &str) -> String {
    // Sort parameters to ensure consistent cache keys
    let mut param_strings: Vec<String> = params
        .iter()
        .map(|p| format!("{}/{}", p.category, p.sku.as_str()))
        .collect();
    param_strings.sort();

    // Create combined string: sorted_params_view_plate
    let combined_string = format!("{}_{}_{}",
        param_strings.join("_"),
        view.as_str(),
        plate_value
    );

    // Hash using xxHash64 (seed 0, matching Bun.hash default)
    let hash = xxh64(combined_string.as_bytes(), 0);

    // Convert to hexadecimal string (matching TypeScript .toString(16))
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Sku;

    #[test]
    fn test_generate_cache_key() {
        let params = vec![
            LayerParam::new("hoodies", Sku::new("hoodie-black")),
            LayerParam::new("pants", Sku::new("cargo-darkgreen")),
        ];
        let key = generate_cache_key(&params, View::Front, "base-model-black");

        // Should produce a valid hex string
        assert!(!key.is_empty());
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_key_consistency() {
        let params1 = vec![
            LayerParam::new("hoodies", Sku::new("hoodie-black")),
            LayerParam::new("pants", Sku::new("cargo-darkgreen")),
        ];
        let params2 = vec![
            LayerParam::new("pants", Sku::new("cargo-darkgreen")),
            LayerParam::new("hoodies", Sku::new("hoodie-black")),
        ];

        let key1 = generate_cache_key(&params1, View::Front, "base-model-black");
        let key2 = generate_cache_key(&params2, View::Front, "base-model-black");

        // Should produce the same key regardless of order
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_differs_by_view() {
        let params = vec![LayerParam::new("hoodies", Sku::new("hoodie-black"))];

        let key_front = generate_cache_key(&params, View::Front, "base-model-black");
        let key_back = generate_cache_key(&params, View::Back, "base-model-black");

        // Should produce different keys for different views
        assert_ne!(key_front, key_back);
    }

    #[test]
    fn test_cache_key_differs_by_plate() {
        let params = vec![LayerParam::new("hoodies", Sku::new("hoodie-black"))];

        let key1 = generate_cache_key(&params, View::Front, "base-model-black");
        let key2 = generate_cache_key(&params, View::Front, "patch-plate");

        // Should produce different keys for different plates
        assert_ne!(key1, key2);
    }
}
