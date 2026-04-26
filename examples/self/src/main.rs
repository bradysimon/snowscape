/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(app::previews)
}

#[cfg(test)]
mod tests {

    #[test]
    fn passes_visual_tests() -> Result<(), snowscape::test::Error> {
        snowscape::test::run(
            app::previews,
            format!("{}/tests", env!("CARGO_MANIFEST_DIR")),
        )
    }
}
