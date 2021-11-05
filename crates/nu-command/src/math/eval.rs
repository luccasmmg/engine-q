use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Example, SyntaxShape, PipelineData, ShellError, Spanned, Signature, Span, Value};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "math eval"
    }

    fn usage(&self) -> &str {
        "Evaluate a math expression into a number"
    }

    fn signature(&self) -> Signature {
        Signature::build("math eval").optional(
            "math expression",
            SyntaxShape::String,
            "the math expression to evaluate",
        )
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let spanned_expr: Option<Spanned<String>> = call.opt(engine_state, stack, 0)?;
        eval(spanned_expr, input, engine_state)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Evalulate math in the pipeline",
            example: "'10 / 4' | math eval",
            result: Some(Value::Float {
                val: 2.5,
                span: Span::unknown(),
            }),
        }]
    }
}

pub fn eval(spanned_expr: Option<Spanned<String>>, input: PipelineData, engine_state: &EngineState) -> Result<PipelineData, ShellError> {
    if let Some(expr) = spanned_expr  {
        match parse(&expr.item, &expr.span) {
            Ok(value) => Ok(PipelineData::Value(value)),
            Err(err) => Err(ShellError::UnsupportedInput(
                format!("Math evaluation error: {}", err),
                expr.span,
            )),
        }
    } else {
        if let PipelineData::Value(Value::Nothing { .. }) = input {
            return Ok(input);
        }
        input.map(move |val| {
                if let Ok(string) = val.as_string() {
                    match parse(&string, &val.span().unwrap_or(Span::unknown())) {
                        Ok(value) => value,
                        Err(err) => Value::Error { error : ShellError::UnsupportedInput(
                                format!("Math evaluation error: {}", err),
                            val.span().unwrap_or(Span::unknown()))}
                    }
                } else {
                        Value::Error { error : ShellError::UnsupportedInput(
                                format!("Expected a string from pipeline"),
                            val.span().unwrap_or(Span::unknown()))}
                }
            }, engine_state.ctrlc.clone())
    }}

pub fn parse(math_expression: &str, span: &Span) -> Result<Value, String> {
    let mut ctx = meval::Context::new();
    ctx.var("tau", std::f64::consts::TAU);
    match meval::eval_str_with_context(math_expression, &ctx) {
        Ok(num) if num.is_infinite() || num.is_nan() => Err("cannot represent result".to_string()),
        Ok(num) => Ok(Value::Float { val: num, span: *span}),
        Err(error) => Err(error.to_string().to_lowercase()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
