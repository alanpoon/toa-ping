# toa-ping

[![Build status](https://ci.appveyor.com/api/projects/status/19begg2drvh0630n/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/toa-ping/branch/master)
[![Build Status](https://travis-ci.org/DoumanAsh/toa-ping.svg?branch=master)](https://travis-ci.org/DoumanAsh/toa-ping)

Utility to perform pings.

Supported protocols:
* TCP

Named after cute [Toa](https://vndb.org/c34928)

## Usage

```
usage: toa-ping [flags] [options] <destination>

Performs ping toward destination.

Destination format: <host>[:<port>]

Flags:
  -h, --help    - Prints this message.
  -f, --forever - Keep going forever.

Options:
  -p <protocol> - Specifies protocol to use. Default is tcp.
  -n <number>   - Number of pings to send. Default is 4.
  -i <interval> - Time interval between pings in milliseconds. Default is 500.
  -w <timeout>  - Time to wait for each response in milliseconds. Default is 1000.
  -4            - Enforce IPv4 version. Default is first resolved address.
  -6            - Enforce IPv6 version. Default is first resolved address.

Supported protocols:
  tcp - Measures RTT of connection establishment.

```
