use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::args;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_liquidity_pool_dao() {

    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_account();

    let mut token_matadata: HashMap<String, String> = HashMap::new();
    token_matadata.insert("name".to_string(), "USD".to_string());

    // Mint USD
    let mint_usd = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .new_token_fixed( token_matadata, dec!("100000"))
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let mint_usd_receipt = test_runner.execute_manifest_ignoring_fee(mint_usd, vec![public_key]);
    println!("{:?}\n", mint_usd_receipt);
    mint_usd_receipt.expect_commit_success();

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    let usd = mint_usd_receipt
        .expect_commit()
        .entity_changes
        .new_resource_addresses[0];

    // Test the `instantiate_component` function.
    let transaction1 = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .withdraw_from_account_by_amount(dec!("50"), RADIX_TOKEN, account_component)
        .withdraw_from_account_by_amount(dec!("50"), usd, account_component)
        .take_from_worktop(RADIX_TOKEN, |builder, xrd_bucket| {
            builder.take_from_worktop(usd, |builder1, usd_bucket| {
                builder1.call_function(
                    package_address,
                    "LiquidityPool",
                    "new",
                    args!(
                        dec!("0.5"), 
                        dec!("0.5"),
                        scrypto::resource::Bucket(xrd_bucket),
                        scrypto::resource::Bucket(usd_bucket),
                        dec!("0.02")
                    ),
                )
            })
            })
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(transaction1, vec![public_key]);
    println!("{:?}\n", receipt1);
    receipt1.expect_commit_success();
    
}