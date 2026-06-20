use anyhow::Result;

fn main() -> Result<()> {
    iron_insights_pipeline::download::run()
}
