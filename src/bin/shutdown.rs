use std::fs;

fn main() {
    match fs::File::create("acpi:off") {
        Err(err) => println!("Failed to cut power (error: {})", err),
        Ok(..) => println!("Good bye!"),
    }
}
