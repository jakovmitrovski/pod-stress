use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_ec2::Client as Ec2Client;

async fn shutdown_instances(client: &Ec2Client) -> Result<(), eyre::Error> {
    let describe_result = client.describe_instances().send().await?;

    let mut instance_ids = Vec::new();

    for reservation in describe_result.reservations() {
        for instance in reservation.instances() {
            if let Some(id) = instance.instance_id() {
                if instance.state().map(|s| s.name().unwrap().as_str()) == Some("running") {
                    instance_ids.push(id.to_string());
                }
            }
        }
    }

    if instance_ids.is_empty() {
        println!("No running instances found");
        return Ok(());
    }

    println!("Terminating {} instances...", instance_ids.len());
    for id in &instance_ids {
        client.terminate_instances().instance_ids(id).send().await?;
    }

    println!("All instances have been terminated");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    let region_provider = RegionProviderChain::first_try(Region::new("eu-central-1".to_string()));
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Ec2Client::new(&config);

    shutdown_instances(&client).await?;

    Ok(())
}
