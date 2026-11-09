// Jackson Coxson

use idevice::{
    IdeviceService, RsdService, core_device::LocationServiceClient,
    core_device_proxy::CoreDeviceProxy, provider::IdeviceProvider, rsd::RsdHandshake,
};
use jkcli::{CollectedArguments, JkArgument, JkCommand};

pub fn register() -> JkCommand {
    JkCommand::new()
        .help("Inspect the device's CoreDevice location-simulation service")
        .with_subcommand(
            "available-scenarios",
            JkCommand::new().help("List the device's built-in location-simulation scenarios"),
        )
        .with_subcommand(
            "set",
            JkCommand::new()
                .help("Simulate the device's location")
                .with_argument(JkArgument::new().with_help("latitude").required(true))
                .with_argument(JkArgument::new().with_help("longitude").required(true)),
        )
        .with_subcommand(
            "clear",
            JkCommand::new().help("Stop simulating the location"),
        )
        .subcommand_required(true)
}

pub async fn main(arguments: &CollectedArguments, provider: Box<dyn IdeviceProvider>) {
    let (sub_name, sub_args) = arguments
        .first_subcommand()
        .expect("no subcommand passed, pass -h for help");

    let proxy = CoreDeviceProxy::connect(&*provider)
        .await
        .expect("no core device proxy");
    let rsd_port = proxy.tunnel_info().server_rsd_port;
    let adapter = proxy.create_software_tunnel().expect("no software tunnel");
    let mut adapter = adapter.to_async_handle();
    let stream = adapter.connect(rsd_port).await.expect("no RSD connect");
    let mut handshake = RsdHandshake::new(stream).await.unwrap();

    let mut client = LocationServiceClient::connect_rsd(&mut adapter, &mut handshake)
        .await
        .expect("no locationservice");

    match sub_name.as_str() {
        "available-scenarios" => {
            let scenarios = client
                .available_location_scenarios()
                .await
                .expect("availableLocationScenarios failed");
            for s in scenarios {
                if s.name == s.localized_name {
                    println!("{}", s.name);
                } else {
                    println!("{} ({})", s.localized_name, s.name);
                }
            }
        }
        "set" => {
            let mut sub_args = sub_args.clone();
            let latitude: f64 = sub_args
                .next_argument::<String>()
                .expect("No latitude passed! Pass -h for help")
                .parse()
                .expect("Failed to parse latitude as float");
            let longitude: f64 = sub_args
                .next_argument::<String>()
                .expect("No longitude passed! Pass -h for help")
                .parse()
                .expect("Failed to parse longitude as float");
            client
                .set_simulated_location(latitude, longitude)
                .await
                .expect("setsimulatedlocation failed");
            println!("Location set!");
        }
        "clear" => {
            client
                .clear_simulated_location()
                .await
                .expect("clearsimulatedlocation failed");
            println!("Location cleared!");
        }
        _ => unreachable!(),
    }
}
