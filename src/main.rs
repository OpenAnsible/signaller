#![feature(box_syntax)]
#![allow(dead_code)]
#[warn(unused_attributes)]
extern crate hyper;

mod jsonrpc;
use jsonrpc::{ JsonRpc, RpcResult, RpcRequest, 
    json, ToJson };

use std::io::{ copy, Read, Write };
use std::sync::{ Arc, Mutex };
use std::env;

use hyper::server::{ Server, Request, Response };
use hyper::method::Method::{ Get, Put, Post };
use hyper::status::StatusCode; // { Ok, BadRequest, NotFound, MethodNotAllowed };


// TODO: 给 RPC Method 增加 Share Memeory 支持.
//          fn hello (sm: ShareMemoy , params: json::Json) -> RpcResult { }
fn hello (params: &Option<json::Json>) -> RpcResult {
    match params.as_ref() {
        Some(p) => {
            println!("Params: {:?}", p );
        },
        None => {
            println!("Params: Null" );
        }
    };
    
    Ok("Hello World".to_json())
}

fn ice (params: &Option<json::Json>) -> RpcResult {
    match params.as_ref() {
        Some(p) => {
            println!("Params: {:?}", p );
        },
        None => {
            println!("Params: Null" );
        }
    };
    let data = vec![1,2,3,4];
    Ok( data.to_json() )
}

fn test_rpc() {
    let mut rpc = JsonRpc::new();
    rpc.add_method("hello", Box::new(hello));
    rpc.add_method("ice", Box::new(ice));
    let request1 = "{\"params\": [\"参数1\", \"param 2\"], \"jsonrpc\": \"2.0\", \"method\": \"ice\", \"id\": 1}";
    match RpcRequest::new(&request1) {
        Ok(rpc_request) => {
            println!("{}", rpc.call(&rpc_request).to_json().to_string() );
        },
        Err(e) => {
            println!("Error:   {:?}", e);
        }
    }
}


fn main (){
    let args: Vec<String> = env::args().collect();
    println!("Args: {:?}", args);

    // let mut rpc = JsonRpc::new();
    // rpc.add_method("hello", Box::new(hello));
    // rpc.add_method("ice", Box::new(ice));
    // let share_rpc = Arc::new(Mutex::new(rpc));

    Server::http("0.0.0.0:80").unwrap().handle(move |mut req: Request, mut res: Response| {
        match req.method {
            Post | Put => {
                println!("{:?}", req.headers);
                let mut rpc = JsonRpc::new();
                rpc.add_method("hello", Box::new(hello));
                rpc.add_method("ice", Box::new(ice));

                let mut body = String::new();
                match req.read_to_string(&mut body){
                    Ok(body_length) => {
                        println!("Body length: {:?}\n{:?}", body_length, body);
                        let mut res = &mut res.start().unwrap();
                        match RpcRequest::new(&body) {
                            Ok(rpc_request) => {
                                let response_content = rpc.call(&rpc_request).to_json().to_string();
                                res.write(response_content.as_bytes()).unwrap();
                            },
                            Err(rpc_error) => {
                                println!("Error:   {:?}", rpc_error);
                                res.write(rpc_error.to_json().to_string().as_bytes()).unwrap();

                                // *res.status_mut() = StatusCode::MethodNotAllowed;
                            }
                        }
                        // res.status_mut() = &mut StatusCode::Ok;
                    },
                    Err(e) => {
                        println!("Bad Request. {:?}", e);
                        let mut res = &mut res.start().unwrap();
                        // res.status_mut() = StatusCode::BadRequest;
                        res.write(e.to_string().as_bytes()).unwrap();
                    }
                }
            },
            Get => {
                println!("{:?}", req.headers);
                copy(&mut req, &mut res.start().unwrap()).unwrap();  
            },
            _ => {
                println!("Method Not Allowed.");
                *res.status_mut() = StatusCode::MethodNotAllowed;
            }
        };
    }).unwrap();
}