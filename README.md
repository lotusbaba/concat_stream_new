# concat_stream_new

This is a Fastly Ceompute@Edge based Rust app which will build into a wasm binary. You can use it to concatenate the following 3 different mp3 streams in any order - https://concat-stream-new.edgecompute.app/?path=splitFile-segment-0000.mp3&path=splitFile-segment-0001.mp3&path=splitFile-segment-0002.mp3

Or concatenate any number of them - https://concat-stream-new.edgecompute.app/?path=splitFile-segment-0001.mp3&path=splitFile-segment-0002.mp3

Or any number of times - https://concat-stream-new.edgecompute.app/?path=splitFile-segment-0000.mp3&path=splitFile-segment-0001.mp3&path=splitFile-segment-0002.mp3&path=splitFile-segment-0001.mp3&path=splitFile-segment-0002.mp3
