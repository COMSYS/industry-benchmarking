use std::{path::PathBuf, collections::HashMap, ops::Div, io::BufWriter};

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::{StreamExt};
use reqwest::Client;
use types::consts::CC_SPECTATOR_EVAL_OUTPUT_KEY;

use crate::{api::TeebenchHttpsAPI, config::EvalMode};

use super::state::ClientConnection;

// [AlgoUpload, AlgoParse, AlgoTopo, CompUpload, CompParse, BenchSingleAVG, BenchTotal, BenchClust, BenchAgg, BenchAssemb]
#[derive(Debug, serde::Serialize)]
struct EvalRecord {
    algo_upload: u128,
    algo_parse: u128,
    algo_topo: u128,
    comp_upload_avg: u128,
    comp_parse_avg: u128,
    comp_bench_avg: u128,
    bench_total: u128,
    bench_clust: u128,
    bench_agg: u128,
    bench_assemb: u128,
    eval_mode: String,
}


/// Analyst Connection has connection Strings
/// and a state which has to be initialized.
pub struct SpectatorConnection {
    /// Conncetion information
    conn_info: SpectatorConnectionInfo,
}

#[derive(Clone)]
/// Internal data structure to perform connections with the analyst
struct SpectatorConnectionInfo {
    /// Connection Specific
    client: Client,
    /// API Routes
    routes_https: TeebenchHttpsAPI,
    /// Eval output path
    eval_output: PathBuf,
    /// Eval mode to write to logs
    eval_mode: EvalMode,
}


impl SpectatorConnectionInfo {
    /// Create new analyst connection info 
    fn new(client: Client, host: String, https_port: String, eval_output: PathBuf, eval_mode: EvalMode) -> Self {
        
        let routes_https = TeebenchHttpsAPI::new(host, https_port);

        SpectatorConnectionInfo {
            client,
            routes_https,
            eval_output,
            eval_mode
        }
    }
}

/// The signature for Analyst an Company are very similar and allow creation and
/// running of the connection.
#[async_trait]
impl ClientConnection for SpectatorConnection {
    
    /// Create an analyst connection from analyst configuration information 
    /// that is passed internally to create the client connection. 
    fn new(client: Client, host: String, _http_port: String, https_port: String, paths: &HashMap<String, PathBuf>, _uuid: Option<u128>, eval_mode: EvalMode ) -> Self {

        let conn_info = SpectatorConnectionInfo::new(client, host, https_port, paths.get(CC_SPECTATOR_EVAL_OUTPUT_KEY).unwrap().into(), eval_mode);
        SpectatorConnection { conn_info }
    }

    /// Start logging and log to fs
    async fn run(&mut self) {
        
        log::debug!("[Spectator] Listening for status reports from server!");

        let mut algo_upload: u128 = 0;
        let mut algo_parse: u128 = 0;
        let mut algo_topo: u128 = 0;

        let mut comp_upload: Vec<u128> = vec![];
        let mut comp_parse: Vec<u128> = vec![];
        let mut comp_bench_single: Vec<u128> = vec![];

        let mut bench_total: u128 = 0;
        let mut bench_clust: u128 = 0;
        let mut bench_agg: u128 = 0;
        let mut bench_assemb: u128 = 0;

        // Get event stream
        match self.conn_info.client.get(self.conn_info.routes_https.get_events()).send().await {
            // Successful response
            Ok(rsp) => {
                let mut stream = rsp.bytes_stream().eventsource();

                // Listen for events and exit whenever the event is `Ready`
                while let Some(server_event) =  stream.next().await{
                    match server_event {
                        Ok(event) => {
                            
                            // LOG TO CSV
                            match event.data.as_str() {
                                "connected" |
                                "keep-alive" | 
                                "Company benchmark started!" |
                                "[1/4] Processing Aggregation: Complete! Clustering data..." |
                                "[2/4] Clustering KPI results: Complete! Aggregating..." |
                                "[3/4] Processing Aggregation: Complete! Assembling information.." |
                                "[4/4] Assembling Information: Complete! You can return your results now!" |
                                "Benchmark sucessfully completed! Analyst is happy!" |
                                "all-participants-enrolled" => {},
                                
                                msg => {
                                    
                                    
                                    if msg.contains("EVAL") {

                                        // We know that we have the eval format
                                        let num: Vec<&str> = msg.clone().split_whitespace().collect();
                                        let var = num[1].parse::<u128>().unwrap();

                                        if msg.contains("ALGO") && msg.contains("UPLOAD") { algo_upload = var; }
                                        if msg.contains("ALGO") && msg.contains("PARSE") { algo_parse = var; }
                                        if msg.contains("ALGO") && msg.contains("TOPO") { algo_topo = var; }

                                        if msg.contains("COMP") && msg.contains("UPLOAD") { comp_upload.push(var); }
                                        if msg.contains("COMP") && msg.contains("PARSE") { comp_parse.push(var); }
                                        
                                        if msg.contains("BENCH") && msg.contains("COMP") && msg.contains("SINGLE") { comp_bench_single.push(var); }
                                        if msg.contains("BENCH") && msg.contains("COMP") && msg.contains("TOTAL") { bench_total = var; }
                                        if msg.contains("BENCH") && msg.contains("CLUST") { bench_clust = var; }
                                        if msg.contains("BENCH") && msg.contains("AGG") { bench_agg = var; }
                                        if msg.contains("BENCH") && msg.contains("ASSEMB") { bench_assemb = var; }

                                    } else {
                                        log::info!("{}", msg);
                                    }
                                },
                            }
                        },
                        Err(_) => { 
                            if comp_upload.is_empty() || comp_parse.is_empty() || comp_bench_single.is_empty(){ log::error!("FATAL: No companies were registered!"); }
                            else {
                                // Do CSV serialization

                                let comp_bench_sum: u128 = comp_bench_single.iter().sum();
                                let comp_upload_sum: u128 = comp_upload.iter().sum();
                                let comp_parse_sum: u128 = comp_parse.iter().sum();

                                let comp_bench_avg: u128 = comp_bench_sum.div(comp_bench_single.len() as u128);
                                let comp_upload_avg: u128 = comp_upload_sum.div(comp_upload.len() as u128);
                                let comp_parse_avg: u128 = comp_parse_sum.div(comp_parse.len() as u128);


                                let record = EvalRecord {
                                    algo_parse, 
                                    algo_topo, 
                                    algo_upload, 
                                    bench_agg, 
                                    bench_assemb, 
                                    bench_clust, 
                                    comp_bench_avg, 
                                    bench_total, 
                                    comp_parse_avg, 
                                    comp_upload_avg,
                                    eval_mode: self.conn_info.eval_mode.to_string()
                                };

                                // Append new headers only when file does not exist
                                let exists = PathBuf::from(self.conn_info.eval_output.clone()).exists();
                                let outfile = std::fs::OpenOptions::new()
                                    .write(true)
                                    .append(true)
                                    .create(true)
                                    .open(self.conn_info.eval_output.clone())
                                    .unwrap();
                                let outbuf = BufWriter::new(outfile);
                                let mut wtr = csv::WriterBuilder::new()    
                                    .has_headers(!exists)
                                    .from_writer(outbuf);

                                wtr.serialize(record).expect("Could not write benchmarking result to CSV file");
                                wtr.flush().expect("Could not flush buffer to file");

                            }
                            log::warn!("Connection to server lost!"); break;}
                    }
                };                
            },
            Err(e) => {log::error!("FATAL {}", e); std::process::exit(-1);}
        }
        
    }
}