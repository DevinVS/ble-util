use std::{env::args, time::Duration, io::{stdin, Read}};

use btleplug::{platform::Manager, api::{Manager as _, ScanFilter, Central, Peripheral, Characteristic, WriteType}};
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
    write <addr> <char> connect to the device and write a value to the characteristic via stdin
    help                print this help message
"###;

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

            if args.get(3).is_none() {
                eprintln!("No characteristic uuid specified\n");
                help();
                return Ok(());
            }

            let mut input = String::new();
            stdin().read_to_string(&mut input).unwrap();

            write(&args[2], &args[3], &input).await?;
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

async fn write(addr: &str, char_id: &str, msg: &str) -> Result<(), Box<dyn Error>> {
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

    dev.write(ch, msg.as_bytes(), WriteType::WithoutResponse).await?;

    Ok(())
}


fn help() {
    eprintln!("{}", HELP_MSG);
}
