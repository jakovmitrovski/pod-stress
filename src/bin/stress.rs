use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, TxKind, U256};
use alloy::providers::{fillers::GasFiller, Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use eyre::Result;
use hex;
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::sync::Arc;

sol! {
    #[sol(rpc)]
    contract RankedFeed {
        mapping(bytes32 => uint256) public votes;
        mapping(address => mapping(bytes32 => bool)) public voted;

        event PostCreated(bytes32 indexed post_id, address indexed poster, bytes post_data);
        event PostVoted(bytes32 indexed post_id, address indexed voter);

        error AlreadyVoted();

        function createPost(bytes calldata post_data) public;
        function votePost(bytes32 post_id) public;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // dotenv()?;
    let _ = CryptoProvider::install_default(default_provider());

    let rpc_url_ws = std::env::var("RPC_URL_WS")?;
    let contract_address = std::env::var("CONTRACT_ADDRESS")?.parse::<Address>()?;
    let private_key = std::env::var("PRIVATE_KEY")?;

    let signing_key =
        SigningKey::from_bytes(&hex::decode(&private_key).unwrap().into_iter().collect()).unwrap();
    let signer = PrivateKeySigner::from_signing_key(signing_key);
    let wallet = EthereumWallet::new(signer.clone());
    let address = signer.address();

    let provider = Arc::new(
        ProviderBuilder::new()
            .filler(GasFiller)
            .wallet(wallet.clone())
            .on_ws(WsConnect::new(Url::parse(&rpc_url_ws)?))
            .await?,
    );

    let mut random_wallets = vec![];
    for i in 0..500 {
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let signer = PrivateKeySigner::from_signing_key(signing_key.clone());
        let wallet = EthereumWallet::new(signer.clone());
        random_wallets.push(wallet);
        let tx_req = TransactionRequest {
            from: Some(address),
            to: Some(TxKind::Call(signer.address())),
            value: Some(U256::from(85_000_000_000_000u64)),
            gas: Some(1000000),
            gas_price: Some(1_000_000_000),
            ..Default::default()
        };

        let res = provider.send_transaction(tx_req).await?;
        let receipt = provider
            .get_transaction_receipt(*res.tx_hash())
            .await?
            .unwrap();
        assert!(
            receipt.status(),
            "Transaction funding wallet should have succeeded"
        );
        println!("Funded wallet: {:?} - {}", signer.address(), i);
    }

    let post_data = "{\"title\":\"Jakov Test Post\",\"imageHash\":\"77ad0f4abcd7e8822c96920e21f5dfad667e1a014a86759b95f917d67d467e3b\",\"createdAt\":1743496299600}";
    let post_data_bytes = Bytes::from(post_data.as_bytes().to_vec());

    let mut task_inputs = Vec::new();
    for (i, wallet) in random_wallets.iter().enumerate() {
        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .filler(GasFiller)
            .on_ws(WsConnect::new(Url::parse(&rpc_url_ws)?))
            .await;

        if let Err(_) = provider {
            continue;
        }

        println!("Provider: {}", i);

        task_inputs.push(provider.unwrap());
    }

    let _shared_provider = ProviderBuilder::new()
        .filler(GasFiller)
        .on_ws(WsConnect::new(Url::parse(&rpc_url_ws)?))
        .await?;

    let tasks = task_inputs.into_iter().map(|provider| {
        let provider = provider.clone();
        let contract_address = contract_address;
        let post_data_bytes = post_data_bytes.clone();

        tokio::spawn(async move {
            let contract = RankedFeed::new(contract_address, provider);
            let tx = contract
                .createPost(post_data_bytes)
                .gas(80_000)
                .gas_price(1000000000)
                .send()
                .await
                .unwrap();

            Ok::<_, eyre::Error>(*tx.tx_hash())
        })
    });

    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(Ok(tx_hash)) => {
                let receipt = provider.get_transaction_receipt(tx_hash).await?.unwrap();
                if receipt.status() == false {
                    println!("❌ Transaction failed: {i} - {:?}", tx_hash);
                } else {
                    println!("✅ Transaction succeeded: {i} - {:?}", tx_hash);
                }
                assert!(receipt.status(), "Transaction should have succeeded");
            }
            Ok(Err(err)) => eprintln!("❌ Send error: {:?}", err),
            Err(err) => eprintln!("❌ Join error: {:?}", err),
        }
    }

    Ok(())
}
