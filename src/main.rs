use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Data {
    id: String,
}

struct DataService {
    items: Vec<Data>
}

impl DataService {
    fn new() -> Self{
        DataService {
            items: vec!(),
        }
    }

    fn get(&self) -> Vec<Data> {
        self.items.clone()
    }

    fn post(&mut self, d: Data) {
        println!("{:?}", d);
        self.items.push(d);
    }
}

async fn get(svc: web::Data<Arc<Mutex<DataService>>>, req: HttpRequest) -> impl actix_web::Responder {
    println!("{:?}", req);
    HttpResponse::Ok().json(svc.lock().unwrap().get())
}

async fn post(svc: web::Data<Arc<Mutex<DataService>>>, data: web::Json<Data>, req: HttpRequest) -> impl actix_web::Responder {
    println!("{:?}", req);
    svc.lock().unwrap().post(data.into_inner());
    HttpResponse::Ok()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let key_file = std::env::var("SVC_KEY_FILE").unwrap();
    let cert_file = std::env::var("SVC_CERT_FILE").unwrap();
    let bind_address = std::env::var("SVC_BIND_ADDRESS").unwrap();
    let num_workers = std::env::var("SVC_NUM_WORKERS").unwrap().parse::<usize>().unwrap();
    let shutdown_timeout = std::env::var("SVC_SHUTDOWN_TIMEOUT").unwrap().parse::<u64>().unwrap();

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file(key_file, SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file(cert_file).unwrap();

    let (srv_sender, srv_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let datasvc = Arc::new(Mutex::new(DataService::new()));
        let sys = actix_rt::System::new("https-server");
        let srv = HttpServer::new(move || {
            App::new()
                // enable logger
                .wrap(middleware::Logger::default())
                .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
                .data(datasvc.clone())
                .service(web::resource("/data")
                    .route(web::post().to(post))
                    .route(web::get().to(get)))
        })
        .workers(num_workers)
        .shutdown_timeout(shutdown_timeout)
        .bind_openssl(bind_address, builder)?
        .run();

        let _ = srv_sender.send(srv);
        sys.run()
    });

    let (sigterm_sender, sigterm_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let signals = signal_hook::iterator::Signals::new(&[signal_hook::SIGINT, signal_hook::SIGTERM]).unwrap();
        for sig in signals.forever() {
            println!("Signal encountered: {:?}", sig);
            let _ = sigterm_sender.send(sig);
        }
    });

    let srv = srv_receiver.recv().unwrap();
    let _ = sigterm_receiver.recv().unwrap();
    println!("Termination signal received. Gracefully shutting down HTTP server.");
    let _ = srv.stop(true);
    let _ = srv.await;

    Ok(())
}
