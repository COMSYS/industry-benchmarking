// All route handlers from rest directory
use crate::{api::{
    benchmark_admin::{
        get_benchmark_config,
        modify_benchmark_config,
        company_enroll,
        get_company_status,
        broadcast_event,
        start_benchmark
    },
    company::{
        register,
        upload_input_data,
        get_input_data,
        modify_input_data,
        get_results,
        enroll_event_stream,
    },
    analyst::{
        upload_algorithms,
        get_algorithms,
        modify_algorithms,
    },
    server_admin::{ setup_config, check_config, shutdown},
    index::{ index, favicon, whoami },
}, middleware::request_verifier::VerifyRequest
};

use actix_web::web;
use types::consts::{ROUTE_FAVICON, ROUTE_SETUP, ROUTE_WHOAMI, ROUTE_API, ROUTE_INDEX, ROUTE_ATTEST, ROUTE_SHUTDOWN, ROUTE_COMPANY, ROUTE_ENROLL_EVENTS, S_ROUTE_COMPANY_EXT_INPUT_DATA_ID, S_ROUTE_COMPANY_EXT_REGISTER_ID, S_ROUTE_COMPANY_EXT_RESULTS_ID, ROUTE_ANALYST, ROUTE_ANALYST_EXT_BENCHMARK_CONFIG, S_ROUTE_ANALYST_EXT_COMPANY_STATUS, ROUTE_ANALYST_EXT_ENROLL_COMPANY, ROUTE_ANALYST_EXT_ALGORITHMS, ROUTE_ANALYST_EXT_BENCHMARK, ROUTE_ANALYST_EXT_EVENT};

///
/// NOTE: GET Routes are used for debugging purposes and will be disabled
///       in later builds. 
/// 

// HTTP routes only allow the setup and asking the server, whether he was configured
pub fn http_routes(app: &mut web::ServiceConfig) {
    app 
        // Icon of website
        .service(web::resource(ROUTE_FAVICON).to(favicon))
        // Index for general Information
        .service(web::resource(ROUTE_INDEX).to(index))
        // For participants to get their certificate which they use and their connection
        .service(web::resource(ROUTE_WHOAMI).to(whoami))
        // API access
        .service(web::scope(ROUTE_API)
            // server setup route
            .service(web::resource(ROUTE_SETUP)
                .route(web::post().to(setup_config))
            )
            // server attestation route
            .service(web::resource(ROUTE_ATTEST)
                .route(web::get().to(check_config))
            )
        );
}

// HTTPs routes allow the full feature set of the application to be used
pub fn https_routes(app: &mut web::ServiceConfig) {
    app 
        // Icon of website
        .service(web::resource(ROUTE_FAVICON).to(favicon))
        // Root of website (index)
        .service(web::resource(ROUTE_INDEX).to(index))
        // For participants to get their certificate which they use and their connection
        .service(web::resource(ROUTE_WHOAMI).to(whoami))
        // API access
        .service(web::scope(ROUTE_API)
            // server setup routes
            .service(web::resource(ROUTE_SETUP)
                //
                // Only the analyst performs the configuration with the setup
                // config. (He is the first to connect to the server!)
                // Thus he is the first to upload everything before the
                // service locks down to restricted access. The setup does
                // also perform the basic benchmark config, thus the analyst
                // **gets only the ability to change but not to set**
                // the config later on.
                // 
                .wrap(VerifyRequest::verify_analyst())
                .route(web::post().to(setup_config))
            )
            // Server attestation route
            .service(web::resource(ROUTE_ATTEST)
                // 
                // This route is especially for companies to attest the
                // genuineness of the server.
                // 
                .route(web::get().to(check_config))
            )
            .service(web::resource(ROUTE_SHUTDOWN)
                //
                // Only the analyst has access to shutdown the server
                // Thus the server verifies requests to be from him.
                //
                .wrap(VerifyRequest::verify_analyst())
                .route(web::post().to(shutdown))
            )
            .service(web::scope(ROUTE_COMPANY)
                //
                // Company paths: These paths are for companies
                // to manage themselves w.r.t. their data, enrollment
                // and results. The access to these routes is company
                // exclusive (by ID).
                // 
                .service(web::resource(S_ROUTE_COMPANY_EXT_REGISTER_ID)
                    //
                    // Company registration with Certificate for further communication
                    // This route is not secured as the setup first extracts the peers
                    // TLS certificate from the request.
                    //
                    .route(web::post().to(register))
                )
                // !TODO! [WARN] PLEASE change to header ID or decide on this
                .service(web::resource(S_ROUTE_COMPANY_EXT_INPUT_DATA_ID)
                    //
                    // Company data upload and change data are inherently
                    // required to be secured by request verification.
                    //
                    .wrap(VerifyRequest::verify_company())
                    .route(web::post().to(upload_input_data))
                    .route(web::put().to(modify_input_data))
                    .route(web::get().to(get_input_data))
                )
                .service(web::resource(S_ROUTE_COMPANY_EXT_RESULTS_ID)
                    //
                    // Similarly: getting company results for given inputs.
                    //
                    .wrap(VerifyRequest::verify_company())
                    .route(web::get().to(get_results))
                )
            )
            .service(web::resource(ROUTE_ENROLL_EVENTS)
                //
                // Registration for company updates on benchmarking
                // Event path: This path is "publicly" available (i.e.
                // to all participants) and dispatches the status of 
                // the server (i.e. the benchmarking progress).
                //
                .route(web::get().to(enroll_event_stream))
            )
            .service(web::scope(ROUTE_ANALYST)
                //
                // analyst configuration, registration and benchmark
                //    
                .wrap(VerifyRequest::verify_analyst())
                .service(web::resource(ROUTE_ANALYST_EXT_BENCHMARK_CONFIG)
                    //
                    // config for k-anonymity or data to provide to companies
                    //
                    .route(web::put().to(modify_benchmark_config))
                    .route(web::get().to(get_benchmark_config))
                )
                .service(web::resource(S_ROUTE_ANALYST_EXT_COMPANY_STATUS)
                    //
                    // Get company status (in case of k < required k)
                    //
                    .route(web::get().to(get_company_status))
                )
                .service(web::resource(ROUTE_ANALYST_EXT_ENROLL_COMPANY)
                    //
                    // registration for companies (not already having an id)
                    //
                    .route(web::post().to(company_enroll))
                )
                .service(web::resource(ROUTE_ANALYST_EXT_ALGORITHMS)
                    // 
                    // Algorithm upload and modification
                    // 
                    .route(web::post().to(upload_algorithms))
                    .route(web::put().to(modify_algorithms))
                    .route(web::get().to(get_algorithms))
                )
                .service(web::resource(ROUTE_ANALYST_EXT_BENCHMARK)
                    //
                    // start benchmarking → server sends SSE as status messages
                    //
                    .route(web::post().to(start_benchmark))
                )
                .service(web::resource(ROUTE_ANALYST_EXT_EVENT)
                    //    
                    // Analyst custom events
                    //
                    .route(web::post().to(broadcast_event))
                )
            )
        );
}