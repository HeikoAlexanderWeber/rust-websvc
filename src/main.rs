use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    id: String,
}

struct DataService {
}
impl DataService {
    fn new() -> Self{
        DataService{}
    }

    fn get(&self) -> Data {
        Data {
            id: "bananarama".to_owned(),
        }
    }
}

/// This handler uses json extractor with limit
async fn index(svc: web::Data<DataService>, req: HttpRequest) -> impl actix_web::Responder {
    println!("{:?}", req);
    HttpResponse::Ok().json(svc.get()) // <- send json response
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
        let sys = actix_rt::System::new("https-server");
        let srv = HttpServer::new(|| {
            let svc = DataService::new();
            App::new()
                // enable logger
                .wrap(middleware::Logger::default())
                .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
                .data(svc)
                .service(
                    web::resource("/data")
                        .data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (resource level)
                        .route(web::get().to(index)),
                )
        })
        .workers(num_workers)
        .shutdown_timeout(shutdown_timeout)
        .bind_openssl(bind_address, builder)?
        .run();

        let _ = srv_sender.send(srv);
        sys.run()
    });

    let srv = srv_receiver.recv().unwrap();
    let _ = srv.await;

    Ok(())
}
