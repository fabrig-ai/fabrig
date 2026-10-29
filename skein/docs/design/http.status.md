# http: status

2026-10-29. Where skein's code lags or departs from [http.md](http.md).

- **Built:** 3: gzip encoder. 3.5, 4.2, 5.6: edge validity and scenario ceilings without client, SSE or server heap sums.
- **Not yet:** 3 to 5, 8: chunked uploads, other content codings, trailers, HTTP/2, upgrades, event-stream reconnect/retry, simultaneous request/response bodies, peer pools, 100-continue waiting and server deadlines. 6: selective collector, metered protocol worlds and fuzz targets.
