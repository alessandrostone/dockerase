use crate::display::{print_disk_usage, print_error, print_footer, print_header};
use crate::docker::Docker;

pub fn run() -> Result<(), String> {
    if !Docker::is_available() {
        print_error("Docker is not available. Is Docker running?");
        return Err("Docker not available".to_string());
    }

    print_header();
    println!();

    let usage = Docker::get_disk_usage()?;
    print_disk_usage(&usage);
    print_footer();

    Ok(())
}
