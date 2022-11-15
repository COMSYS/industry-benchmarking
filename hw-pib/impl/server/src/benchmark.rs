//! **Company Benchmarking**
//! 
//! Here the computation of the company benchmarks really happens.
//! The process is designed to run in parallel s.t. it can use
//! the full processor's capacity without running into bottlenecks.

use std::{sync::Arc, collections::HashMap};
use async_lock::{RwLock, RwLockUpgradableReadGuard};
use actix_web::web::Data;
use benchmark::{error::BenchmarkingError};
use server_util::broadcast_event::Broadcaster;
use rayon::prelude::*;
use types::output::{Output, OutputVariable};
use std::sync::mpsc::channel;

use crate::server::BenchmarkingServer;

/// Benchmarking start
///
/// We start the threads for all companies & KPIs to perform
/// computation of results. The results get aggregated in the
/// end s.t. the results get stored at each company individually.
/// 
/// The benchmarking uses the broadcaster where it posts events
/// depending on what part of the benchmarking is finished (i.e
/// shows the percentage of tasks that have performed through).
pub fn run_benchmark(srv: Data<Arc<RwLock<BenchmarkingServer>>>, broadcaster: Arc<Broadcaster>) -> Result<(), BenchmarkingError> {

    ////////////////////////////////////////////////////////
    //  STAGE 0 -- Prepare company input data             //  
    ////////////////////////////////////////////////////////

    // Get lock on data and partition it on multiple threads (IN PLACE!)
    let server = srv.try_upgradable_read().ok_or(BenchmarkingError::from("Could not read data now. Try again later!".to_string())).map_err(|e| e)?;

    ////////////////////////////////////////////////////////
    //  STAGE 1 -- Perform KPI computation for companies  //  
    ////////////////////////////////////////////////////////

    // Broadcast message to all registered clients that computation starts
    broadcaster.send("Company benchmark started!");
    // MSPC (multiple producer single consumer) channel
    let (sender, receiver) = channel();

    ////////////////////////////////////////////////////////
    
    #[cfg(feature="evaluation")]
    let now = std::time::SystemTime::now();

    server.companies().par_iter()
        .try_for_each_with(sender, |s, (company_id, company)| {
            
            #[cfg(feature="evaluation")]
            let company_now = std::time::SystemTime::now();
            
            // Compute algorithms for each company and return their results
            let benchmarking_results = server.algorithms().unwrap().run(company);

            #[cfg(feature="evaluation")]
            {
                broadcaster.send(format!("EVAL-BENCH-COMP-SINGLE: {:?}", company_now.elapsed().unwrap().as_nanos()).as_str());
            }
            

            match benchmarking_results {
                Ok(output) => { 
                    #[cfg(feature="evaluation")]
                    {
                        broadcaster.send(format!("EVAL-COMP-PARSE {:?}", output.1).as_str());
                    }
                    s.send((*company_id, output.0)).ok(); 
                    Ok(()) 
                },
                Err(e) => {
                    log::error!("Encountered benchmarking error in thread for company {}: {}", company_id, e);
                    broadcaster.send(format!("Encountered Error in computation: {}", e).as_str());
                    Err(e)
                }
            }

        })?;

    #[cfg(feature="evaluation")]
    broadcaster.send(format!("EVAL-BENCH-COMP-TOTAL: {:?}", now.elapsed().unwrap().as_nanos()).as_str());

    ////////////////////////////////////////////////////////

    let company_kpis: HashMap<u128, Output> = receiver.iter().collect();
    broadcaster.send(&format!("[1/4] Processing Aggregation: Complete! Clustering data..."));
    log::info!("Collected KPI results! -- Starting KPI Clustering!");

    ////////////////////////////////////////////////////////
    // STAGE 2 -- Cluster vars for aggregation            //
    ////////////////////////////////////////////////////////

    let kpis = server.algorithms().unwrap().get_kpis().clone();

    ////////////////////////////////////////////////////////

    let (sender, receiver) = channel();

    #[cfg(feature="evaluation")]
    let now = std::time::SystemTime::now();

    kpis.par_iter().for_each_with(sender, |s: &mut std::sync::mpsc::Sender<(&str, Vec<&Vec<f64>>)>, atomic| {
        // This vector holds all companies results for one specific KPI
        let mut companies_kpi_result: Vec<&Vec<f64>> = Vec::new();

        // Push results to vector of companies KPI results
        for company_output in company_kpis.iter() {
            let company_result = company_output.1.get_result_from_var(atomic.name()).unwrap();
            companies_kpi_result.push(company_result);
        }

        // Add the results to the hashmap
        s.send((atomic.name(), companies_kpi_result)).ok();
    });

    let mut clustered_kpis: HashMap<&str, Vec<&Vec<f64>>>  = receiver.iter().collect();

    #[cfg(feature="evaluation")]
    broadcaster.send(format!("EVAL-BENCH-CLUST: {:?}", now.elapsed().unwrap().as_nanos()).as_str());
    
    ////////////////////////////////////////////////////////

    broadcaster.send(&format!("[2/4] Clustering KPI results: Complete! Aggregating..."));
    log::info!("Clustering Complete! -- Aggregation starting!");

    ////////////////////////////////////////////////////////
    // STAGE 3 -- Aggregate the clustered data            //
    ////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////

    #[cfg(feature="evaluation")]
    let now = std::time::SystemTime::now();

    let statistical_data: HashMap<&str, OutputVariable> = clustered_kpis.par_iter_mut().map(|(&kpi_name, results)| {
        // Compute overall metrics
        let aggregate = server.algorithms().unwrap().aggregate_atomic_var(results).unwrap();
        (kpi_name, aggregate)
    }).collect();

    #[cfg(feature="evaluation")]
    broadcaster.send(format!("EVAL-BENCH-AGG: {:?}", now.elapsed().unwrap().as_nanos()).as_str());
    
    ////////////////////////////////////////////////////////

    log::info!("Aggregation Complete -- Assembling information!");

    #[cfg(not(feature="evaluation"))]
    broadcaster.send(&format!("[3/4] Processing Aggregation: Complete! Assembling information.."));

    ////////////////////////////////////////////////////////
    // STAGE 4 -- Assemble informations for all clients   //
    ////////////////////////////////////////////////////////

    // Perform upgrade on rwlock and update data in place
    let mut server_write = RwLockUpgradableReadGuard::try_upgrade(server).expect("Could not get writer upgrade");

    ////////////////////////////////////////////////////////

    #[cfg(feature="evaluation")]
    let now = std::time::SystemTime::now();

    for (company_id, company) in server_write.set_companies().iter_mut() {
        
        // Copy statistical results into company results
        let vars = company_kpis
            .get(company_id)
            .unwrap()
            .vars()
            .into_par_iter()
            .map(|(key, var)| {
                let res = OutputVariable::new_result_with_statistics(var, statistical_data.get(var.name().as_str()).unwrap());
                (key.to_string(), res)
            })
            .collect();
        
        company.set_results_data(Output::from_output_vars(vars));
    }

    #[cfg(feature="evaluation")]
    broadcaster.send(format!("EVAL-BENCH-ASSEMBLE: {:?}", now.elapsed().unwrap().as_nanos()).as_str());

    log::info!("Assembling information Complete!");
    broadcaster.send(&format!("[4/4] Assembling Information: Complete! You can return your results now!"));
    
    // Let the server sleep s.t. all clients can setup their sockets
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    broadcaster.send("benchmarking-success");

    log::info!("Benchmarking finished!");

    Ok(())
}