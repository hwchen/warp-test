potential sticking points:
- easily writing blocking operations
- streaming response (chunked transfer encoding)

Blocking operations can be handled, but seems a little awkward: https://github.com/seanmonstar/warp/issues/185

Streaming responses seem to be handled by having response be `Response<impl Into<hyper::Body>>`. https://github.com/seanmonstar/warp/issues/38 . But I just haven't tested yet, probably won't until absolutely necessary.
