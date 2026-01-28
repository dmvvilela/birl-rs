//! sandwich-core: Core image composition logic for the Sandwich app
//!
//! This crate provides the business logic for layering clothing items over base models.
//! It handles SKU normalization, layer ordering, and image composition.

pub mod cache;
pub mod compositor;
pub mod layers;
pub mod models;

// Re-export commonly used types
pub use cache::generate_cache_key;
pub use compositor::{compose_layers, Compositor};
pub use layers::{parse_params, LayerNormalizer};
pub use models::{LayerOrder, LayerParam, Sku, View};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // Parse parameters
        let params_str = "hoodies/baerskin4-black-xl,pants/cargo-darkgreen-40,hats/beanie-black";
        let params = parse_params(params_str);
        assert_eq!(params.len(), 3);

        // Normalize for front view
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize_all(&params);

        // Should be sorted by layer order: pants, hoodies, hats
        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0].category, "pants");
        assert_eq!(normalized[1].category, "hoodies");
        assert_eq!(normalized[2].category, "hats");

        // Generate cache key
        let cache_key = generate_cache_key(&normalized, View::Front, "swatthermals-black");
        assert!(!cache_key.is_empty());
        assert!(cache_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_gloves_and_jackets_normalization() {
        let params_str = "gloves/ski-black,jackets/greenland-grey";
        let params = parse_params(params_str);

        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize_all(&params);

        // Gloves should be gloves-top (ski), jackets should be outer-jackets (greenland)
        assert!(normalized.iter().any(|p| p.category == "gloves-top"));
        assert!(normalized.iter().any(|p| p.category == "outer-jackets"));
    }

    #[test]
    fn test_patches_with_softshell() {
        let params_str = "jackets/softshell-grey,patches-left/americanflag-red";
        let params = parse_params(params_str);

        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize_all(&params);

        // Patch should use softshell-patches-left
        assert!(normalized
            .iter()
            .any(|p| p.category == "softshell-patches-left"));
    }

    #[test]
    fn test_back_view_filters_patches() {
        let params_str = "hoodies/baerskin4-black,patches-left/americanflag-red";
        let params = parse_params(params_str);

        let normalizer = LayerNormalizer::new(View::Back, &params);
        let normalized = normalizer.normalize_all(&params);

        // Patches should be filtered out for back view
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].category, "hoodies");
    }

    #[test]
    fn test_left_view_filters_categories() {
        let params_str =
            "pants/cargo-black,hoodies/baerskin4-black,jackets/softshell-grey,hats/beanie-black";
        let params = parse_params(params_str);

        let normalizer = LayerNormalizer::new(View::Left, &params);
        let normalized = normalizer.normalize_all(&params);

        // Only hoodies and jackets should be included for left view
        assert_eq!(normalized.len(), 2);
        assert!(normalized.iter().any(|p| p.category == "hoodies"));
        assert!(normalized.iter().any(|p| p.category == "jackets"));
        assert!(!normalized.iter().any(|p| p.category == "pants"));
        assert!(!normalized.iter().any(|p| p.category == "hats"));
    }
}
