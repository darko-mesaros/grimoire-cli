use anyhow::Result;
use serde_json::Value;

pub struct Output {
    json: bool,
    query: Option<String>,
}

impl Output {
    pub fn new(json: bool, query: Option<&str>) -> Self {
        Self { json, query: query.map(str::to_string) }
    }

    pub fn is_json(&self) -> bool {
        self.json
    }

    pub fn print_json(&self, value: &Value) -> Result<()> {
        let out = if let Some(expr) = &self.query {
            let expr = jmespath::compile(expr)
                .map_err(|e| anyhow::anyhow!("Invalid --query: {e}"))?;
            let data: jmespath::Variable = serde_json::from_value(value.clone())?;
            serde_json::to_value(&expr.search(data)?)?
        } else {
            value.clone()
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        Ok(())
    }

    pub fn error(&self, msg: &str) {
        if self.json {
            eprintln!("{}", serde_json::json!({ "error": msg }));
        } else {
            eprintln!("error: {msg}");
        }
    }
}
