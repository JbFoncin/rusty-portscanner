use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::net::TcpStream;
use tokio::time::{timeout};
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn connect_to_addr(ip_to_scan: IpAddr, port: u16, sem_arc: Arc<Semaphore>) -> Option<u16> {
    let _permit: tokio::sync::SemaphorePermit<'_> = sem_arc.acquire().await.unwrap();
    let socket_addr: SocketAddr = SocketAddr::new(ip_to_scan, port);
        if let Ok(_stream) = TcpStream::connect(&socket_addr).await {
            Some(port.clone())
        } else {
            //println!("connection to {} with port {} failed", &ip_to_scan, &port);
            None
        }
}

async fn loop_over_all_values(ip_addr: &str) -> Vec<u16> {
    let mut open_ports: Vec<u16> = Vec::with_capacity(1000);
    let mut tasks: Vec<JoinHandle<Option<u16>>> = Vec::with_capacity(65535);
    let ip_to_scan: IpAddr = IpAddr::from_str(&ip_addr).unwrap();
    let sem_arc = Arc::new(Semaphore::new(500));
    for port in 0..=65535_u16 {
        let timeout_duration: Duration = std::time::Duration::from_millis(100000); //big timeout because most servers restrict too much connexions        
        let fut = timeout(timeout_duration, connect_to_addr(ip_to_scan, port, sem_arc.clone()));
        let task: JoinHandle<Option<u16>> = tokio::task::spawn(async move { if let Ok(result) = fut.await { result }
                                                                else { None } });
        tasks.push(task);
    }
    for task in tasks {
        if let Some(port) = task.await.unwrap() { open_ports.push(port); }
    }
    open_ports
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ip_to_scan: String = args[1].clone();
    let open_ports: Vec<u16> = loop_over_all_values(&ip_to_scan).await;
    println!("{:?}", &open_ports)
}