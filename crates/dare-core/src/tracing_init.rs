/// Instala subscriber fmt para testes. Idempotente o suficiente para testes unitários.
pub fn init_test_subscriber() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}
