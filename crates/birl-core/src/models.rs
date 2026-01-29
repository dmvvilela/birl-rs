use serde::{Deserialize, Serialize};
use std::fmt;

/// View types for the birl composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Front,
    Back,
    Side,
    Left,
    Right,
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            View::Front => write!(f, "front"),
            View::Back => write!(f, "back"),
            View::Side => write!(f, "side"),
            View::Left => write!(f, "left"),
            View::Right => write!(f, "right"),
        }
    }
}

impl View {
    pub fn as_str(&self) -> &'static str {
        match self {
            View::Front => "front",
            View::Back => "back",
            View::Side => "side",
            View::Left => "left",
            View::Right => "right",
        }
    }

    /// Get the plate value for this view
    pub fn plate_value(&self) -> &'static str {
        match self {
            View::Left | View::Right => "patch-plate",
            View::Side => "side-special-plate",
            View::Front | View::Back => "swatthermals-black",
        }
    }

    /// Check if patches are visible in this view
    pub fn allows_patches(&self) -> bool {
        !matches!(self, View::Back)
    }

    /// Check if this view allows full composition
    pub fn allows_full_composition(&self) -> bool {
        matches!(self, View::Front)
    }

    /// Check if this view only allows certain categories
    pub fn allowed_categories(&self) -> Option<&[&str]> {
        match self {
            View::Left | View::Right => {
                Some(&["hoodies", "jackets", "patches-left", "patches-right"])
            }
            _ => None,
        }
    }
}

/// Layer ordering with compile-time guarantees
/// The order here defines the z-index of layers (lowest to highest)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerOrder {
    Pants = 0,
    Tops = 1,
    Hoodies = 2,
    GlovesBottom = 3,
    Jackets = 4,
    GlovesTop = 5,
    OuterJackets = 6,
    Hats = 7,
    Patches = 8,
    PatchesLeft = 9,
    PatchesRight = 10,
    SoftshellPatches = 11,
    SoftshellPatchesLeft = 12,
    SoftshellPatchesRight = 13,
}

impl LayerOrder {
    pub fn from_category(category: &str) -> Option<Self> {
        match category {
            "pants" => Some(LayerOrder::Pants),
            "tops" => Some(LayerOrder::Tops),
            "hoodies" => Some(LayerOrder::Hoodies),
            "gloves-bottom" => Some(LayerOrder::GlovesBottom),
            "jackets" => Some(LayerOrder::Jackets),
            "gloves-top" => Some(LayerOrder::GlovesTop),
            "outer-jackets" => Some(LayerOrder::OuterJackets),
            "hats" => Some(LayerOrder::Hats),
            "patches" => Some(LayerOrder::Patches),
            "patches-left" => Some(LayerOrder::PatchesLeft),
            "patches-right" => Some(LayerOrder::PatchesRight),
            "softshell-patches" => Some(LayerOrder::SoftshellPatches),
            "softshell-patches-left" => Some(LayerOrder::SoftshellPatchesLeft),
            "softshell-patches-right" => Some(LayerOrder::SoftshellPatchesRight),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LayerOrder::Pants => "pants",
            LayerOrder::Tops => "tops",
            LayerOrder::Hoodies => "hoodies",
            LayerOrder::GlovesBottom => "gloves-bottom",
            LayerOrder::Jackets => "jackets",
            LayerOrder::GlovesTop => "gloves-top",
            LayerOrder::OuterJackets => "outer-jackets",
            LayerOrder::Hats => "hats",
            LayerOrder::Patches => "patches",
            LayerOrder::PatchesLeft => "patches-left",
            LayerOrder::PatchesRight => "patches-right",
            LayerOrder::SoftshellPatches => "softshell-patches",
            LayerOrder::SoftshellPatchesLeft => "softshell-patches-left",
            LayerOrder::SoftshellPatchesRight => "softshell-patches-right",
        }
    }
}

