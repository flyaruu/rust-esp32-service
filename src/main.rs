use std::thread;
use std::{sync::Arc, time::Duration};

use anyhow::{bail};
use embedded_svc::wifi::*;
use embedded_svc::ping::*;
use embedded_svc::ipv4;
use esp_idf_hal::prelude::Peripherals;
use log::{info, warn};
use stepper::embedded_hal::digital::blocking::OutputPin;
use std::iter::Iterator;
use std::sync::{Mutex,Condvar};
use esp_idf_svc::netif::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::wifi::*;
use esp_idf_svc::ping::*;
use esp_idf_svc::http::server::{EspHttpRequest,EspHttpResponse};
use esp_idf_hal::gpio::{Gpio1,Gpio2,Gpio3,Gpio4,Gpio5,Output};

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

use embedded_svc::errors::wrap::WrapError;
use embedded_svc::http::server::registry::Registry;
use embedded_svc::http::server::Response;
use embedded_svc::http::SendStatus;
use embedded_svc::http::server::Request;


mod curtain_stepper;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    #[allow(unused)]
    let peripherals = Peripherals::take().unwrap();
    #[allow(unused)]
    let pins = peripherals.pins;
    // let ledpin = pins.gpio19.into_output().unwrap();

    let duration: Arc<Mutex<u64>> = Arc::new(Mutex::new(5));
    println!("got pins");
    let ledpin = pins.gpio1.into_output().unwrap();

    let pin_state = Arc::new(Mutex::new(ledpin));
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let _wifi = wifi(netif_stack, sys_loop_stack, default_nvs)?;
    println!("wifi done");

    let mutex: Arc<(Mutex<Option<u32>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));
    warn!("mutex running");
    
    // Default::default()
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;
    warn!("HTTPD constructed");

    let mut stepper1 = pins.gpio2.into_output().unwrap();
    let mut stepper2 = pins.gpio3.into_output().unwrap();
    let mut stepper3 = pins.gpio4.into_output().unwrap();
    let mut stepper4 = pins.gpio5.into_output().unwrap();
    httpd(&pin_state, &mut server,duration.clone())?;
    println!("HTTPD running");
    let mut wait = mutex.0.lock().unwrap();
    println!("wait constructed");

    let mut phase: u8 = 0;
    #[allow(unused)]
    let cycles = loop {
        phase = (phase + 1) % 8;
        stepper(phase,&mut stepper1,&mut stepper2,&mut stepper3, &mut stepper4);
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            let my_duration = *duration.lock().unwrap();
            wait = mutex
                .1
                .wait_timeout(wait, Duration::from_millis(my_duration   ))
                .unwrap()
                .0;

        }
    };

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
            thread::sleep(Duration::from_secs(1));
    }

    drop(server);
    info!("Httpd stopped");
    drop(wifi);
    info!("Wifi stopped");
    Ok(())
}


fn stepper(
    phase: u8,
    pin1: &mut Gpio2<Output>,
    pin2: &mut Gpio3<Output>,
    pin3: &mut Gpio4<Output>,
    pin4: &mut Gpio5<Output>
)-> anyhow::Result<()> {
    match phase {
        0 => {
            pin1.set_high()?;
            pin2.set_low()?;
            pin3.set_low()?;
            pin4.set_low()?;
        },
        1 => {
            pin1.set_high()?;
            pin2.set_high()?;
            pin3.set_low()?;
            pin4.set_low()?;
        },
        2 => {
            pin1.set_low()?;
            pin2.set_high()?;
            pin3.set_low()?;
            pin4.set_low()?; //
        },
        3 => {
            pin1.set_low()?;
            pin2.set_high()?;
            pin3.set_high()?;
            pin4.set_low()?;
        },
        4 => {
            pin1.set_low()?;
            pin2.set_low()?;
            pin3.set_high()?;
            pin4.set_low()?;
        },
        5 => {
            pin1.set_low()?;
            pin2.set_low()?;
            pin3.set_high()?;
            pin4.set_high()?;
        },
        6 => {
            pin1.set_low()?;
            pin2.set_low()?;
            pin3.set_low()?;
            pin4.set_high()?;
        },
        7 => {
            pin1.set_high()?;
            pin2.set_low()?;
            pin3.set_low()?;
            pin4.set_high()?;
        },        
        _ => {
            pin1.set_low()?;
            pin2.set_low()?;
            pin3.set_low()?;
            pin4.set_low()?;
        }
    }
    Ok(())
}

#[allow(unused_variables)]
#[cfg(feature = "experimental")]
fn httpd<'a>(
    pin_state:&Arc<Mutex<Gpio1<Output>>>,
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    duration: Arc<Mutex<u64>>

) -> anyhow::Result<()> {

    let state_ref = Arc::clone(&pin_state);
    let state_ref2 = Arc::clone(&pin_state);
    // let mut dur = Arc::clone(&self)
    server
        .handle_get("/on", move|_req,resp| {
            resp.send_str("led on!")?;
            // ledbox.lock().unwrap().set_high().unwrap();
            let mut mut_state = state_ref.lock().unwrap();
            mut_state.set_high().unwrap();

            Ok(())
        })?
        .handle_get("/off",  move|_req,resp| {
            resp.send_str("led off!")?;
            // let state_ref = Arc::clone(&pin_state);
            let mut mut_state = state_ref2.lock().unwrap();
            mut_state.set_low().unwrap();
            Ok(())
        })?
        .handle_get("/duration",  move|req:EspHttpRequest,resp:EspHttpResponse| {
            let query = req.query_string();
            let mut res = duration.lock().unwrap();
            // let state_ref = Arc::clone(&pin_state);

            match query.parse::<u64>() {
                Ok(nr) => {*res = nr;
                    let result_str = format!("updated duration: {}",query);
                    resp.send_str(&result_str)?;

                },
                Err(_) => {
                    let result_str = format!("illegal duration: {}",query);
                    resp.status(500).send_str(&result_str)?;
            }
                } 
            Ok(())
        })?
        .handle_get("/foo", |_req, resp| {
            Result::Err(WrapError("Boo, something happened!").into())
        })?
        .handle_get("/bar", |_req, resp| {
            resp.status(403)
                .status_message("No permissions")
                .send_str("You have no permissions to access this page")?;

            Ok(())
        })?
        .handle_get("/panic", |_req, _resp| panic!("User requested a panic!"))?;
    Ok(())
}


fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");
    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    let client_config = ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        channel,
        ..Default::default()
    }; 

    wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(client_config))?;

    info!("Wifi configuration set, about to get status");

    let intermediate_status = wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional());

    warn!("Interm status: {:?}",intermediate_status);
    intermediate_status.map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;


    let status = wifi.get_status();
    info!("Status: {:?}",status);
    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Stopped,
    ) = status
    {
        info!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}


fn ping(ip_settings: &ipv4::ClientSettings) -> anyhow::Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

    Ok(())
}

fn test_https_client() -> anyhow::Result<()> {
    use embedded_svc::http::{client::*};
    use embedded_svc::io;
    use esp_idf_svc::http::client::*;

    let url = String::from("https://google.com");

    info!("About to fetch content from {}", url);

    let mut client = EspHttpClient::new(&EspHttpClientConfiguration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    })?;

    let mut response = client.get(&url)?.submit()?;

    let mut body = [0_u8; 3048];

    let (body, _) = io::read_max(response.reader(), &mut body)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(body).into_owned()
    );

    Ok(())
}

