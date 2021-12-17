## Files

| File      | Curl                                                                                                                                       |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| case1.txt | curl -F "foo=bar" --trace-ascii /dev/stdout http://httpbin.org/post                                                                        |
| case2.txt | curl -F "foo=bar;filename=foo.txt;type=text/plain;headers=\"X-A: 1\";headers=\"X-B: 2\"" --trace-ascii /dev/stdout http://httpbin.org/post |