/// Normalized SKU that removes size variations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sku(String);

impl Sku {
    /// Create a new normalized SKU by removing size suffixes
    /// Examples:
    ///   mensdenimjeans-blue-36 -> mensdenimjeans-blue
    ///   baerskinzip-grey-s -> baerskinzip-grey
    ///   baerskin4-black-lxl -> baerskin4-black
    pub fn new(raw: &str) -> Self {
        // Convert to lowercase first
        let mut result = raw.trim().to_lowercase();

        // Apply pattern matching to remove size suffixes
        let size_patterns = [
            "-xs", "-s", "-m", "-l", "-xl", "-xxl", "-2xl", "-3xl", "-4xl", "-5xl", "-lxl",
        ];

        for pattern in &size_patterns {
            if result.ends_with(pattern) {
                result = result[..result.len() - pattern.len()].to_string();
                break;
            }
        }

        // Also handle numeric sizes like -36, -38, -40
        if let Some(pos) = result.rfind('-') {
            if result[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                result = result[..pos].to_string();
            }
        }

        Self(result)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Sku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Sku {
    fn from(s: &str) -> Self {
        Sku::new(s)
    }
}

impl From<String> for Sku {
    fn from(s: String) -> Self {
        Sku::new(&s)
    }
}

/// A layer parameter with category and SKU
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerParam {
    pub category: String,
    pub sku: Sku,
}

impl LayerParam {
    pub fn new(category: impl Into<String>, sku: impl Into<Sku>) -> Self {
        Self {
            category: category.into(),
            sku: sku.into(),
        }
    }

    /// Parse from "category/sku" format
    pub fn parse(param: &str) -> Option<Self> {
        let parts: Vec<&str> = param.split('/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Get the layer order for this parameter
    pub fn layer_order(&self) -> Option<LayerOrder> {
        LayerOrder::from_category(&self.category)
    }

    /// Format as "category/sku"
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.category, self.sku)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sku_normalization() {
        assert_eq!(Sku::new("mensdenimjeans-blue-36").as_str(), "mensdenimjeans-blue");
        assert_eq!(Sku::new("baerskinzip-grey-s").as_str(), "baerskinzip-grey");
        assert_eq!(Sku::new("baerskin4-black-lxl").as_str(), "baerskin4-black");
        assert_eq!(Sku::new("baerskin4-black-xl").as_str(), "baerskin4-black");
        assert_eq!(Sku::new("baerskin4-black-2xl").as_str(), "baerskin4-black");
        assert_eq!(Sku::new("cargo-darkgreen-40").as_str(), "cargo-darkgreen");
    }

    #[test]
    fn test_view_plate_value() {
        assert_eq!(View::Front.plate_value(), "swatthermals-black");
        assert_eq!(View::Back.plate_value(), "swatthermals-black");
        assert_eq!(View::Side.plate_value(), "side-special-plate");
        assert_eq!(View::Left.plate_value(), "patch-plate");
        assert_eq!(View::Right.plate_value(), "patch-plate");
    }

    #[test]
    fn test_view_allows_patches() {
        assert!(View::Front.allows_patches());
        assert!(!View::Back.allows_patches());
        assert!(View::Side.allows_patches());
        assert!(View::Left.allows_patches());
        assert!(View::Right.allows_patches());
    }

    #[test]
    fn test_layer_param_parse() {
        let param = LayerParam::parse("hoodies/baerskin4-black").unwrap();
        assert_eq!(param.category, "hoodies");
        assert_eq!(param.sku.as_str(), "baerskin4-black");
    }

    #[test]
    fn test_layer_order() {
        assert!(LayerOrder::Pants < LayerOrder::Tops);
        assert!(LayerOrder::Tops < LayerOrder::Hoodies);
        assert!(LayerOrder::Hoodies < LayerOrder::Jackets);
        assert!(LayerOrder::Jackets < LayerOrder::Hats);
    }
}
