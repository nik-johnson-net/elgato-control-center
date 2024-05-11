use std::time::Duration;

use anyhow::Result;
use reqwest_websocket::{RequestBuilderExt, WebSocket};
use serde::{Deserialize, Serialize};

use crate::jrpc::Jrpc;

const DEFAULT_URL: &str = "ws://127.0.0.1:1804/";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    pub firmware_version: String,
    pub firmware_version_build: i32,
    pub name: String,
    pub r#type: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfiguration {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    pub lights: Lights,
}

impl DeviceConfiguration {
    pub fn modify(&self) -> SetDeviceConfiguration {
        SetDeviceConfiguration {
            device_id: self.device_id.clone(),
            lights: self.lights.modify(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lights {
    pub brightness: u16,
    pub brightness_max: u16,
    pub brightness_min: u16,
    pub on: bool,
    pub temperature: u16,
    pub temperature_max: u16,
    pub temperature_min: u16,
}

impl Lights {
    pub fn modify(&self) -> SetLights {
        SetLights {
            brightness: self.brightness,
            on: self.on,
            temperature: self.temperature,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetDeviceConfiguration {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    pub lights: SetLights,
}

impl SetDeviceConfiguration {
    pub fn set_on(mut self, on: bool) -> Self {
        self.lights.on = on;
        self
    }

    pub fn set_brightness(mut self, brightness: u16) -> Self {
        self.lights.brightness = brightness;
        self
    }

    pub fn set_temperature(mut self, temperature: u16) -> Self {
        self.lights.temperature = temperature;
        self
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetLights {
    pub brightness: u16,
    pub on: bool,
    pub temperature: u16,
}

pub struct ControlCenter {
    connection: Jrpc,
}

impl ControlCenter {
    pub async fn connect() -> Result<ControlCenter> {
        ControlCenter::connect_url(DEFAULT_URL).await
    }

    pub async fn connect_url<T: AsRef<str>>(url: T) -> Result<ControlCenter> {
        let response = reqwest::Client::default()
            .get(url.as_ref())
            .timeout(Duration::from_secs(2))
            .upgrade()
            .send()
            .await?;

        let websocket: WebSocket = response.into_websocket().await?;

        Ok(ControlCenter {
            connection: Jrpc::handle(websocket),
        })
    }

    pub async fn devices(&self) -> Result<Vec<Device>> {
        self.connection.send("getDevices", None).await
    }

    pub async fn device_configuration<T: AsRef<str>>(&self, id: T) -> Result<DeviceConfiguration> {
        let param = serde_json::json!({
          "deviceID": id.as_ref(),
        });

        self.connection
            .send("getDeviceConfiguration", Some(param))
            .await
    }

    pub async fn set_device_configuration(&self, device: SetDeviceConfiguration) -> Result<()> {
        let param = serde_json::to_value(device)?;

        self.connection
            .send("setDeviceConfiguration", Some(param))
            .await
    }
}
