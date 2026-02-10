#[derive(Clone, Debug)]
struct Counter {
    value: i64,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }
}

pub fn run_sample_script() -> anyhow::Result<(), Box<rhai::EvalAltResult>> {
    let mut engine = rhai::Engine::new();

    engine
        .register_type_with_name::<Counter>("Counter")
        .register_fn("NewCounter", Counter::new)
        .register_fn("increment", Counter::increment);

    let result = engine.eval::<Counter>(
        "
            let x = NewCounter();
            x.increment();
            x
        ",
    )?;

    println!("{result:?}");

    let result = engine.eval::<Counter>(
        "
            let x = [ NewCounter() ];
            x[0].increment();
            x[0].increment();
            x[0]
        ",
    )?;

    println!("{result:?}");

    Ok(())
}
