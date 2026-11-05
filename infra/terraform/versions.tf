terraform {
  required_version = ">= 1.5.0, < 2.0.0"

  required_providers {
    linode = {
      source  = "linode/linode"
      version = "4.9.0"
    }
  }
}

provider "linode" {
  # Token is supplied exclusively via the LINODE_TOKEN environment variable.
}
