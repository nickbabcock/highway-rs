terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}

resource "aws_instance" "highway_aarch64" {
  ami           = "ami-055859c8e0f361065"
  instance_type = "t4g.small"
  key_name      = "me"
}

output "public_ip" {
  value = aws_instance.highway_aarch64.public_ip
}
