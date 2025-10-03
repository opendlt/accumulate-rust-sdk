use enum_test_standalone::{test_enum_roundtrips, test_specific_values, test_enum_count};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Standalone Enum Tests ===");

    println!("\n1. Testing enum count...");
    test_enum_count()?;

    println!("\n2. Testing specific enum values...");
    test_specific_values()?;

    println!("\n3. Testing enum JSON roundtrips...");
    test_enum_roundtrips()?;

    println!("\n🎉 All enum tests passed! G1=PASS (14/14 enums)");
    Ok(())
}