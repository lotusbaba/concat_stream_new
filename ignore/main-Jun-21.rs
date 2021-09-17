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

    //let mut be_resp_clone = be_resp.clone_without_body();
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
                // Only expecting one query param or none
                //match pairs.next() {

                let mut be_body;
                //let mut first_run = true;
                //println!("Path 1.0: We will fetch this first {}", format!("{}{}", url_path, val_vect[0]));
                req_clone.remove_query();
                println!("Path 1.0: We will fetch this next {}", utf8_percent_encode(&format!("{}{}",req_clone.get_url(), val_vect[0]), FRAGMENT).to_string());
                let mut be_resp = fastly::Request::new(Method::GET, utf8_percent_encode(&format!("{}{}", req_clone.get_url(), val_vect[0]), FRAGMENT).to_string())
                    .send_async(PRIMARY_BACKEND)?
                    .wait()?;
                let be_resp_clone = be_resp.clone_without_body();
                be_body = std::mem::replace(be_resp.get_body_mut(), Body::new());
                let mut streaming_body = be_resp.stream_to_client();

                for chunk in be_body.read_chunks(8192) {
                    let mut chunk = chunk.unwrap();

                    streaming_body.write_bytes(&chunk);
                    num_lines += 1;
                }
                println!("Path1: Loop ran {} and received status {}", num_lines, StatusCode::OK);

                if be_resp_clone.get_status() == StatusCode::OK {
                    for idx in 1..val_vect.len() { // You will go through every response one by one in the order you requested
                        println!("Path 1.0: We will fetch this next {}", utf8_percent_encode(&format!("{}{}",req_clone.get_url(), val_vect[idx]), FRAGMENT).to_string());
                        // Forward the request to the backend
                        let mut be_resp_temp = fastly::Request::new(Method::GET, utf8_percent_encode(&format!("{}{}", req_clone.get_url(), val_vect[idx]), FRAGMENT).to_string())
                            .send_async(PRIMARY_BACKEND)?
                            .wait()?;

                        /*if first_run { // We only want to do this once for the first response
                    first_run = false;
                    be_resp = be_resp_temp?;

                    // Take the body out of beresp and replace it with a new empty body that we can stream to
                    be_body = std::mem::replace(be_resp.get_body_mut(), Body::new());
                    // Send the response headers downstream, and get ahold of the streaming body

                    streaming_body_vec.push( be_resp.stream_to_client());
                }*/
                        num_lines = 0;
                        for chunk in be_resp_temp.get_body_mut().read_chunks(8192) {
                            let mut chunk = chunk.unwrap();

                            streaming_body.write_bytes(&chunk);
                            num_lines += 1;
                        }
                        println!("Path2: Loop ran {} times", num_lines);
                    }
                }

                //impl Copy for PendingRequest {}
                //impl Clone for PendingRequest { fn clone(&self) -> PendingRequest { *self } }
                //let mut pending_reqs = vec![vec![]];
                //let mut pending_req_vec;
                //let mut pending_req;
                //for item in &val_vect { // You will go through every response one by one in the order you requested
                //let resource = item.to_string();
                //pending_req_vec = vec![];

                // Forward the downstream request to the backend

                /*:pending_req_vec.push(fastly::Request::get(format!("{}{}", url_path, item))
                    .with_method(Method::GET)
                    .send_async(PRIMARY_BACKEND)?);
                pending_reqs.push(pending_req_vec);*/
                //}
                /*            let mut be_resp;
            let mut be_body;
            let mut streaming_body;
            for idx in 0..pending_reqs.len() {
                // Send two asynchronous requests, and store the pending requests in a vector.

                use std::sync::Arc;
                //let temp = Arc::new(pending_reqs).to_owned();
                let temp = Arc::new(pending_reqs).clone();
                //let temp = Arc::new(pending_reqs[idx]).to_owned();
                //use std::cell::Cell;
                //let c = Cell::new(vec![5,4]);
                //println!("{}", c.get());
                //let temp = Cell::new(&pending_reqs[idx]).clone();

                //let (be_resp_temp, _) = fastly::http::request::select( temp.get(idx).as_deref());
                let (be_resp_temp, _) = fastly::http::request::select( temp.to_owned()[idx]);

                //let mut be_resp = req.send(PRIMARY_BACKEND))?;

                if idx == 0 { // We only want to do this once for the first response
                    be_resp = be_resp_temp?;

                    // Take the body out of beresp and replace it with a new empty body that we can stream to
                    be_body = std::mem::replace(be_resp.get_body_mut(), Body::new());
                    // Send the response headers downstream, and get ahold of the streaming body

                    streaming_body = be_resp.stream_to_client();
                }
                //println!("Path2.1: Loop ran {} times", num_lines);
                for chunk in be_body.read_chunks(8192) {
                    let mut chunk = chunk.unwrap();

                    streaming_body.write_bytes(&chunk);
                    num_lines += 1;
                }
                println!("Path2: Loop ran {} times", num_lines);
            }
            // Drop the streaming body so the response will finish sending.
            drop(streaming_body);
*/
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
                // Send the response headers downstream, and get ahold of the streaming body
                //be_resp.set_status(StatusCode::from_u16(206));
                //if (range_val_num == 0) | (range_val_num < be_resp.get_content_length().unwrap())  {
                /*if range_val_num >= 0 && range_val_num < be_resp.get_content_length().unwrap() as i32  {
                    println!("Setting partial content");
                    be_resp.set_status(StatusCode::PARTIAL_CONTENT);
                    if range_val_num == 0 {
                        //let val = format!("bytes 0-{}/{}", 4095, be_resp.get_content_length().unwrap());
                        let local_be_resp = be_resp.clone_with_body();
                        //let val = format!("bytes 0-{}/{}", local_be_resp.into_body_bytes().len(), be_resp.get_content_length().unwrap());
                        //let val2 = format!("{}", be_resp.get_content_length().unwrap());
                        //be_resp.set_header("Content-Length", val2);
                        let val = format!("bytes 0-{}/{}", be_resp.get_content_length().unwrap()-1, be_resp.get_content_length().unwrap());
                        be_resp.set_header("Content-Range", val);
                    }
                }*/
                //println!("Path2.0: Loop ran {} times", num_lines);
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