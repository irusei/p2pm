use semver::Version;

pub fn fix_semver_version(version: &str) -> Result<Version, semver::Error> {
    let cleaned = version.trim_start_matches('v');

    let mut parts: Vec<&str> = cleaned.split('.').collect();

    while parts.len() < 3 {
        parts.push("0");
    }

    Version::parse(&parts.join("."))
}
