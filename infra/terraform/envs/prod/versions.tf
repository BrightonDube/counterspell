terraform {
  required_version = ">= 1.9.0"

  required_providers {
    ovh = {
      source  = "ovh/ovh"
      version = "= 2.25.0"
    }
  }
}

# Credentials come from the shell (application key, application secret and consumer key). Nothing is
# written to disk (deploy-ops design, Provisioning: Terraform).
provider "ovh" {
  endpoint = "ovh-eu"
}
