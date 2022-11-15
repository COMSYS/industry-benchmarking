//! Benchmarking administration
//! 
//! These routes are managed by the analyst and are for managing the
//! benchmarking process, i.e. by adding configuration or asking for
//! it. Especially companies should be able to have insights on the 
//! name, description and k-anonymity of the server that were 
//! configured by the analyst.
//! 
//! Here the companies are also enrolled and the analyst can even check
//! which company did not upload their data, i.e. administrational info.
//! 
//! Finally the analyst can broadcast events to all clients that are 
//! enrolled in the broadcasting channel and start the benchmark, when
//! enough participants uploaded their data (k-anonymity).

use std::{sync::Arc, fs, str::FromStr};
use async_lock::RwLock;

use actix_multipart::Multipart;
use actix_web::{Responder, web::{Json, self, Path, Data}};
use uuid::Uuid;

use types::{message::{
    request::{AnalystBenchmarkingMsg, AnalystEventMsg},
    response::RspMsg,
}, entity::{BenchmarkingConfig, Company}, consts::{FORM_DATA_FIELD_03_CONFIGURATION_MIME, FORM_DATA_FIELD_03_CONFIGURATION_NAME}};
use server_util::{error::ApiError, files::save_multipart_files, broadcast_event::Broadcaster};

use crate::{server::BenchmarkingServer, benchmark::run_benchmark};

///
/// SERVER CONFIGURATION_ROUTES
/// 

/// Get benchmarking config or throw error if there is none
pub async fn get_benchmark_config(srv: Data<Arc<RwLock<BenchmarkingServer>>>,) -> Result<impl Responder, ApiError>{
    
    let srv_rdr = srv.read().await;
    Ok(Json(RspMsg::new(true,"Benchmark Config".to_string() , srv_rdr.benchmarking_config().clone())))
}

/// Change initially set options afterwards
pub async fn modify_benchmark_config(srv: Data<Arc<RwLock<BenchmarkingServer>>>, payload: Multipart) -> Result<impl Responder, ApiError>{

    let mut mut_srv = srv.write().await;

    // Force initial configuration
    if mut_srv.crypto_config().root_ca_certificate().is_none() {
        return Err(ApiError::from(&("Settings have to be initialized first".to_string())));
    }

    // Extract config
    let required_multiparts = vec![(FORM_DATA_FIELD_03_CONFIGURATION_NAME, FORM_DATA_FIELD_03_CONFIGURATION_MIME)];
    let req_multipart_names: Vec<&str> = required_multiparts.iter().map(|x| x.0).collect();
    let required_files = 1;
    let files = save_multipart_files(payload,required_multiparts, required_files).await?;

    // Parse benchmarking server configuration information
    let data = fs::read_to_string(files.get(req_multipart_names[0]).ok_or(ApiError::from("Did not find yaml to be correctly uploaded (incorrect name)!"))?);
    let server_config: BenchmarkingConfig = serde_yaml::from_str::<BenchmarkingConfig>(&data.unwrap()).map_err(|e| ApiError::from(&e.to_string()))?;

    // Skip check for valid configuration
    mut_srv.set_benchmarking_config_all(server_config.clone());

    // Respond with success
    Ok(
        Json(RspMsg::new(
            true,
            "Benchmarking configuration successfully modified!".into(),
            server_config
        ))
    )

}


///
/// COMPANY MANAGEMENT ROUTES
/// 

