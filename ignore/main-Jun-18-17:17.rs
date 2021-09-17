//! Default Compute@Edge template program.

use fastly::http::{header, HeaderValue, Method, StatusCode};
use fastly::{mime, Error, Request, Response, Body};
use std::io::{Read, Write, BufRead};
use std::ops::Index;
use fastly::http::body::StreamingBody;

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
    let req_clone = req.clone_without_body();
    let req_clone_for_match = req.clone_without_body();

    //let mut be_resp_clone = be_resp.clone_without_body();
    match (req_clone_for_match.get_method(), req_clone_for_match.get_path()) {
        (&Method::GET, url_path) => {

            let mut num_lines = 1;

            use std::borrow::Cow;
            use fastly::http::Url;

            let myurl = Url::parse(req_clone.get_url_str())?;
            //let url = req.get_url_str().parse().unwrap();
            let mut pairs = myurl.query_pairs();
            let qp_vect : Vec<_> = pairs.collect();
            let val_vect = qp_vect.into_iter().map(|x| Some(x.1).unwrap().to_string()).collect::<Vec<String>>();
            // Only expecting one query param or none
            //match pairs.next() {
            let mut pending_reqs = vec![vec![]];
            let mut pending_req_vec;
            for item in &val_vect { // You will go through every response one by one in the order you requested
                //let resource = item.to_string();
                pending_req_vec = vec![];

                // Forward the downstream request to the backend
                pending_req_vec.push(fastly::Request::get(format!("{}{}", url_path, item))
                    .with_method(Method::GET)
                    .send_async(PRIMARY_BACKEND)?);
                pending_reqs.push(pending_req_vec);
            }
            let mut be_resp;
            let mut be_body;
            let mut streaming_body;
            for idx in 0..pending_reqs.len() {

                //let mut temp_pending_req = Box::new(pending_reqs[idx]);
                //pending_reqs[idx].int (&temp_pending_req);

                let (be_resp_temp, _) = fastly::http::request::select( pending_reqs[idx]);

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

            // Done!

            Ok(())
        }
        _ => {
            // Forward the downstream request to the backend
            let mut be_resp = req.send(PRIMARY_BACKEND)?;
            Ok(())
        }
    }
}