# Hermit Crab

## Purpose
[__NIH__(Not Invented Here)](https://en.wikipedia.org/wiki/Not_invented_here)
Just for fun.

## Features & Potential Improvements
- Multi-thread-pooled TCP stream handler. Fixed number cap on threads. Each TCP session only handle a single HTTP request(No handling of "Keep-Alive" header). No mechanism to kill idle threads(No scheduler).
- TCP stream is handled as Read and Write traits. Future TLS incorporation would be easy.
- HTTP handler is stored as Arc&Mutex, which means later calls to the same endpoint need to wait for ealier call to complete. Arc&Mutex is a catch-all for HTTP handlers so that handlers(closures) can always mutate their environments. But it is not the case when handler doesn't need to mutate its environment and cause unnecessary delay.
- It can serve static directory. It currently load the entire directory into memory.(This need to be flexible) However, just like what's mentioned above, each call will lock the mutex on the handler until the call is fully reponded.
- Besides, when handling requests to static content, it should respond with "Last-Modified" or "E-Tag" for HTTP validation. It should also include "Expire" for caching. These features don't exist currently.


## Future goal
routing with macro
