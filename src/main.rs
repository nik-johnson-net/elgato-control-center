use anyhow::anyhow;
use clap::{Parser, Subcommand};
use controlcenter::{ControlCenter, DeviceConfiguration, SetDeviceConfiguration};

mod controlcenter;
mod jrpc;

#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    url: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Devices,
    On {
        device: Option<String>,
    },
    Off {
        device: Option<String>,
    },
    SetBrightness {
        brightness: u16,
        device: Option<String>,
    },
    SetTemperature {
        temperature: u16,
        device: Option<String>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let cc = if let Some(url) = args.url {
        ControlCenter::connect_url(url).await?
    } else {
        ControlCenter::connect().await?
    };

    match args.command {
        Commands::Devices => {
            let devices = cc.devices().await?;
            println!("id,name");
            for device in devices {
                println!("{},{}", device.device_id, device.name);
            }
        }
        Commands::On { device } => {
            modify_device_or_all(&cc, device, |config| config.modify().set_on(true)).await?;
        }
        Commands::Off { device } => {
            modify_device_or_all(&cc, device, |config| config.modify().set_on(false)).await?;
        }
        Commands::SetBrightness { brightness, device } => {
            modify_device_or_all(&cc, device, |config| {
                config.modify().set_brightness(brightness)
            })
            .await?;
        }
        Commands::SetTemperature {
            temperature,
            device,
        } => {
            modify_device_or_all(&cc, device, |config: DeviceConfiguration| {
                config.modify().set_temperature(temperature)
            })
            .await?;
        }
    };

    Ok(())
}

async fn modify_device_or_all<F: Fn(DeviceConfiguration) -> SetDeviceConfiguration>(
    cc: &ControlCenter,
    device: Option<String>,
    function: F,
) -> Result<(), anyhow::Error> {
    let devices = cc.devices().await?;

    if let Some(device) = device {
        let found = devices
            .into_iter()
            .find(|item| item.device_id == device || item.name == device);
        if let Some(found) = found {
            let config = cc.device_configuration(found.device_id).await?;
            cc.set_device_configuration(function(config)).await?;
        } else {
            return Err(anyhow!("Device \"{}\" not found.", device));
        }
    } else {
        for device in devices {
            let config = cc.device_configuration(device.device_id).await?;
            cc.set_device_configuration(function(config)).await?;
        }
    }

    Ok(())
}
