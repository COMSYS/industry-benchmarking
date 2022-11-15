use serde::Deserialize;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use client::{
    config::EvalMode, execute_client_analyst, execute_client_company, execute_client_spectator,
};
use server::execute_server;
use types::message::io::CompanyUUIDs;

use std::time::Duration;

#[derive(Deserialize)]
struct OrchestraConfig {
    server_host: String,
    server_http: u16,
    server_https: u16,
    exec_server: bool,
    rounds: usize,
}

/// Zip extraction subdir
static ZIP_EXTRATCION_PATH: &str = "../data/orchestra_data";
static ORCHESTRA_CONFIG_PATH: &str = "/orchestra.yaml";

/// Analyst key file subpaths
static ANALYST_PKCS_12_PATH: &str = "/crypto/analyst/analyst.pfx";
static ANALYST_CA_CERTIFICATE_PATH: &str = "/crypto/analyst_ca/analyst_ca.pem";
static ANALYST_CERTIFICATE_PATH: &str = "/crypto/analyst/analyst.pem";

/// Analyst config files subpath
static ANALYST_ALGORITHMS_PATH: &str = "/analyst/algorithms.yaml";
static ANALYST_BENCHMARKING_CONFIG_PATH: &str = "/analyst/benchmarking_config.yaml";

/// Server Certificate paths
static SERVER_CA_CERTIFICATE_PATH: &str = "/crypto/server_ca/server_ca.pem";

/// Path where the analyst puts the uuids of the companies
static COMPANY_UUID_PATH: &str = "../data/client_data/uuids.yaml";
static ANALYST_FINISHED_PATH: &str = "../data/client_data/finished";

/// WAITING TIMES
static SERVER_STARTUP_WAIT_MS: u64 =  20000;
static SERVER_SHUTDOWN_WAIT_MS: u64 = 10000;
static FILE_LOOKUP_WAIT_MS: u64 = 200;
static CLIENT_STARTUP_WAIT_MS: u64 = 2500;
static SPECTATOR_WAIT_MS: u64 = 1100;

/// EVAL IO PATH
static EVAL_IO_BASE_PATH: &str = "../data/evaluation/";

/// == ORCHESTRA ==
///
/// Start servers and clients at once and let them interact
fn main() {
    // Start logger for application
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Get input
    let (orchestra_prefix, zip_file, eval_name, eval_mode) = parse_input();

    // Extract file
    let archive_extract_path = extract_archive(zip_file, ZIP_EXTRATCION_PATH.into());

    // Get orchestra configuration
    let oc = orchestra_config(&orchestra_prefix);

    // Run for the specified amount of rounds
    for i in 0..oc.rounds {
        log::warn!("[ORCHESTRA] Round {} of {}!", i + 1, oc.rounds);

        // Start server if required
        start_server(&oc);

        // Wait for the server to start up before the analyst can send messages
        log::info!(" ## Waiting 1 Second for server startup! ## ");
        std::thread::sleep(Duration::from_millis(SERVER_STARTUP_WAIT_MS));

        // Start up the analyst
        start_analyst(&oc, &orchestra_prefix);

        // Wait for spectator
        std::thread::sleep(Duration::from_millis(SPECTATOR_WAIT_MS));

        start_spectator(&oc, &orchestra_prefix, &eval_name, eval_mode.clone());

        // Check for written UUID file of client
        log::info!(" ## Waiting for analysts written UUID file! ## ");
        let uuid_path: PathBuf = COMPANY_UUID_PATH.into();
        while !uuid_path.exists() {
            std::thread::sleep(Duration::from_millis(FILE_LOOKUP_WAIT_MS));
        }

        // Finally ready
        log::info!(" ## Analyst has provided our input written UUID file! ## ");
        let companies: CompanyUUIDs =
            serde_yaml::from_reader(File::open(uuid_path.clone()).unwrap())
                .expect("Invalid format");

        // Start companies
        start_companies(&oc, &orchestra_prefix, companies.comps);

        // Clean up uuid file
        std::fs::remove_file(uuid_path).unwrap();

        // Check for  file of analyst to finish the run
        log::info!(" ## Waiting for analyst to terminate! ## ");
        let finished_path: PathBuf = ANALYST_FINISHED_PATH.into();
        while !finished_path.exists() {
            std::thread::sleep(Duration::from_millis(FILE_LOOKUP_WAIT_MS));
        }

        // Clean up finished file
        log::info!(" ## Analyst terminated! ## ");
        std::fs::remove_file(finished_path).unwrap();

        // Wait for the OS to free the HTTPs port
        std::thread::sleep(Duration::from_millis(SERVER_SHUTDOWN_WAIT_MS));
    }

    // Clean up extracted files
    log::warn!(
        "Finished orchestra - cleaning up {:?}!",
        archive_extract_path
    );
    std::fs::remove_dir_all(archive_extract_path).expect("Could not remove all files");
}

/// Parse the input argument which is considered to
/// be a zip file which holds all information for
/// one successful run.
fn parse_input() -> (String, File, PathBuf, EvalMode) {
    // Read zip file for orchestration
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        log::error!("No archive or mode provided! Usage: {} <filename> <mode>", args[0]);
        log::error!("Current directory: {:?}", std::env::current_dir());
        std::process::exit(-1);
    }
    let fname = std::path::Path::new(&*args[1]);
    if !fname.exists() {
        log::error!("Could not find orchestra archive! Exiting...");
        std::process::exit(-1);
    }
    let file = fs::File::open(&fname).unwrap();

    let mode: EvalMode = args[2]
        .parse()
        .expect("Could not parse EvaluationMode: Unencrypted | Enclave | Homomorphic");

    // Prefix for all following files
    (
        format!(
            "{}/{}",
            ZIP_EXTRATCION_PATH,
            fname.file_stem().unwrap().to_str().unwrap()
        ),
        file,
        fname.file_stem().unwrap().into(),
        mode,
    )
}

