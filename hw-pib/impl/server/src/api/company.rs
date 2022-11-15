//! Company routes
//! 
//! Here companies can upload their input data, modify it and get
//! their results after a benchmark has been performed. Companies
//! can also register in the broadcasting channel for further info
//! from the analyst or on the progress of one benchmark.
//! 
//! Similar to the analyst, the companies also do the HMAC procedure
//! to authenticate their messages. 

use std::{sync::Arc, str::FromStr};
use async_lock::RwLock;

use actix_http::header;
use actix_multipart::Multipart;
use actix_web::{Responder, web::{Json, Data, Path}, HttpResponse, HttpRequest};
use rustls::Certificate;
use server_util::{
    error::ApiError,
    files::{save_multipart_files}, broadcast_event::Broadcaster,
};
use types::{message::response::RspMsg, input::Input, consts::{FORM_DATA_FIELD_04_COMPANY_INPUT_NAME, FORM_DATA_FIELD_04_COMPANY_INPUT_MIME}};

use crate::server::BenchmarkingServer;

///
/// Company specific routes
///

/// Perform company input data upload and store it in data structure
pub async fn upload_input_data(srv: Data<Arc<RwLock<BenchmarkingServer>>>, broadcaster: Data<Broadcaster>, company_id: Path<String>, payload: Multipart) -> Result<impl Responder, ApiError>{
    
    let (input, company_id_uuid) = {
        let mut mut_srv = srv.write().await;

        // Extract uuid from request
        let company_id_uuid = match u128::from_str(&company_id) {
            Ok(uuid) => uuid,
            Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
        };
    
        // Get information on company and respond with current info
        let company = match mut_srv.set_companies().get_mut(&company_id_uuid) {
            None => { return Err(ApiError::from("The given ID is not enrolled - the provided UUID might be incorrect!")); },
            Some(result) => result
        };
    
        // No changes allowed before integriy cannot be assured
        if company.certificate().is_none() {
            return Err(ApiError::from("The requesting company has not setup signing - This is required in advance!"));
        }
    
        if company.input_data().size() != 0 {
            return Err(ApiError::from("Existing data has to be modified!"));
        }
    

        #[cfg(feature="evaluation")]
        let now = std::time::SystemTime::now();

        // Extract config
        let required_multiparts = vec![(FORM_DATA_FIELD_04_COMPANY_INPUT_NAME, FORM_DATA_FIELD_04_COMPANY_INPUT_MIME)];
        let req_multipart_names: Vec<&str> = required_multiparts.iter().map(|x| x.0).collect();
        let required_files = 1;
        let files = save_multipart_files(payload,required_multiparts, required_files).await?;
        
        #[cfg(feature="evaluation")]
        {
            broadcaster.send(format!("EVAL-COMP-UPLOAD {:?}", now.elapsed().unwrap().as_nanos()).as_str());
        }

        let input_data = Input::load(files.get(&req_multipart_names[0]).ok_or(ApiError::from("File upload not successful!"))?).map_err(|e| ApiError::from(&e.to_string()))?;

        (input_data, company_id_uuid)
    };
    
    #[cfg(not(feature="evaluation"))]
    {
        let srv_rdr = srv.read().await;
        srv_rdr.algorithms().clone().unwrap().verify_input(input.clone())?;
    }

    // Since we know that the company exists from above we skip checks
    let mut mut_srv = srv.write().await;
    let company = mut_srv.set_companies().get_mut(&company_id_uuid).unwrap();

    // Finally set input data
    company.set_input_data(input.clone());
    
    // Bump the active participants count
    mut_srv.increment_active_participants();

    // Check if enough participants enrolled and send a broadcast 
    if mut_srv.active_participants() == mut_srv.benchmarking_config().k_anonymity() {
        broadcaster.send("all-participants-enrolled");
    }else {
        broadcaster.send(format!("{} of {} participants enrolled!", mut_srv.active_participants(), mut_srv.benchmarking_config().k_anonymity()).as_str())
    }

    // Insert the data to the input
    Ok(Json(RspMsg::new(true, "Successfully uploaded input data!".to_string(), input)))
}

