use config::Config;

const ENV_PREFIX: &'static str = "EZA";

pub fn init_config() -> Config {
  let mut settings = Config::default();
  settings
    .merge(config::Environment::with_prefix(ENV_PREFIX))
    .unwrap();

  settings
}
