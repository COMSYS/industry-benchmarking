//! Analyst routes [`/api/analyst`] for managing Algorithms
//! 
//! The analyst can upload his algorithms here.  He can upload or even change them.
//! For now also getting uploaded algortihms is possible. When uploaded, algorithms
//! get verified for correctness (i.e. no dependency issues and syntactical errors).
//! 
//! Algorithms are encoded as YAML files that hold all operations (not necessarily)
//! in sequential order. There is only one file which holds many KPIs, as they can
//! depend on each other. One Algorithm example:
//! 
//! ```yaml
//! operations:
//! - name: test_op
//!   op: Addition
//!   is_kpi: true
//!   var:
//!     - three
//!     - two_op
//! 
//! - name: two_op
//!   op: AdditionConst
//!   is_kpi: false
//!   var:
//! - one
//!   constant: 1
//! 
//! - name: mul
//!   compute_for_each: false
//!   op: Multiplication
//!   is_kpi: true
//!     var:
//!     - two
//!     - two_op
//!     - three
//! ```
//! 
//! Here the algorithm is requiring `three`, `one` and `two` as input variables as
//! they are not explicitly mentioned to be computable. 

use std::sync::Arc;
use async_lock::RwLock;

use actix_multipart::Multipart;
use actix_web::{Responder, web::{Json, Data}};

use benchmark::Algorithm;
use server_util::{error::ApiError, files::save_multipart_files, broadcast_event::Broadcaster};
use types::{message::response::RspMsg, consts::{FORM_DATA_FIELD_02_ALGORITHMS_MIME, FORM_DATA_FIELD_02_ALGORIHTMS_NAME}};

use crate::server::BenchmarkingServer;

pub async fn upload_algorithms(payload: Multipart, srv: Data<Arc<RwLock<BenchmarkingServer>>>, _broadcaster: Data<Broadcaster>) -> Result<impl Responder, ApiError> {

    #[cfg(feature="evaluation")]
    let now = std::time::SystemTime::now();

    // Extract algorithm from request
    let required_multiparts = vec![(FORM_DATA_FIELD_02_ALGORIHTMS_NAME, FORM_DATA_FIELD_02_ALGORITHMS_MIME)];
    let req_multipart_names: Vec<&str> = required_multiparts.iter().map(|x| x.0).collect();
    let required_files = 1;
    let files = save_multipart_files(payload,required_multiparts, required_files).await.map_err(|e| ApiError::from(&e.to_string()))?;
    
    #[cfg(feature="evaluation")]
    _broadcaster.send(format!("EVAL-ALGO-UPLOAD: {:?}", now.elapsed().unwrap().as_nanos()).as_str());

    // Parse Algorithm
    #[cfg(not(feature="evaluation"))]
    let (algorithms,_,_) = Algorithm::load(files.get(req_multipart_names[0]).ok_or(ApiError::from("Could not process upload!"))?)?;

    #[cfg(feature="evaluation")]
    let (algorithms,parse, topo) = Algorithm::load(files.get(req_multipart_names[0]).ok_or(ApiError::from("Could not process upload!"))?)?;

    #[cfg(feature="evaluation")]
    {
        _broadcaster.send(format!("EVAL-ALGO-PARSE: {}", parse).as_str());
        _broadcaster.send(format!("EVAL-ALGO-TOPO: {}", topo).as_str());
    }
    

    log::debug!("Uploaded algorithms: {:#?}", algorithms);

    // Write to config
    let mut mut_srv = srv.write().await;
    mut_srv.set_algorithms(Some(algorithms));

    Ok(Json(RspMsg::new(true, "Upload successful!".to_string(), ())))
}

pub async fn modify_algorithms(payload: Multipart, srv: Data<Arc<RwLock<BenchmarkingServer>>>, broadcaster: Data<Broadcaster>) -> Result<impl Responder, ApiError> {
    
    {
        let srv_rdr = srv.read().await;
        // Require an exisiting configuration
        if srv_rdr.algorithms().is_none() {
            return Err(ApiError::from("No algorithms present!"));
        }
    }

    upload_algorithms(payload, srv, broadcaster).await
}

pub async fn get_algorithms(srv: Data<Arc<RwLock<BenchmarkingServer>>>,) -> Result<impl Responder, ApiError> {
    let srv_rdr = srv.read().await;
    // Require an exisiting configuration
    if srv_rdr.algorithms().is_none() {
        Err(ApiError::from("No algorithms present!"))
    } else {
        Ok(Json(RspMsg::new(true, "Algorithms".to_string(), srv_rdr.algorithms().unwrap().clone())))
    }
}