/// Company registration by an analyst
/// 
/// This lets the server generate a UUID for a company.
/// The UUID is shared by the analyst to the respective
/// company which performs further registration on its own.
pub async fn company_enroll(srv: Data<Arc<RwLock<BenchmarkingServer>>>) -> Result<impl Responder, ApiError>{

    let mut mut_srv = srv.write().await;

    // Verify that an algorithm has been provided - this assures that enrollment of companies is always possible with their algorithms
    // Otherwise their input data cannot be semantically verified (i.e. missing fields)
    if mut_srv.algorithms().is_none() {
        return Err(ApiError::from("No algorithms have been provided up until now. They are required in advance!"));
    }

    // Generate new UUID for company 
    let mut company_id = Uuid::new_v4().as_u128();

    // In case there are miraculously collisions
    while mut_srv.companies().get(&company_id).is_some() {
        company_id = Uuid::new_v4().as_u128();
    }

    log::debug!("Generating company: {:?}, exists: {:?}", company_id, mut_srv.companies().get(&company_id).is_some());

    // Populate settings info data structure with new company information
    let company = Company::default();
    mut_srv.set_companies().insert(company_id, company);

    log::info!("Currently enrolled companies: {}", mut_srv.companies().len());

    // Send positive response
    Ok(Json(
        RspMsg::new(true, format!("Company with ID {} successfully added!", company_id), company_id)
    ))
}


/// Retrieve a company on request
/// 
/// This request returns an ApiError in case the company was not found.
pub async fn get_company_status(srv: Data<Arc<RwLock<BenchmarkingServer>>>, company_id: Path<String>,) -> Result<impl Responder, ApiError>{
    
    let srv_rdr = srv.read().await;

    // Extract uuid from request
    let company_id_uuid = match u128::from_str(&company_id) {
        Ok(uuid) => uuid,
        Err(_) => { return Err(ApiError::from("Could not parse the given Company UUID!")); }
    };

    // Get information on company and respond with current info
    let company = match srv_rdr.companies().get(&company_id_uuid) {
        None => { return Err(ApiError::from("Could not find requested company!")); },
        Some(result) => result
    };

    // HMAC registration counts as "registered"
    Ok(Json(RspMsg::new(
        true, 
        format!("{} is{}registered and does{}participate by uploading input data!", company_id_uuid, if company.certificate().is_some()  {""}  else {" not "}, if company.does_participate()  {""}  else {" not "}),
        () )))
}

///
/// BENCHMARKING ROUTES
/// 

/// Start Benchmark from analyst's perspective
pub async fn start_benchmark(
    analyst_bm_msg: web::Json<AnalystBenchmarkingMsg>, 
    srv: Data<Arc<RwLock<BenchmarkingServer>>>,
    broadcaster: Data<Broadcaster>) -> Result<impl Responder, ApiError>{

    let srv_handle = srv.clone();
    let srv_rdr = srv_handle.read().await;


    // Todo: Extract information from the request to find out which KPIs should be evaluated (let analyst select)
    // Now: all are selected regardless of analyst's choice
    log::debug!("Selected analyst Benchmarking KPIs: {:?}", analyst_bm_msg);
    
    // Check whether all participants have ready data - error out in case of missing
    if srv_rdr.active_participants() < srv_rdr.benchmarking_config().k_anonymity() {
        return Err(ApiError::from(&format!("K-anonymity threshold not satisfied: {} / {}", srv_rdr.active_participants(), srv_rdr.benchmarking_config().k_anonymity())));
    }

    // WARNING: THIS UNLOCK MIGHT BE TOO LATE!

    // Spawn threads for computation of benchmarks and return imediately
    std::thread::spawn(move || { 
        match run_benchmark(srv, broadcaster.into_inner()) {
            Ok(()) => log::info!("Benchmarking successfully finished!"),
            Err(err) => {
                log::error!("Benchmarking failed due to invalid computation: {}", err);
                std::process::exit(-1);
            },
        } });

    // Put another message in body
    Ok(Json(RspMsg::new(true, "Benchmarking successfully stated. Listen for events on /api/events.".to_string(), ())))
}

///
/// ANALYST SERVER MANAGEMENT
/// 

/// Send custom broadcast message to all registered participants
pub async fn broadcast_event(event_msg: web::Json<AnalystEventMsg>, broadcaster: Data<Broadcaster>) -> Result<impl Responder, ApiError>{
    // Broadcast message to all registered clients
    broadcaster.send(event_msg.event.as_str());
    // Put another message in body
    Ok(Json(RspMsg::new(true, "Event successfully dispatched".to_string(), ())))
}