/// Pre-made example combinations for easy testing
pub struct Example {
    pub name: &'static str,
    pub description: &'static str,
    pub params: &'static str,
}

pub const EXAMPLES: &[Example] = &[
    Example {
        name: "basic",
        description: "Single black hoodie on front view",
        params: "hoodies/hoodie-black",
    },
    Example {
        name: "full-outfit",
        description: "Complete outfit: hoodie, pants, and beanie",
        params: "hoodies/hoodie-black,pants/cargo-darkgreen,hats/beanie-black",
    },
    Example {
        name: "with-patches",
        description: "Hoodie with American flag patch on left",
        params: "hoodies/hoodie-black,patches-left/flag-patch-red",
    },
    Example {
        name: "jacket-outfit",
        description: "Jacket over hoodie with pants",
        params: "hoodies/hoodie-black,jackets/softshell-grey,pants/cargo-black",
    },
    Example {
        name: "gloves-hat",
        description: "Full winter outfit with gloves and hat",
        params: "hoodies/hoodie-black,pants/cargo-black,hats/beanie-black,gloves/leather-gloves-black",
    },
    Example {
        name: "outer-jacket",
        description: "Greenland outer jacket over hoodie",
        params: "hoodies/hoodie-black,jackets/greenland-black,pants/cargo-darkgreen",
    },
];

pub fn get_example(name: &str) -> Option<&'static Example> {
    EXAMPLES.iter().find(|e| e.name == name)
}

pub fn list_examples() {
    println!("Available examples:\n");
    for example in EXAMPLES {
        println!("  {:<20} - {}", example.name, example.description);
        println!("  {:<20}   params: {}\n", "", example.params);
    }
}
