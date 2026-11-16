# The provider implements the provider-neutral contract in modules/host. The first six variables
# match that contract. The rest are OVH order details. Items marked [U] come from the OVH order
# catalogue, which Brighton reads at T23 (deploy-ops tasks.md). Placeholders in
# envs/prod/terraform.tfvars.example are never real values.

variable "name" {
  type        = string
  description = "Host name, for example counterspell-prod-1."

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{2,62}$", var.name))
    error_message = "name must be 3 to 63 lower-case letters, digits or hyphens, starting with a letter."
  }
}

variable "datacentre" {
  type        = string
  description = "OVH datacentre code for the vps_datacenter option, for example GRA or RBX [U]. EU only."
}

variable "plan" {
  type        = string
  description = "OVH plan code for the VPS-2 offer [U], read from the order catalogue."
}

variable "image" {
  type        = string
  description = "OS configuration value for the vps_os option, for example Ubuntu 24.04 [U]."
}

variable "ssh_public_keys" {
  type        = list(string)
  description = "Public SSH keys. OVH installs only the first one; the base role adds the rest."

  validation {
    condition     = length(var.ssh_public_keys) > 0
    error_message = "At least one public SSH key is required so the host can be reached."
  }
}

variable "ovh_subsidiary" {
  type        = string
  description = "OVH subsidiary of the billing entity, for example IE or DE [U]. Read from /me in the OVH API."
}

variable "vps_image_id" {
  type        = string
  description = "Image UUID for the OS, required when a public SSH key is set [U]. Read from the OVH image list."
}

variable "vps_duration" {
  type        = string
  description = "ISO 8601 subscription duration, for example P1M for a month [U]."
  default     = "P1M"
}

variable "vps_pricing_mode" {
  type        = string
  description = "Pricing mode that matches the duration [U]."
  default     = "default"
}