/// Perform company input data upload and modify existing data in state
pub async fn modify_input_data(srv: Data<Arc<RwLock<BenchmarkingServer>>>, company_id: Path<String>, payload: Multipart) -> Result<impl Responder, ApiError>{
    
    let mut mut_srv = srv.write().await;

    // Extract uuid from request
    let company_id_uuid = match u128::from_str(&company_id) {
        Ok(uuid) => uuid,
        Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
    };

    // Get information on company and respond with current info
    let company = match mut_srv.set_companies().get_mut(&company_id_uuid) {
        None => { return Err(ApiError::from("You are not enrolled or your UUID is incorrect!")); },
        Some(result) => result
    };

    // No changes allowed before integriy cannot be assured
    if company.certificate().is_none() {
        return Err(ApiError::from("The requesting company has not setup signing - This is required in advance!"));
    }

    // Check that no config is present
    if !company.does_participate() {
        return Err(ApiError::from("No input data in memory - nothing to modify - perform an upload first."))
    }

    // Extract config
    let required_multiparts = vec![("input_data", "text/yaml")];
    let req_multipart_names: Vec<&str> = required_multiparts.iter().map(|x| x.0).collect();
    let required_files = 1;
    let files = save_multipart_files(payload,required_multiparts, required_files).await.map_err(|e| ApiError::from(&e.to_string()))?;
    
    let input_data = Input::load(files.get(&req_multipart_names[0]).ok_or(ApiError::from("File upload not successful!"))?).map_err(|e| ApiError::from(&e.to_string()))?;
    company.set_input_data(input_data.clone());

    #[cfg(not(feature="evaluation"))]
    mut_srv.algorithms().clone().unwrap().verify_input(input_data.clone())?;

    // Insert the data to the input
    Ok(Json(RspMsg::new(true, "Successfully uploaded input data!".to_string(), input_data)))
}

/// Return a copy of already uploaded company data to the participant or respond with error
pub async fn get_input_data(srv: Data<Arc<RwLock<BenchmarkingServer>>>, company_id: Path<String>,) -> Result<impl Responder, ApiError>{
    
    let srv_rdr = srv.read().await;

    // Extract uuid from request
    let company_id_uuid = match u128::from_str(&company_id) {
        Ok(uuid) => uuid,
        Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
    };

    // Get information on company and respond with current info
    let company = match srv_rdr.companies().get(&company_id_uuid) {
        None => { return Err(ApiError::from("You are not enrolled or your UUID is incorrect!")); },
        Some(result) => result
    };

    // No changes allowed before integriy cannot be assured
    if company.certificate().is_none() {
        return Err(ApiError::from("The requesting company has not setup signing - This is required in advance!"));
    }

    Ok(Json(RspMsg::new(true, format!("{} input data", company_id_uuid), company.input_data().clone())))
}

/// Retrieve company result - is `None` (i.e., `null`) if no result exists
pub async fn get_results(srv: Data<Arc<RwLock<BenchmarkingServer>>>, company_id: Path<String>,) -> Result<impl Responder, ApiError>{
    
    let srv_rdr = srv.read().await;

    // Extract uuid from request
    let company_id_uuid = match u128::from_str(company_id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
    };

    // Get information on company and respond with current info
    let company = match srv_rdr.companies().get(&company_id_uuid) {
        None => { return Err(ApiError::from("You are not enrolled or your UUID is incorrect!")); },
        Some(result) => result
    };

    // Check if results exist
    if company.results_data().size() == 0 {
        return Err(ApiError::from("No results available!"));
    }
    
    // Return results
    Ok(Json(RspMsg::new(true, format!("Results for company {}", company_id_uuid), company.results_data().clone())))
}

/// Companies register for events that happen on the server.
/// 
/// The server transmits its status (e.g. whether enough participants joined)
/// and informs all clients instead of relying on polling mechanisms.
/// We explicitly need to use `text/event-stream` as mimetype.
pub async fn enroll_event_stream(broadcaster: Data<Broadcaster>) -> Result<impl Responder, ApiError> {
    let rx = broadcaster.new_client();

    Ok(HttpResponse::Ok()
    .append_header((header::CONTENT_TYPE, "text/event-stream"))
    .streaming(rx))
}


/// Submission of new company HMAC for succeeding messages
/// 
/// The procedure is similar to the registration function.
pub async fn register(srv: Data<Arc<RwLock<BenchmarkingServer>>>, company_id: Path<String>, req: HttpRequest) -> Result<impl Responder, ApiError>{
    
    let mut mut_srv = srv.write().await;

    // Extract uuid from request
    let company_id_uuid = match u128::from_str(&company_id) {
        Ok(uuid) => uuid,
        Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
    };

    // Get information on company and respond with current info
    let company = match mut_srv.set_companies().get_mut(&company_id_uuid) {
        None => { return Err(ApiError::from("You are not enrolled or your UUID is incorrect!")); },
        Some(result) => result
    };

    // Check if Certificate was already configured
    if company.certificate().is_some() {
        return Err(ApiError::from("You have already registered your certificate!"));
    }

    // Insert Certificate into structure 
    company.set_certificate(req.conn_data::<Certificate>().unwrap().clone());

    // Respond and show changes
    Ok(Json(RspMsg::new(true, format!("{} successfully registered HMAC!", company_id_uuid), ())))
}