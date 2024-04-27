use {
    criterion::{criterion_group, criterion_main, Criterion},
    wast::{parser, parser::ParseBuffer, Wat},
};

fn new_stitch_store_and_instance(
    bytes: &[u8],
) -> (makepad_stitch::Store, makepad_stitch::Instance) {
    use makepad_stitch::{Engine, Linker, Module, Store};

    let engine = Engine::new();
    let mut store = Store::new(engine);
    let module = Module::new(store.engine(), &bytes).unwrap();
    let instance = Linker::new().instantiate(&mut store, &module).unwrap();
    (store, instance)
}

fn new_wasmi_store_and_instance(bytes: &[u8]) -> (wasmi::Store<()>, wasmi::Instance) {
    use wasmi::{Config, Engine, Linker, Module, Store};

    let config = Config::default();
    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());
    let module = Module::new(store.engine(), bytes).unwrap();
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &module)
        .unwrap();
    let instance = instance.start(&mut store).unwrap();
    (store, instance)
}

fn fac(c: &mut Criterion) {
    let buffer = ParseBuffer::new(include_str!("wat/fac.wat")).unwrap();
    let mut wat = parser::parse::<Wat>(&buffer).unwrap();
    let bytes = wat.encode().unwrap();

    let n = 32;
    let mut group = c.benchmark_group("fac");
    group.bench_function("stitch", |b| {
        let (mut store, instance) = new_stitch_store_and_instance(&bytes);
        let fac = instance.exported_func("fac").unwrap();

        b.iter(|| {
            use makepad_stitch::Val;

            fac.call(&mut store, &[Val::I64(n)], &mut [Val::I64(0)])
                .unwrap();
        })
    });
    group.bench_function("wasmi", |b| {
        let (mut store, instance) = new_wasmi_store_and_instance(&bytes);
        let fac = instance.get_func(&store, "fac").unwrap();

        b.iter(|| {
            use wasmi::Value;

            fac.call(&mut store, &[Value::I64(n)], &mut [Value::I64(0)])
                .unwrap();
        })
    });
}

fn fib(c: &mut Criterion) {
    let buffer = ParseBuffer::new(include_str!("wat/fib.wat")).unwrap();
    let mut wat = parser::parse::<Wat>(&buffer).unwrap();
    let bytes = wat.encode().unwrap();

    let n = 32;
    let mut group = c.benchmark_group("fib");
    group.bench_function("stitch", |b| {
        use makepad_stitch::Val;

        let (mut store, instance) = new_stitch_store_and_instance(&bytes);
        let fib = instance.exported_func("fib").unwrap();

        b.iter(|| {
            fib.call(&mut store, &[Val::I64(n as i64)], &mut [Val::I64(0)])
                .unwrap();
        })
    });
    group.bench_function("wasmi", |b| {
        let (mut store, instance) = new_wasmi_store_and_instance(&bytes);
        let fib = instance.get_func(&store, "fib").unwrap();

        b.iter(|| {
            use wasmi::Value;

            fib.call(&mut store, &[Value::I64(n as i64)], &mut [Value::I64(0)])
                .unwrap();
        })
    });
}

fn fill(c: &mut Criterion) {
    let buffer = ParseBuffer::new(include_str!("wat/fill.wat")).unwrap();
    let mut wat = parser::parse::<Wat>(&buffer).unwrap();
    let bytes = wat.encode().unwrap();

    let idx = 0;
    let val = 42;
    let count = 1_048_576;
    let mut group = c.benchmark_group("fib");
    group.bench_function("stitch", |b| {
        use makepad_stitch::Val;

        let (mut store, instance) = new_stitch_store_and_instance(&bytes);
        let fill = instance.exported_func("fill").unwrap();

        b.iter(|| {
            fill.call(
                &mut store,
                &[
                    Val::I32(idx as i32),
                    Val::I32(val as i32),
                    Val::I32(count as i32),
                ],
                &mut [],
            )
            .unwrap();
        });
    });
    group.bench_function("wasmi", |b| {
        use wasmi::Value;

        let (mut store, instance) = new_wasmi_store_and_instance(&bytes);
        let fill = instance.get_func(&store, "fill").unwrap();

        b.iter(|| {
            fill.call(
                &mut store,
                &[
                    Value::I32(idx as i32),
                    Value::I32(val as i32),
                    Value::I32(count as i32),
                ],
                &mut [],
            )
            .unwrap();
        })
    });
}

fn sum(c: &mut Criterion) {
    let buffer = ParseBuffer::new(include_str!("wat/sum.wat")).unwrap();
    let mut wat = parser::parse::<Wat>(&buffer).unwrap();
    let bytes = wat.encode().unwrap();

    let idx = 0;
    let count = 1_048_576;
    let mut group = c.benchmark_group("fib");
    group.bench_function("stitch", |b| {
        use makepad_stitch::Val;

        let (mut store, instance) = new_stitch_store_and_instance(&bytes);
        let memory = instance.exported_mem("memory").unwrap();
        let sum = instance.exported_func("sum").unwrap();

        for (idx, byte) in &mut memory.bytes_mut(&mut store)[..count].iter_mut().enumerate() {
            let val = (idx % 256) as u8;
            *byte = val;
        }
        b.iter(|| {
            sum.call(
                &mut store,
                &[Val::I32(idx as i32), Val::I32(count as i32)],
                &mut [Val::I64(0)],
            )
            .unwrap();
        });
    });
    group.bench_function("wasmi", |b| {
        use wasmi::Value;

        let (mut store, instance) = new_wasmi_store_and_instance(&bytes);
        let memory = instance.get_memory(&store, "memory").unwrap();
        let sum = instance.get_func(&store, "sum").unwrap();

        for (idx, byte) in &mut memory.data_mut(&mut store)[..count].iter_mut().enumerate() {
            let val = (idx % 256) as u8;
            *byte = val;
        }
        b.iter(|| {
            sum.call(
                &mut store,
                &[Value::I32(idx as i32), Value::I32(count as i32)],
                &mut [Value::I64(0)],
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, fac, fib, fill, sum);
criterion_main!(benches);