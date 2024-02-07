use std::time::Duration;

use log::{error, trace};
use protobuf::Message;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;
mod sealed {
    pub trait Sealed {}
}
mod pb;
#[derive(Deserialize, Debug)]
struct VmConfig {
    cluster: String,
}

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot{
        header_content:String::new(),
    }) });
}}
struct HttpHeaders {
    context_id: u32,
}
struct HttpHeadersRoot {
    header_content: String,
}
static mut VM_CONFIG_GLBOAL: VmConfig = VmConfig {
    cluster: String::new(),
};

impl Context for HttpHeaders {
    fn on_grpc_call_response(&mut self, _token_id: u32, _status_code: u32, response_size: usize) {
        if let Some(body) = self.get_grpc_call_response_body(0, response_size) {
            let mut hp = pb::hello::HelloReply::new();
            match hp.merge_from_bytes(body.as_slice()) {
                Ok(_) => {
                    let msg = hp.get_message();
                    error!("{}", msg);
                    self.set_http_request_header("hello", Some(&msg.to_string()));
                }
                Err(e) => {
                    error!("{}", e)
                }
            }
        }
        self.resume_http_request();
    }
}
impl Context for HttpHeadersRoot {}
impl RootContext for HttpHeadersRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        let vm_config = self.get_vm_configuration();
        match vm_config {
            Some(v) => unsafe {
                let d = v.as_slice();
                let result: Result<VmConfig, serde_json::Error> = serde_json::from_slice(d);
                if let Ok(vm_json) = result {
                    VM_CONFIG_GLBOAL = vm_json;
                }
            },
            None => {}
        }
        true
    }

    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.header_content = String::from_utf8(config_bytes).unwrap()
        }
        true
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders { context_id }))
    }
}
impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let path = self.get_http_request_header(":path").unwrap_or_default();
        let grpc_name: &str;
        if !path.contains("name") {
            return Action::Continue;
        }
        let uparms_arr: Vec<&str> = path.split("name=").collect();
        if uparms_arr[1].contains("&") {
            let real_uparms: Vec<&str> = uparms_arr[1].split("&").collect();
            grpc_name = real_uparms[0];
        } else {
            grpc_name = uparms_arr[1]
        }
        if grpc_name.is_empty() {
            return Action::Continue;
        }
        let mut req = pb::hello::HelloRequest::new();
        req.set_name(grpc_name.to_string());
        let message = req.write_to_bytes().unwrap();
        unsafe {
            match self.dispatch_grpc_call(
                &VM_CONFIG_GLBOAL.cluster,
                "hello.HelloService",
                "Hello",
                Vec::<(&str, &[u8])>::new(),
                Some(message.as_slice()),
                Duration::from_millis(20),
            ) {
                Ok(_) => error!("grpc send success"),
                Err(e) => error!("grpc send error:{:?}", e),
            }
        }
        Action::Pause
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        Action::Continue
    }

    fn on_log(&mut self) {
        trace!("#{} completed.", self.context_id);
    }
}
