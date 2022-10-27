use clap::StructOpt;
use paperazzi::PRZZIConfig;
use paperazzi::PRZZIError;
use paperazzi::PRZZI;

fn main() -> Result<(), PRZZIError> {
    let config = PRZZIConfig::parse();
    let mut przzi = PRZZI::new(config)?;
    przzi.run()?;
    Ok(())
}
