#!/bin/bash

if [ -z "$1" ]; then
  echo "Node name should be passed as an argument"
  echo "Usage: $0 <node_name>"
  echo "Example: $0 node1"
  exit 1
fi


NODE="$1"
DIR="$HOME/ethshadow/data/shadow/hosts/$NODE"
PCAP_FILE="$DIR/eth0.pcap"
CSV_FILE="$DIR/${NODE}.csv"
OUTPUT_CL="$DIR/${NODE}_cl.csv"
OUTPUT_EL="$DIR/${NODE}_el.csv"

if [ ! -f "$PCAP_FILE" ]; then
  echo "PCAP file does not exist: $PCAP_FILE"
  exit 1
fi

tshark -r "$PCAP_FILE" \
  -T fields \
  -e frame.time_epoch -e frame.len -e tcp.srcport -e tcp.dstport -e udp.srcport -e udp.dstport \
  -E header=y -E separator=, -E quote=d -E occurrence=f \
  > "$CSV_FILE"

if [ ! -f "$CSV_FILE" ]; then
  echo "Failed to create CSV file: $CSV_FILE"
  exit 1
fi


HEADER=$(head -n 1 "$CSV_FILE")
echo "$HEADER" > "$OUTPUT_CL"
echo "$HEADER" > "$OUTPUT_EL"

echo "CSV file format: $HEADER"

tail -n +2 "$CSV_FILE" | while IFS=, read -r epoch len tcp_src tcp_dst udp_src udp_dst
do
  tcp_src=$(echo "$tcp_src" | tr -d ' "')
  tcp_dst=$(echo "$tcp_dst" | tr -d ' "')
  udp_src=$(echo "$udp_src" | tr -d ' "')
  udp_dst=$(echo "$udp_dst" | tr -d ' "')
  line="$epoch,$len,$tcp_src,$tcp_dst,$udp_src,$udp_dst"

  if [[ "$tcp_src" == "31000" || "$tcp_dst" == "31000" || "$udp_src" == "31000" || "$udp_dst" == "31000" ]]; then
    echo "$line" >> "$OUTPUT_CL"
  elif [[ "$tcp_src" == "21000" || "$tcp_dst" == "21000" || "$udp_src" == "21000" || "$udp_dst" == "21000" ]]; then
    echo "$line" >> "$OUTPUT_EL"
  else
    echo "$line"
  fi
done