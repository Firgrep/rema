pub mod gh;
pub mod transformer;
pub mod writer;

pub struct Rema {}

impl Rema {
    pub fn run() {
        match gh::list_releases() {
            Ok(releases) => {
                let latest_versions = transformer::get_latest_versions(releases);
                println!("Latest versions:");
                for (app_name, version) in &latest_versions {
                    println!("{}: {}", app_name, version);
                }
            }
            Err(err) => {
                eprintln!("Error fetching releases: {}", err);
                std::process::exit(1); // Exit with non-zero status on error
            }
        }
    }
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
