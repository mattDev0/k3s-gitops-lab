# Import existing dev-workspace Linode VM (Linode ID: 109711421)
import {
  to = linode_instance.dev_workspace
  id = "109711421"
}

# Describes the VM I created by hand in the Cloud Manager (Ubuntu 24.04). `image` is left
# out on purpose: the API doesn't return it for an existing Linode, and setting it would
# make Terraform plan to rebuild the VM.
resource "linode_instance" "dev_workspace" {
  label  = "dev-workspace"
  region = "gb-lon"
  type   = "g6-standard-4"

  lifecycle {
    prevent_destroy = true
    # Provider-only settings for future resizes and migrations. An imported instance has
    # no stored value for them, so without this the plan shows a no-op "update in-place".
    ignore_changes = [migration_type, resize_disk]
  }
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

  lifecycle {
    prevent_destroy = true
  }
}
