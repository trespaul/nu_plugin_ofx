mod ofx_to_nu;

use nu_plugin::{EngineInterface, Plugin, PluginCommand, SimplePluginCommand};
use nu_plugin::{EvaluatedCall, MsgPackSerializer, serve_plugin};
use nu_protocol::{LabeledError, Signature, Type, Value};

use crate::ofx_to_nu::convert_document;

struct OfxPlugin;

impl Plugin for OfxPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(FromOfx)]
    }
}

struct FromOfx;

impl SimplePluginCommand for FromOfx {
    type Plugin = OfxPlugin;

    fn name(&self) -> &str {
        "from ofx"
    }

    fn description(&self) -> &str {
        "Parse text as Open Financial Exchange data, both v1 (SGML) and v2 (XML)."
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .category(nu_protocol::Category::Formats)
            .input_output_type(Type::String, Type::Any)
    }

    fn run(
        &self,
        _plugin: &OfxPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let span = input.span();
        match input {
            Value::String { val, .. } => ofx_rs::parse(val)
                .map(|doc| convert_document(&doc, span))
                .map_err(|e| LabeledError::new(format!("Failed to parse OFX: {}", e))),
            _ => Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            ),
        }
    }
}

fn main() {
    serve_plugin(&OfxPlugin, MsgPackSerializer)
}
