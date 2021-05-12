use lambda_runtime::{handler_fn, run, Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    run(func).await
}

async fn func(_: serde_json::Value, _: Context) -> Result<String, std::string::FromUtf8Error> {
    let mut out = Vec::new();
    annual_events::make_calendar(&mut out).unwrap();
    String::from_utf8(out)
}
