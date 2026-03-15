wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

local counter = 0

request = function()
    counter = counter + 1
    local body = string.format('{"body":"bench-msg-%d-%d"}', counter, math.random(1000000))
    return wrk.format(nil, nil, nil, body)
end

done = function(summary, latency, requests)
    io.write("--- BENCH SUMMARY ---\n")
    io.write(string.format("Total requests: %d\n", summary.requests))
    io.write(string.format("Duration (s): %.2f\n", summary.duration / 1e6))
    io.write(string.format("Requests/sec: %.2f\n", summary.requests / (summary.duration / 1e6)))
    io.write(string.format("Avg latency (ms): %.2f\n", latency.mean / 1e3))
    io.write(string.format("Max latency (ms): %.2f\n", latency.max / 1e3))
    io.write(string.format("Stdev latency (ms): %.2f\n", latency.stdev / 1e3))
    io.write(string.format("Errors connect: %d\n", summary.errors.connect))
    io.write(string.format("Errors read: %d\n", summary.errors.read))
    io.write(string.format("Errors write: %d\n", summary.errors.write))
    io.write(string.format("Errors status: %d\n", summary.errors.status))
    io.write(string.format("Errors timeout: %d\n", summary.errors.timeout))
    io.write("--- END SUMMARY ---\n")
end
