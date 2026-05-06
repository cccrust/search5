mod handlers;

pub use handlers::*;

use minijinja::Environment;

pub fn create_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("index", include_str!("templates/index.html"))
        .unwrap();
    env.add_template("search", include_str!("templates/search.html"))
        .unwrap();
    env
}
