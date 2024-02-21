#!/bin/bash
set -euxo pipefail

if [[ -z "$AWS_ACCESS_KEY_ID" || -z "$AWS_SECRET_ACCESS_KEY"  ]]; then
    echo "Must provide AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY"
    exit 1
fi

cd "$(dirname "$0")"

terraform plan
terraform apply -auto-approve
public_ip=$(terraform output --raw public_ip)
sleep 30
ssh ec2-user@$public_ip "
    sudo yum update -y &&
    sudo yum groupinstall -y 'Development Tools' &&
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y &&
    source "\$HOME/.cargo/env" &&
    git clone https://github.com/nickbabcock/highway-rs.git &&
    cd highway-rs &&
    cargo test"

if [[ $? = 0 ]]; then
  read -p "Test successful, preserve instance? (y/N): " -n 1 -r -t 300
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    terraform destroy -auto-approve
  fi
else
  echo "Test failed"
fi
