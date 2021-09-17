//! Default Compute@Edge template program.

use fastly::http::{header, HeaderValue, Method, StatusCode};
use fastly::{mime, Error, Request, Response, Body};
use std::io::{Read, Write, BufRead, BufWriter};
use std::ops::Index;
use fastly::http::body::StreamingBody;
use fastly::http::request::PendingRequest;
use std::sync::mpsc::SendError;
use std::ptr::{null, null_mut};
use urlencoding::encode;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// The name of a backend server associated with this service.
///
/// This should be changed to match the name of your own backend. See the the `Hosts` section of
/// the Fastly WASM service UI for more information.
const PRIMARY_BACKEND: &str = "primary_backend";

/// The name of a second backend associated with this service.
const SECONDARY_BACKEND: &str = "secondary_backend";

/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.
//#[fastly::main]
//fn main(mut req: Request) -> Result<Response, Error> {
fn main() -> Result<(), Error> {
    // Make any desired changes to the client request.
    let req = Request::from_client();
    let mut req_clone = req.clone_without_body();
    let mut req_clone_for_match = req.clone_without_body();

    match (req_clone_for_match.get_method(), req_clone_for_match.get_path()) {
        (&Method::GET, url_path) => {
            let mut num_lines = 1;

            use std::borrow::Cow;
            use fastly::http::Url;
            if req_clone.get_url_str().contains("path") {
                println!("This is the url received with path {}", req_clone.get_url_str());
                let myurl = Url::parse(req_clone.get_url_str())?;
                //let url = req.get_url_str().parse().unwrap();
                let mut pairs = myurl.query_pairs();
                let qp_vect: Vec<_> = pairs.collect();

                let val_vect = qp_vect.into_iter().map(|x| Some(x.1).unwrap().to_string()).collect::<Vec<String>>();

                req_clone.remove_query();
                let mut pending_req_vec = vec![];

                /*println!("Path 1.0: We will fetch this next {}", utf8_percent_encode(&format!("{}{}",req_clone.get_url(), val_vect[0]), FRAGMENT).to_string());
                let mut be_resp = fastly::Request::new(Method::GET, utf8_percent_encode(&format!("{}{}", req_clone.get_url(), val_vect[0]), FRAGMENT).to_string())
                    .send_async(PRIMARY_BACKEND)?
                    .wait()?;

                let pending_req_val = fastly::Request::get(format!("{}{}", req_clone.get_url(), val_vect[0]))
                    .with_method(Method::GET)
                    .send_async(PRIMARY_BACKEND)?;
                pending_req_vec.push(pending_req_val);*/


            for item in &val_vect { // You will go through every response one by one in the order you requested
                let resource = item.to_string();
                //pending_req_vec = vec![];

                // Forward the downstream request to the backend
                println!("Path 1.0: We will fetch this next {}", utf8_percent_encode(&format!("{}{}",req_clone.get_url(), item), FRAGMENT).to_string());
                //println!("Path 1.1: We will fetch this next {}", format!("{}{}", url_path, item));
                let pending_req_val = fastly::Request::get(format!("{}{}", req_clone.get_url(), item))
                    .with_method(Method::GET)
                    .send_async(PRIMARY_BACKEND)?;
                //let pending_req_vec = vec![pending_req_val];
                //pending_reqs.push(pending_req_vec);
                pending_req_vec.push(pending_req_val);
            }
            let mut be_resp;
                // Take the body out of beresp and replace it with a new empty body that we can stream to
            let mut be_body = fastly::Body::new();
            let mut streaming_body= fastly::Response::new().stream_to_client();
            //let mut streaming_body_temp = Body::new();
            let mut pending_vec_len = pending_req_vec.len();
            let mut val_vect_len = val_vect.len();
            //for idx in 0..pending_reqs.len() {
            for req in pending_req_vec {
                // Send two asynchronous requests, and store the pending requests in a vector.

                use std::sync::Arc;

                //let mut temp_vec = pending_reqs[idx].to_vec();
                let mut be_resp_temp =  req.wait()?;
                println!("Got a response");
                be_resp = be_resp_temp.clone_with_body();
                //let mut be_resp_clone = be_resp_temp.clone_without_body();
                if val_vect_len == pending_vec_len { // We only want to do this once for the first response
                println!("will  stream to client");
                    //val_vect_len += 1;
                    // Take the body out of beresp and replace it with a new empty body that we can stream to
                    //let be_body = std::mem::replace(be_resp_temp.get_body_mut(), Body::new());
                    be_body = std::mem::replace(be_resp_temp.get_body_mut(), Body::new());
                    // Send the response headers downstream, and get ahold of the streaming body
                    //drop(streaming_body);
                    //streaming_body = be_resp_temp.stream_to_client();
                    //drop(be_body);
                    for chunk in be_body.read_chunks(8192) {
                        //for chunk in be_body.read_chunks(8192) {
                        let mut chunk = chunk.unwrap();

                        streaming_body.write_bytes(&chunk);
                        num_lines += 1;
                    }
                }

                //Response::from(streaming_body_temp).send_to_client();


                //if be_resp_clone.get_status() == StatusCode::OK {
                if val_vect_len != pending_vec_len {
                    for chunk in be_resp.get_body_mut().read_chunks(8192) {
                        //for chunk in be_body.read_chunks(8192) {
                        let mut chunk = chunk.unwrap();

                        streaming_body.write_bytes(&chunk);
                        //streaming_body.append(&chunk);
                        num_lines += 1;
                    }
                }
                pending_vec_len -= 1;
                //}
                /*if val_vect_len == pending_vec_len {
                    //if be_resp_clone.get_status() == StatusCode::OK {
                    for chunk in be_body.read_chunks(8192) {
                        //for chunk in be_body.read_chunks(8192) {
                        let mut chunk = chunk.unwrap();

                        streaming_body.write_bytes(&chunk);
                        num_lines += 1;
                    }
                    //}
                } else {
                    for chunk in be_resp_temp.get_body_mut().read_chunks(32768) {
                        let mut chunk = chunk.unwrap();

                        //streaming_body.write_bytes(&chunk);
                        streaming_body.write_all(&chunk);
                        num_lines += 1;
                    }
                }*/
                println!("Path2: Loop ran {} times", num_lines);
            }
            // Drop the streaming body so the response will finish sending.
            drop(streaming_body);

                // Done!
            } else {
                println!("Path 2.0: Fetching this url {}", req_clone.get_url_str());
                let mut be_resp = req.send(PRIMARY_BACKEND)?;
                use std::borrow::Cow;
                use fastly::http::Url;

                let myurl = Url::parse(req_clone.get_url_str())?;
                //let url = req.get_url_str().parse().unwrap();
                let mut pairs = myurl.query_pairs();
                let mut be_body = std::mem::replace(be_resp.get_body_mut(), Body::new());
                let mut range_val = "-1";
                if let Some(x) = req_clone.get_header_str("range") {
                    range_val = x;
                    println!("Range val received is {}", range_val);
                    let pair: Vec<_> = range_val.split('=').collect();
                    println!("This is what was split {:?}", pair);
                    let val_pair: Vec<_> = pair[1].split('-').collect();
                    println!("This is what was split {:?}", val_pair);
                    println!("Range value from is {}", val_pair[0]);
                    range_val = val_pair[0];
                }

                let mut range_val_num = -1;
                let mut line = range_val.to_string();
                if let Some (x) = line
                    .trim()
                    .parse::<i32>().ok() {
                    println!("Setting range value to {}", x);
                    range_val_num = x;
                }
                println!("The range value requested is {}", range_val_num);

                let mut streaming_body = be_resp.stream_to_client();
                //println!("Path2.1: Loop ran {} times", num_lines);
                for chunk in be_body.read_chunks(8192) {

                    let mut chunk = chunk.unwrap();
                    println!("Path2.2: Loop ran {} times with length written {}", num_lines, chunk.len());
                    //chunk.retain(|b| *b != 0);
                    streaming_body.write_bytes(&chunk);
                    num_lines += 1;
                }
                println!("Path2.3: Loop ran {} times", num_lines);

                // Drop the streaming body so the response will finish sending.
                drop(streaming_body);
            }
            Ok(())
        }
        _ => {
            // Forward the downstream request to the backend
            let mut be_resp = req.send(PRIMARY_BACKEND)?;
            Ok(())
        }
    }
}