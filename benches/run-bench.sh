#!/bin/bash

# Default values
DEVICE_TYPE="ping1d"
SAMPLE_SIZE=10
MEASUREMENT_TIME=60
CONNECTION_TYPE="serial"
SERIAL_PORT="/dev/ttyUSB0"
SERIAL_BAUD=115200
UDP_ADDRESS="127.0.0.1"
UDP_PORT=12345
CRITERION_ARGS=""

# Help function
show_help() {
    echo "Usage: $0 [options] [-- <criterion_args>]"
    echo ""
    echo "Options:"
    echo "  --device-type <ping1d|ping360>    Set device type (default: ping1d)"
    echo "  --sample-size <number>            Set benchmark sample size (default: 10)"
    echo "  --measurement-time <seconds>      Set benchmark measurement time in seconds (default: 60)"
    echo "  --serial <port> [baud]            Use serial connection (default port: /dev/ttyUSB0, default baud: 115200)"
    echo "  --udp <address> [port]            Use UDP connection (default address: 127.0.0.1, default port: 8080)"
    echo "  --help                            Show this help message"
    echo "  -- <criterion_args>               Pass remaining arguments directly to Criterion"
    echo ""
    echo "Examples:"
    echo "  $0 --device-type ping1d --serial /dev/ttyUSB0 115200"
    echo "  $0 --device-type ping360 --udp 192.168.1.197 12345 --sample-size 20"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --device-type)
            DEVICE_TYPE="$2"
            shift 2
            ;;
        --sample-size)
            SAMPLE_SIZE="$2"
            shift 2
            ;;
        --measurement-time)
            MEASUREMENT_TIME="$2"
            shift 2
            ;;
        --serial)
            CONNECTION_TYPE="serial"
            if [[ $# -gt 1 && ! $2 =~ ^- ]]; then
                SERIAL_PORT="$2"
                shift
            fi
            if [[ $# -gt 1 && ! $2 =~ ^- ]]; then
                SERIAL_BAUD="$2"
                shift
            fi
            shift
            ;;
        --udp)
            CONNECTION_TYPE="udp"
            if [[ $# -gt 1 && ! $2 =~ ^- ]]; then
                UDP_ADDRESS="$2"
                shift
            fi
            if [[ $# -gt 1 && ! $2 =~ ^- ]]; then
                UDP_PORT="$2"
                shift
            fi
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        --)
            shift
            CRITERION_ARGS="$*"
            break
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

if [[ "$DEVICE_TYPE" != "ping1d" && "$DEVICE_TYPE" != "ping360" ]]; then
    echo "Error: Invalid device type '$DEVICE_TYPE'. Must be either 'ping1d' or 'ping360'."
    exit 1
fi

export PING_DEVICE_TYPE="$DEVICE_TYPE"
export PING_SAMPLE_SIZE="$SAMPLE_SIZE"
export PING_MEASUREMENT_TIME="$MEASUREMENT_TIME"

if [[ "$CONNECTION_TYPE" == "serial" ]]; then
    echo "Using serial connection: $SERIAL_PORT at $SERIAL_BAUD baud"
    export PING_SERIAL_PORT="$SERIAL_PORT"
    export PING_SERIAL_BAUD="$SERIAL_BAUD"
else
    echo "Using UDP connection: $UDP_ADDRESS:$UDP_PORT"
    export PING_UDP_ADDRESS="$UDP_ADDRESS"
    export PING_UDP_PORT="$UDP_PORT"
fi

echo "============================================="
echo "Running benchmark with the following config:"
echo "Device type:       $DEVICE_TYPE"
echo "Sample size:       $SAMPLE_SIZE"
echo "Measurement time:  $MEASUREMENT_TIME seconds"
if [[ "$CONNECTION_TYPE" == "serial" ]]; then
    echo "Connection:       Serial - $SERIAL_PORT @ $SERIAL_BAUD baud"
else
    echo "Connection:       UDP - $UDP_ADDRESS:$UDP_PORT"
fi

if [[ -n "$CRITERION_ARGS" ]]; then
    echo "Criterion args:   $CRITERION_ARGS"
fi

echo "============================================="

if [[ -n "$CRITERION_ARGS" ]]; then
    cargo bench --bench bench_hil -- $BASELINE_ARGS $CRITERION_ARGS
else
    cargo bench --bench bench_hil -- $BASELINE_ARGS
fi

unset PING_DEVICE_TYPE
unset PING_SAMPLE_SIZE
unset PING_MEASUREMENT_TIME
unset PING_SERIAL_PORT
unset PING_SERIAL_BAUD
unset PING_UDP_ADDRESS
unset PING_UDP_PORT

echo "Benchmark completed."