use near_indexer_test_framework::*;
use mintbase_deps::*;

#[test]
fn test_deploy_contracts() {
    while_indexer(|ns:NearState| async move {
        
        background_contracts_deployed(&ns).await;
        as_a_store_owner(&ns,None).await;
       

            // println!("hello {:?}",p);
            // c.await;
            // let resp = login_create_new_account(
            //     "http://localhost", 
            //     p.to_string().as_str(),
            //         "random"
            //     )
            //     .await;
            // assert_eq!(hyper::StatusCode::OK, resp.status());
            // let b = hyper::body::to_bytes(resp).await.unwrap();
            // let v = b.to_vec();
            // let s = String::from_utf8(v).unwrap();
            // println!("{}", s);
    });
}