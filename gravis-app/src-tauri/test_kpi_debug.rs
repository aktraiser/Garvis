// Test Debug KPI Patterns
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug KPI Pattern Testing");
    
    let revenue_pattern = Regex::new(r"(?i)(revenue[s]?|chiffre\s+d'affaires|ca)\s*(?:of|was|reached|increased|to|de|:|at|a\s+atteint)?\s*(?:to|à)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")?;
    
    let test_cases = vec![
        "Revenue reached $2,150.5 million",
        "Le chiffre d'affaires a atteint 2.150,5 millions d'euros",
        "CA: 2.150,5 millions d'euros",
        "Revenue: 2,150.5 million USD",
    ];
    
    for test in test_cases {
        println!("\nTesting: {}", test);
        
        if let Some(captures) = revenue_pattern.captures(test) {
            println!("  ✅ Match found!");
            for (i, capture) in captures.iter().enumerate() {
                if let Some(cap) = capture {
                    println!("    Group {}: '{}'", i, cap.as_str());
                }
            }
        } else {
            println!("  ❌ No match");
        }
    }
    
    Ok(())
}