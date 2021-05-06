/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
mod api;
mod component;
mod display;
mod error;
mod pipeline;
mod vc;

use pipeline::*;
use vc::*;

use getopts::Options;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::exit;
use std::sync::{Arc, Mutex};

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install signal handler");
}

fn parse_opts() -> SocketAddr {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("H", "host", "address to bind (default: 127.0.0.1)", "ADDR");
    opts.optopt("P", "port", "port to listen (default: 3000)", "NUM");
    opts.optflag("h", "help", "print this help");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!(
            "{}",
            opts.usage(format!("{}\nUsage: {} [options]", e, program).as_str())
        );
        exit(1);
    });
    if matches.opt_present("h") {
        eprintln!(
            "{}",
            opts.usage(format!("Usage: {} [options]", program).as_str())
        );
        exit(0);
    }

    let addr = matches.opt_str("H").unwrap_or("127.0.0.1".to_string());
    let port = matches.opt_str("P").unwrap_or("3000".to_string());

    format!("{}:{}", addr, port)
        .parse()
        .unwrap_or(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            3000,
        ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = parse_opts();
    omx::init();

    let (width, height) = omx::get_display_size(0);
    let pipeline = Arc::new(Mutex::new(Pipeline::new(width, height)));
    pipeline.lock().unwrap().init().unwrap();

    let service_pipeline = pipeline.clone();
    let service = make_service_fn(move |_| {
        let pipeline = service_pipeline.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| api::handler(req, pipeline.clone()))) }
    });

    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }

    pipeline.lock().unwrap().destroy().unwrap();
    omx::deinit();
    println!("See you!");
    Ok(())
}