/// Extract archive with format from README.md
fn extract_archive(file: File, target_dir: PathBuf) -> PathBuf {
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut zip_outpath: PathBuf = PathBuf::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = target_dir.join(outpath);

        if i == 0 {
            zip_outpath = outpath.clone();
        }

        // either a dir or a file: handle accordingly
        if (*file.name()).ends_with('/') {
            log::debug!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            log::debug!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    zip_outpath
}

/// Parse orchestra config and return it
fn orchestra_config(orchestra_prefix: &String) -> OrchestraConfig {
    // Read orchestra config of archive
    let oc_path: PathBuf = format!("{}{}", orchestra_prefix, ORCHESTRA_CONFIG_PATH).into();
    serde_yaml::from_reader(File::open(oc_path).expect("Orchestra file does not exist!"))
        .expect("Could not parse orchestra config!")
}

/// Start server if he is required
fn start_server(oc: &OrchestraConfig) {
    if oc.exec_server {
        // Start server in subthread
        log::info!(" ## Starting Server! ## ");
        std::thread::spawn(|| {
            execute_server().expect("Server execution unsuccessful!");
        });
    }
}

/// Start analyst client instance
fn start_analyst(oc: &OrchestraConfig, orchestra_prefix: &String) {
    // Start analyst
    log::info!(" ## Starting Analyst! ## ");
    let server_cert_path: PathBuf =
        format!("{}{}", orchestra_prefix, SERVER_CA_CERTIFICATE_PATH).into();
    let algo_path: PathBuf = format!("{}{}", orchestra_prefix, ANALYST_ALGORITHMS_PATH).into();
    let ca_cert_path: PathBuf =
        format!("{}{}", orchestra_prefix, ANALYST_CA_CERTIFICATE_PATH).into();
    let cert_path: PathBuf = format!("{}{}", orchestra_prefix, ANALYST_CERTIFICATE_PATH).into();
    let pkcs12_path: PathBuf = format!("{}{}", orchestra_prefix, ANALYST_PKCS_12_PATH).into();
    let bc_path: PathBuf =
        format!("{}{}", orchestra_prefix, ANALYST_BENCHMARKING_CONFIG_PATH).into();

    let oc_host = oc.server_host.clone();
    let oc_http = oc.server_http.clone();
    let oc_https = oc.server_https.clone();
    let oc_server_cert_path = server_cert_path.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            execute_client_analyst(
                oc_host,
                oc_http,
                oc_https,
                oc_server_cert_path,
                pkcs12_path,
                ca_cert_path,
                cert_path,
                algo_path,
                bc_path,
                None,
            )
            .await;
        });
    });
}

/// Start analyst client instance
fn start_spectator(
    oc: &OrchestraConfig,
    orchestra_prefix: &String,
    eval_name: &PathBuf,
    eval_mode: EvalMode,
) {
    // Start analyst
    log::info!(" ## Starting Spectator! ## ");
    let server_cert_path: PathBuf =
        format!("{}{}", orchestra_prefix, SERVER_CA_CERTIFICATE_PATH).into();
    let pkcs12_path: PathBuf = format!("{}{}", orchestra_prefix, ANALYST_PKCS_12_PATH).into();

    let eval: PathBuf = format!("{}/{}.csv", EVAL_IO_BASE_PATH, eval_name.to_string_lossy()).into();

    let oc_host = oc.server_host.clone();
    let oc_http = oc.server_http.clone();
    let oc_https = oc.server_https.clone();
    let oc_server_cert_path = server_cert_path.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            execute_client_spectator(
                oc_host,
                oc_http,
                oc_https,
                eval_mode,
                oc_server_cert_path,
                pkcs12_path,
                eval,
                None,
            )
            .await;
        });
    });
}

/// Start up as many clients as the analyst requires
fn start_companies(oc: &OrchestraConfig, orchestra_prefix: &String, companies: Vec<u128>) {
    for (uuid, i) in companies.iter().zip(0..) {
        let company_crypto_path: PathBuf = format!(
            "{}/crypto/comp{}/comp{}.pfx",
            orchestra_prefix,
            i.clone(),
            i.clone()
        )
        .into();
        let company_input_path: PathBuf =
            format!("{}/inputs/comp{}.yaml", orchestra_prefix, i.clone()).into();
        log::debug!(
            "PKCS12 path: {:?} || Input Data Path: {:?}",
            company_crypto_path,
            company_input_path
        );

        let oc_server_cert_path =
            format!("{}{}", orchestra_prefix, SERVER_CA_CERTIFICATE_PATH).into();
        let oc_host = oc.server_host.clone();
        let oc_http = oc.server_http.clone();
        let oc_https = oc.server_https.clone();
        let oc_uuid = uuid.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                log::info!(" ## Starting company {}! ## ", i);
                execute_client_company(
                    oc_host,
                    oc_http,
                    oc_https,
                    oc_server_cert_path,
                    company_crypto_path,
                    company_input_path,
                    oc_uuid,
                    None,
                )
                .await
            });
        });

        std::thread::sleep(Duration::from_millis(CLIENT_STARTUP_WAIT_MS));
    }
}
