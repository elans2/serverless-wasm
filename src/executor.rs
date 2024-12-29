use wasmtime::*;
use serde_json::Value;

pub fn execute(code: &str, function_name: &str, inputs: &[Value]) -> Result<Value, Box<dyn std::error::Error>> {
    let engine = Engine::default();
    let module = Module::new(&engine, code)?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    let func = instance.get_func(&mut store, function_name)
        .ok_or_else(|| format!("Function '{}' not found in module", function_name))?;

    let func_ty = func.ty(&store);
    let params: Vec<_> = func_ty.params().collect();
    let results: Vec<_> = func_ty.results().collect();

    if params.len() != inputs.len() {
        return Err(format!(
            "Function '{}' expected {} arguments, but got {}",
            function_name, params.len(), inputs.len()
        ).into());
    }

    let mut wasm_inputs = Vec::new();

    for (param, input) in params.iter().zip(inputs.iter()) {
        let value = match (param, input) {
            (ValType::I32, Value::Number(n)) => Val::I32(n.as_i64().ok_or("Invalid i32")? as i32),
            _ => return Err(format!("Unsupported parameter type: {:?}", param).into()),
        };
        wasm_inputs.push(value);
    }

    let mut wasm_results = vec![Val::I32(0); results.len()];
    func.call(&mut store, &wasm_inputs, &mut wasm_results)?;

    if wasm_results.len() > 1 {
        return Err("Multiple return values are not supported yet".into());
    }

    let result = match wasm_results.get(0) {
        Some(Val::I32(v)) => Value::Number((*v).into()),
        _ => return Err("Unsupported return type".into()),
    };

    Ok(result)
}
