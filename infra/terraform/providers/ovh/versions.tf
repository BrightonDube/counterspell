terraform {
  required_version = ">= 1.9.0"

  required_providers {
    ovh = {
      source = "ovh/ovh"
      # Pinned exactly. Upgrades are a reviewed change (deploy-ops R1.2).
      version = "= 2.25.0"
    }
  }
}
