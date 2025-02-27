use fj::{core::services::Services, handle_model};

fn main() -> fj::Result {
    let mut services = Services::new();
    let model = split::model(1.0, 0.2, &mut services);
    handle_model(model, services)?;
    Ok(())
}
