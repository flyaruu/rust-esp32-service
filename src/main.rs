use std::{sync::Arc, time::Duration};

use anyhow::bail;
use embedded_svc::wifi::*;
use embedded_svc::ping::*;
use embedded_svc::ipv4;
//use embedded_svc::{ping::Ping,wifi::{AccessPointConfiguration, ClientStatus, ClientConnectionStatus, ClientIpStatus, ApStatus, ApIpStatus, Status}, ipv4};
//use esp_idf_svc::{netif::EspNetifStack,nvs::EspDefaultNvs, sysloop::EspSysLoopStack, ping,wifi::EspWifi};
use log::info;
use std::iter::Iterator;

use esp_idf_svc::netif::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::wifi::*;
use esp_idf_svc::ping::*;


const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");


fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let _a = 1;
    println!("Hello, world!");

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    
    wifi(netif_stack, sys_loop_stack, default_nvs)?;
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

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
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