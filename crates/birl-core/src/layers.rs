use crate::models::{LayerParam, Sku, View};

/// Normalize and filter layer parameters based on view and context
pub struct LayerNormalizer {
    view: View,
    has_softshell_jacket: bool,
}

impl LayerNormalizer {
    pub fn new(view: View, params: &[LayerParam]) -> Self {
        // First pass to detect softshell jacket
        let has_softshell_jacket = params.iter().any(|param| {
            param.category == "jackets" && param.sku.as_str().contains("softshell")
        });

        Self {
            view,
            has_softshell_jacket,
        }
    }

    /// Normalize a single layer parameter
    pub fn normalize(&self, param: &LayerParam) -> Option<LayerParam> {
        let category = &param.category;
        let sku = param.sku.as_str();

        // Skip categories that aren't relevant for specific views
        if matches!(self.view, View::Left | View::Right) {
            if !["hoodies", "jackets", "patches-left", "patches-right"].contains(&category.as_str())
            {
                return None;
            }
        }

        // Handle patches based on position, jacket type, and view
        if category.starts_with("patches-") {
            return self.normalize_patch(category, sku);
        }

        // Handle gloves special case
        if category == "gloves" {
            return self.normalize_gloves(sku);
        }

        // Handle jackets special case
        if category == "jackets" {
            return self.normalize_jacket(sku);
        }

        Some(param.clone())
    }

    /// Normalize patch parameters
    fn normalize_patch(&self, category: &str, sku: &str) -> Option<LayerParam> {
        // Extract position from "patches-left" or "patches-right"
        let position = category.strip_prefix("patches-")?;

        // Skip patches for back view
        if self.view == View::Back {
            return None;
        }

        // For left/right views, only show the patch on the matching side
        if (self.view == View::Left && position != "left")
            || (self.view == View::Right && position != "right")
        {
            return None;
        }

        // Determine base category based on jacket type
        let base_category = if self.has_softshell_jacket {
            "softshell-patches"
        } else {
            "patches"
        };

        // For front view, use position suffix
        let new_category = if self.view == View::Front {
            format!("{}-{}", base_category, position)
        } else {
            // For side views, use the standard patch folder
            base_category.to_string()
        };

        Some(LayerParam::new(new_category, sku))
    }

    /// Normalize gloves parameters
    fn normalize_gloves(&self, sku: &str) -> Option<LayerParam> {
        // Ski gloves go on top, others go on bottom
        // Careful: "regular" is NOT a ski glove
        let is_ski_glove = sku.starts_with("ski");
        let category = if is_ski_glove {
            "gloves-top"
        } else {
            "gloves-bottom"
        };

        Some(LayerParam::new(category, sku))
    }

    /// Normalize jacket parameters
    fn normalize_jacket(&self, sku: &str) -> Option<LayerParam> {
        // Greenland jackets are outer jackets
        let is_outer_jacket = sku.contains("greenland");
        let category = if is_outer_jacket {
            "outer-jackets"
        } else {
            "jackets"
        };

        Some(LayerParam::new(category, sku))
    }

    /// Normalize and sort all parameters by layer order
    pub fn normalize_all(&self, params: &[LayerParam]) -> Vec<LayerParam> {
        let mut normalized: Vec<LayerParam> = params
            .iter()
            .filter_map(|param| self.normalize(param))
            .collect();

        // Sort by layer order
        normalized.sort_by_key(|param| param.layer_order());

        normalized
    }
}

/// Parse comma-separated parameter string into LayerParams
pub fn parse_params(params_str: &str) -> Vec<LayerParam> {
    params_str
        .split(',')
        .filter_map(|param| {
            let parts: Vec<&str> = param.split('/').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                Some(LayerParam::new(parts[0], Sku::new(parts[1])))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params() {
        let params = parse_params("hoodies/hoodie-black-xl,pants/cargo-darkgreen-40");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].category, "hoodies");
        assert_eq!(params[0].sku.as_str(), "hoodie-black");
        assert_eq!(params[1].category, "pants");
        assert_eq!(params[1].sku.as_str(), "cargo-darkgreen");
    }

    #[test]
    fn test_normalize_gloves() {
        let params = vec![LayerParam::new("gloves", "ski-black")];
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(normalized.category, "gloves-top");

        let params = vec![LayerParam::new("gloves", "regular-gloves-black")];
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(normalized.category, "gloves-bottom");
    }

    #[test]
    fn test_normalize_jackets() {
        let params = vec![LayerParam::new("jackets", "greenland-black")];
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(normalized.category, "outer-jackets");

        let params = vec![LayerParam::new("jackets", "softshell-grey")];
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(normalized.category, "jackets");
    }

    #[test]
    fn test_normalize_patches_back_view() {
        let params = vec![LayerParam::new("patches-left", "flag-patch-red")];
        let normalizer = LayerNormalizer::new(View::Back, &params);
        assert!(normalizer.normalize(&params[0]).is_none());
    }

    #[test]
    fn test_normalize_patches_with_softshell() {
        let params = vec![
            LayerParam::new("jackets", "softshell-grey"),
            LayerParam::new("patches-left", "flag-patch-red"),
        ];
        let normalizer = LayerNormalizer::new(View::Front, &params);

        // Jacket should stay as jackets
        let jacket_normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(jacket_normalized.category, "jackets");

        // Patch should use softshell-patches
        let patch_normalized = normalizer.normalize(&params[1]).unwrap();
        assert_eq!(patch_normalized.category, "softshell-patches-left");
    }

    #[test]
    fn test_normalize_patches_left_view() {
        let params = vec![
            LayerParam::new("patches-left", "flag-patch-red"),
            LayerParam::new("patches-right", "canadaflag-red"),
        ];
        let normalizer = LayerNormalizer::new(View::Left, &params);

        // Left patch should be included
        let left_normalized = normalizer.normalize(&params[0]).unwrap();
        assert_eq!(left_normalized.category, "patches");

        // Right patch should be filtered out
        assert!(normalizer.normalize(&params[1]).is_none());
    }

    #[test]
    fn test_layer_ordering() {
        let params = vec![
            LayerParam::new("hats", "beanie-black"),
            LayerParam::new("hoodies", "hoodie-black"),
            LayerParam::new("pants", "cargo-darkgreen"),
        ];
        let normalizer = LayerNormalizer::new(View::Front, &params);
        let normalized = normalizer.normalize_all(&params);

        // Should be sorted: pants, hoodies, hats
        assert_eq!(normalized[0].category, "pants");
        assert_eq!(normalized[1].category, "hoodies");
        assert_eq!(normalized[2].category, "hats");
    }
}
