// A manual client for a Google API (e.g. Drive), to test what makes sense and what doesn't.

mod drive_v3_types;

use drive_v3_types as drive;

use yup_oauth2::InstalledFlowAuthenticator;
use std::string::String;
use std::str::FromStr;
use std::path::Path;
use std::fs;

use hyper::Uri;
use hyper_rustls::HttpsConnector;

type TlsConnr = HttpsConnector<hyper::client::HttpConnector>;
type TlsClient = hyper::Client<TlsConnr, hyper::Body>;
type Authenticator = yup_oauth2::authenticator::Authenticator<TlsConnr>;

fn https_client() -> TlsClient {
    let conn = hyper_rustls::HttpsConnector::new();
    let cl = hyper::Client::builder().build(conn);
    cl
}

async fn upload_file(cl: &mut TlsClient, auth: &mut Authenticator, f: &Path) {
    let posturl = "https://www.googleapis.com/upload/drive/v3/files?uploadType=media";
    let tok = auth.token(&["https://www.googleapis.com/auth/drive.file"]).await.unwrap();
    let authtok = format!("&oauth_token={}&fields=*", tok.as_str());

    let file = fs::OpenOptions::new().read(true).open(f).unwrap();
    let len = file.metadata().unwrap().len();

    let body = hyper::Body::from(fs::read(&f).unwrap());
    let req = hyper::Request::post(posturl.to_string()+&authtok).header("Content-Length", format!("{}", len))
        .body(body).unwrap();
    let resp = cl.request(req).await.unwrap();

    let body = resp.into_body();
    let body = hyper::body::to_bytes(body).await.unwrap();
    let dec = String::from_utf8(body.to_vec()).unwrap();
    let about: drive::File = serde_json::from_str(&dec).unwrap();
    println!("{:?}", about);
}

async fn get_about(cl: &mut TlsClient, auth: &mut Authenticator) {
    let baseurl = "https://www.googleapis.com/drive/v3/";
    let path = "about";
    let tok = auth.token(&["https://www.googleapis.com/auth/drive.file"]).await.unwrap();
    let authtok = format!("?oauth_token={}&fields=*", tok.as_str());

    let resp = cl.get(Uri::from_str(&(String::from(baseurl)+path+&authtok)).unwrap()).await.unwrap();
    let body = resp.into_body();
    let body = hyper::body::to_bytes(body).await.unwrap();
    let dec = String::from_utf8(body.to_vec()).unwrap();
    let about: drive::About = serde_json::from_str(&dec).unwrap();
    println!("{:?}", about);
}

#[tokio::main]
async fn main() {
    let sec = yup_oauth2::read_application_secret("client_secret.json")
        .await
        .expect("client secret couldn't be read.");

    let mut auth = InstalledFlowAuthenticator::builder(sec, yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .expect("installed flow authenticator!");

    let scopes = &["https://www.googleapis.com/auth/drive.file"];

    let mut cl = https_client();

    //get_about(&mut cl, &mut auth).await;
    upload_file(&mut cl, &mut auth, Path::new("pp.jpg")).await;

    match auth.token(scopes).await {
        Ok(token) => println!("The token is {:?}", token),
        Err(e) => println!("error: {:?}", e),
    }
}
