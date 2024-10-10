use lambda_runtime::{run, service_fn, Error, LambdaEvent};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    run(func).await
}

async fn func(_: LambdaEvent<serde_json::Value>) -> Result<String, String> {
    let mut out = Vec::new();
    annual_events::make_calendar(&mut out).unwrap();
    String::from_utf8(out)
        .map_err(|err| format!("Failed to parse calendar output as UTF-8: {:?}", err))
}
