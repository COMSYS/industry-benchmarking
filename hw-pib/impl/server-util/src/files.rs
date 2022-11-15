use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::{fmt::Debug, io::Write};
use std::io::BufReader;
use actix_multipart::{Multipart, Field};
use actix_web::{web, Error};
use futures::TryStreamExt;
use std::fs::remove_file;
use uuid::Uuid;
use crate::error::ApiError;
use types::consts::SERVER_DATA_PATH;

type FieldName<'a> = &'a str;
type ContentType<'a> = &'a str;


/// Save multiparted stream over HTTP
///
/// - You specify the number of files you want to have uploaded
/// - In case the file number does not match - revert all changes to filesystem
/// - The accepted mime_types are used in the vector to specify the file list. The ordering is arbitrary.
/// - To allow wildcard content types or even text (application/octet-stream) use * as mime_type
pub async fn save_multipart_files<'a>(mut payload: Multipart, mut accepted_files: Vec<(FieldName<'a>,ContentType<'a>)>, required_files: u16) -> Result<HashMap<&'a str, PathBuf>, ApiError> {

    // Save file paths
    let mut file_list: HashMap<&str, PathBuf> = HashMap::new();
    
    log::debug!("Multipart upload in progress!");

    // iterate over multipart stream
    while let Ok(Some(field)) = payload.try_next().await {

        // Get the content disposition from where on we process the multipart
        // A `multipart/form-data` is required to have a `content_disposition` by definition
        // The content-type is also required for every uploaded key by definition
        let content_disposition = field.content_disposition();
        let content_type = field.content_type();

        log::debug!("Field Name: {:?} | Field Content-Type: {:?}", content_disposition.get_name(), content_type);

        // Require to find the requested field name with according content type
        match accepted_files.iter().position(
            |&s| s.0 == content_disposition.get_name().map_or("", |s| s) && s.1 == &content_type.to_string() ) {
            
            Some(index) => {
                log::debug!("Found a requested type in multipart!");

                // Save everything to the filesystem (regardless whether it was a file before)
                log::debug!("Saving {:?} to fs", content_disposition.get_name());
                file_list.insert(accepted_files[index].0, field_to_file(field).await?);
                accepted_files.remove(index);
            }
            None => {
                log::debug!("Did not find field to be requested! Skipping!");
                continue;
            }
        }
    }
    
    // Verify that the requested number of files was given
    if file_list.len() == required_files as usize {
        Ok(file_list)
    } else {
        Err(ApiError::from("You did not upload the requested file count!"))
    }
}

#[allow(unused)]
/// Decode a Multipart field which has no filename
/// - considered to be a text input, as UTF8 is assumed
async fn save_multipart_json_field<T>(mut field: Field) -> Result<T, Error> 
where T: Debug + for<'de> serde::Deserialize<'de> {
    
    let mut bytes = web::BytesMut::new();

    // Field is a stream of Bytes
    while let Some(chunk) = field.try_next().await? {
        bytes.extend_from_slice(&chunk);
    }

    let json_bytes = bytes.freeze();
    
    // Create data object
    let data: T = serde_json::from_slice(&json_bytes)?;

    Ok(data)
}

/// Save multiparted file to fileystem in non-blocking fashion
async fn field_to_file(mut field: Field) -> Result<PathBuf, ApiError> {
    
    // Generate a unique file name
    let filename = Uuid::new_v4().to_string();
    let filepath = Path::new(SERVER_DATA_PATH).join(filename);
    let thread_fp = filepath.clone();

    // Create file on filesystem with the threadpool
    let mut f = web::block(move || std::fs::File::create(thread_fp))
        .await
        .map_err(|_| ApiError::from(&"Could not write given file (fs block)!".to_string()))?
        .map_err(|_| ApiError::from(&"Could not write given file!".to_string()))?;

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.try_next().await.map_err(|_| ApiError::from(&"The given file is corrupt!".to_string()))? {
        // filesystem operations are blocking, we have to use threadpool
        f = web::block(move || f.write_all(&chunk).map(|_| f))
            .await
            .map_err(|_| ApiError::from("Could not write given file to filesystem!"))?
            .map_err(|_| ApiError::from("Could not write file!"))?;
    }

    Ok(filepath)
}

/// Cleanup operation to clear uploaded files that dont need to be saved!
pub async fn remove_files(file_list: Vec<PathBuf>) {

    // Remove all uploaded files → blocking operation!
    // Remove AFTER all operations are done in parallel
    for file in file_list.iter() {
        let file_clone = file.clone();
        let _ = web::block(move || remove_file(file_clone));
    }
}

/// Parse a YAML file to a generic specified type that is Deserializable
pub fn parse_yaml_file<T>(file_path: &String) -> Result<T, Error>
where T: Debug + for<'de> serde::Deserialize<'de>
{
    // Open YAML file containing provided company information 
    let f = std::fs::File::open(file_path)?;
    let open_file_reader = BufReader::new(f);
    
    // Parse company data
    let data: T = serde_yaml::from_reader(open_file_reader).map_err(|err| Error::from(ApiError::from(&err.to_string())))?;
    
    Ok(data)
}