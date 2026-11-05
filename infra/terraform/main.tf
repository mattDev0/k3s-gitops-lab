# Import existing dev-workspace Linode VM (Linode ID: 109711421)
import {
  to = linode_instance.dev_workspace
  id = "109711421"
}

resource "linode_instance" "dev_workspace" {
  label  = "dev-workspace"
  region = "gb-lon"
  type   = "g6-standard-4"
  image  = "linode/ubuntu24.04"
}

# Import existing dev-workspace Cloud Firewall (Firewall ID: 229456920)
import {
  to = linode_firewall.dev_workspace_fw
  id = "229456920"
}

resource "linode_firewall" "dev_workspace_fw" {
  label           = "dev-workspace-fw"
  inbound_policy  = "DROP"
  outbound_policy = "ACCEPT"

  linodes = [
    linode_instance.dev_workspace.id,
  ]
}
