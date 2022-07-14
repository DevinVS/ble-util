use std::env::args;
use std::time::Duration;
use std::io::stdin;

use btleplug::{platform::Manager, api::{Manager as _, ScanFilter, Central, Peripheral, WriteType}};
use tokio::time;
use std::error::Error;

static HELP_MSG: &'static str = r###"ble-util v0.1
Devin Vander Stelt <devin@vstelt.dev>

Usage:
    ble-util <command> <args>

Commands:
    scan                scan for and print nearby devices
    ping <addr>         connect to device and print its services and characteristics
    read <addr> <char>  connect to the device and read the value of the characteristic
    write <addr>        connect to the device and write a value to the characteristic via stdin
    help                print this help message
"###;

static CHAR_WRITE: &'static str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
static CHAR_READ: &'static str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    if args.get(1).is_none() {
        eprintln!("No command specified\n");
        help();
        return Ok(());
    }

    match args[1].as_str() {
        "scan" => scan_devices().await?,
        "ping" => {
            if args.get(2).is_none() {
                eprintln!("No address specified\n");
                help();
                return Ok(());
            }

            ping(&args[2]).await?;
        }
        "read" => {
            if args.get(2).is_none() {
                eprintln!("No address specified\n");
                help();
                return Ok(());
            }

            if args.get(3).is_none() {
                eprintln!("No characteristic uuid specified\n");
                help();
                return Ok(());
            }

            read(&args[2], &args[3]).await?;
        },
        "write" => {
            if args.get(2).is_none() {
                eprintln!("No address specified\n");
                help();
                return Ok(());
            }

            write(&args[2]).await?;
        },
        "help" => help(),
        _ => {
            eprintln!("Unrecognized command '{}'\n", args[1]);
            help();
        }
    }

    Ok(())
}

async fn scan_devices() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(3)).await;

    for peripheral in central.peripherals().await?.iter() {
        let props = peripheral.properties().await?.unwrap();
        println!("{}: {}", props.address, props.local_name.unwrap_or("Unknown".into()));
    }

    Ok(())
}

async fn ping(addr: &str) -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(3)).await;

    let mut dev = None;
    for p in central.peripherals().await? {
        if p.properties().await?.unwrap().address.to_string().eq(addr) {
            dev = Some(p);
        }
    }

    if dev.is_none() {
        eprintln!("Unable to find device");
        return Ok(());
    }

    let dev = dev.unwrap();
    dev.connect().await?;
    println!("Connected");
    dev.discover_services().await?;

    // Print out the device servers and characteristics
    println!("Services:");
    for s in dev.services() {
        println!("{}:", s.uuid);

        for c in s.characteristics.iter() {
            println!("\t{}: {:?}", c.uuid, c.properties);
        }
    }

    Ok(())
}

async fn read(addr: &str, char_id: &str) -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(3)).await;

    let mut dev = None;
    for p in central.peripherals().await? {
        if p.properties().await?.unwrap().address.to_string().eq(addr) {
            dev = Some(p);
        }
    }

    if dev.is_none() {
        eprintln!("Unable to find device");
        return Ok(());
    }

    let dev = dev.unwrap();
    dev.connect().await?;

    println!("Connected");
    dev.discover_services().await?;

    let chars = dev.characteristics();
    let ch = chars.iter()
        .find(|a| a.uuid.to_string().eq(char_id))
        .unwrap();

    let res = dev.read(ch).await?;
    println!("{:?}", res);
    Ok(())
}

async fn write(addr: &str) -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let mut dev = None;
    for p in central.peripherals().await? {
        if p.properties().await?.unwrap().address.to_string().eq(addr) {
            dev = Some(p);
        }
    }

    if dev.is_none() {
        eprintln!("Unable to find device");
        return Ok(());
    }

    let dev = dev.unwrap();

    dev.connect().await?;

    println!("Connected");
    dev.discover_services().await?;

    let chars = dev.characteristics();
    let ch_write = chars.iter()
        .find(|a| a.uuid.to_string().eq(CHAR_WRITE))
        .unwrap();

    let ch_read = chars.iter()
        .find(|a| a.uuid.to_string().eq(CHAR_READ))
        .unwrap();

    let mut buf = String::new();
    while let Ok(_) = stdin().read_line(&mut buf) {
        let res = dev.write(ch_write, buf.trim().as_bytes(), WriteType::WithoutResponse).await?;
        println!("{:?}", res);

        let res = dev.read(ch_read).await?;
        println!("{:?}", res);
    }


    Ok(())
}

fn help() {
    eprintln!("{}", HELP_MSG);
}
