use aws_config::meta::region::RegionProviderChain;
use aws_config::Region;
use aws_sdk_ec2::types::{IamInstanceProfileSpecification, InstanceType};
use aws_sdk_ec2::Client as Ec2Client;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::fs;

async fn launch_instances(client: &Ec2Client) -> Result<(), eyre::Error> {
    let user_data = fs::read_to_string("user_data.sh")?;
    let user_data_base64 = STANDARD.encode(user_data);

    let _ = client
        .run_instances()
        .image_id("ami-06ee6255945a96aba")
        .instance_type(InstanceType::T22xlarge)
        .min_count(1)
        .max_count(1)
        .user_data(user_data_base64)
        .key_name("jakov-ssh")
        .iam_instance_profile(
            IamInstanceProfileSpecification::builder()
                .name("ecr-access-role")
                .build(),
        )
        .send()
        .await?;

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

    launch_instances(&client).await?;

    Ok(())
}
